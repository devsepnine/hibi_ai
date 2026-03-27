use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::thread;
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, poll};

use crate::app::{App, TargetCli, View};
use crate::component;
use crate::fs;
use crate::mcp;
use crate::plugin;
use crate::process_exec;

pub(crate) type RefreshResult = (
    Vec<component::Component>,
    Vec<mcp::McpServer>,
    Vec<plugin::Plugin>,
    Vec<String>,
);

/// Bundles all channels and state for async process management.
pub(crate) struct ProcessingChannels {
    process_tx: Sender<Result<String>>,
    process_rx: Receiver<Result<String>>,
    cancel_tx: Sender<()>,
    cancel_rx: Receiver<()>,
    current_cancel_tx: Sender<()>,
    pub(crate) processing_active: bool,
    pub(crate) refresh_tx: Sender<Result<RefreshResult>>,
    pub(crate) refresh_rx: Receiver<Result<RefreshResult>>,
}

impl ProcessingChannels {
    pub(crate) fn new() -> Self {
        let (process_tx, process_rx) = mpsc::channel::<Result<String>>();
        let (cancel_tx, cancel_rx) = mpsc::channel::<()>();
        let current_cancel_tx = cancel_tx.clone();
        let (refresh_tx, refresh_rx) = mpsc::channel::<Result<RefreshResult>>();

        Self {
            process_tx,
            process_rx,
            cancel_tx,
            cancel_rx,
            current_cancel_tx,
            processing_active: false,
            refresh_tx,
            refresh_rx,
        }
    }

    /// Replace the cancel channel pair, returning the old receiver for thread use.
    fn take_cancel_rx(&mut self) -> Receiver<()> {
        let (new_tx, new_rx) = mpsc::channel::<()>();
        let old_rx = std::mem::replace(&mut self.cancel_rx, new_rx);
        self.cancel_tx = new_tx;
        old_rx
    }

    /// Reset the cancel channel (used after process completion).
    fn reset_cancel_channel(&mut self) {
        let (new_tx, new_rx) = mpsc::channel::<()>();
        self.cancel_tx = new_tx;
        self.cancel_rx = new_rx;
    }
}

/// Handle completion of a processing thread.
fn handle_process_completion(app: &mut App, channels: &mut ProcessingChannels) {
    match channels.process_rx.try_recv() {
        Ok(result) => {
            channels.processing_active = false;
            app.cancelling = false;
            match result {
                Ok(msg) => app.processing_log.push(msg),
                Err(e) => {
                    let err_msg = e.to_string();
                    if err_msg.contains("Cancelled by user") {
                        app.processing_log.push("[WARN] Cancelled by user".to_string());
                        if !app.is_removing {
                            app.processing_log.push("[INFO] Cleaning up cancelled installation...".to_string());
                        }
                        app.processing_queue.clear();
                    } else if err_msg.contains("timed out") {
                        app.processing_log.push(format!("[ERR] {}", err_msg));
                        if !app.is_removing {
                            app.processing_log.push("[INFO] Cleaning up timed out installation...".to_string());
                        }
                    } else {
                        app.processing_log.push(format!("[ERR] {}", err_msg));
                    }
                }
            }

            let progress = app.processing_progress.unwrap_or(0) + 1;
            app.processing_progress = Some(progress);
            channels.reset_cancel_channel();

            if app.processing_queue.is_empty() {
                app.start_finish_processing();
            }
        }
        Err(TryRecvError::Empty) => {}
        Err(TryRecvError::Disconnected) => {
            channels.processing_active = false;
            app.processing_log.push("[ERR] Process thread crashed".to_string());
            if app.processing_queue.is_empty() {
                app.start_finish_processing();
            }
        }
    }
}

/// Dequeue and spawn the next processing task.
fn dispatch_next_process(app: &mut App, channels: &mut ProcessingChannels) {
    let idx = app.processing_queue.remove(0);
    channels.processing_active = true;

    let item_name = process_exec::get_item_name(app, idx);
    let action = if app.is_removing { "Removing" } else { "Installing" };
    app.processing_log.push(format!("{} {}...", action, item_name));

    let tx_clone = channels.process_tx.clone();
    let is_removing = app.is_removing;
    let target_cli = app.target_cli.unwrap_or(TargetCli::Claude);
    let data = match process_exec::prepare(app, idx) {
        Some(d) => d,
        None => {
            channels.processing_active = false;
            app.processing_log.push(format!("[ERR] Invalid item index: {}", idx));
            return;
        }
    };

    // Update current_cancel_tx BEFORE spawning the thread
    channels.current_cancel_tx = channels.cancel_tx.clone();
    let cancel_rx_for_thread = channels.take_cancel_rx();

    thread::spawn(move || {
        let result = process_exec::execute(data, is_removing, target_cli, cancel_rx_for_thread);
        let _ = tx_clone.send(result);
    });
}

/// Start a background thread to rescan components after install/remove.
fn start_refresh_thread(app: &mut App, refresh_tx: &Sender<Result<RefreshResult>>) {
    app.refreshing = true;

    let tx_clone = refresh_tx.clone();
    let sources = app.sources.clone();
    let dest_dir = app.dest_dir.clone();
    let target_cli = app.target_cli.unwrap_or(TargetCli::Claude);

    thread::spawn(move || {
        let result = (|| -> Result<RefreshResult> {
            let components = fs::scanner::scan_all_sources(&sources, &dest_dir, target_cli)?;
            let (mcp_servers, _warning) = fs::scanner::scan_all_mcp_sources(&sources, target_cli)?;
            let plugins = fs::scanner::scan_all_plugin_sources(&sources)?;
            Ok((components, mcp_servers, plugins, Vec::new()))
        })();
        let _ = tx_clone.send(result);
    });
}

/// Check if a refresh thread has completed and apply results.
fn check_refresh_completion(app: &mut App, refresh_rx: &Receiver<Result<RefreshResult>>) {
    match refresh_rx.try_recv() {
        Ok(Ok((components, mcp_servers, plugins, _))) => {
            app.apply_refresh_result(components, mcp_servers, plugins);
        }
        Ok(Err(e)) => {
            app.processing_log.push(format!("[ERROR] Refresh failed: {}", e));
            app.needs_refresh = false;
            app.refreshing = false;
            app.processing_complete = true;
        }
        Err(TryRecvError::Empty) => {}
        Err(TryRecvError::Disconnected) => {
            app.processing_log.push("[ERROR] Refresh thread crashed".to_string());
            app.needs_refresh = false;
            app.refreshing = false;
            app.processing_complete = true;
        }
    }
}

/// Handle input during the Installing view.
fn handle_installing_input(
    app: &mut App,
    key: KeyCode,
    cancel_tx: &Sender<()>,
    processing_active: &bool,
) -> Result<()> {
    match key {
        KeyCode::Esc => {
            if *processing_active && !app.cancelling {
                let _ = cancel_tx.send(());
                app.processing_log.push("[WARN] Cancelling current operation...".to_string());
                app.cancelling = true;
            } else if app.processing_complete {
                app.close_processing();
            }
        }
        KeyCode::Char('q') | KeyCode::Enter => {
            if app.processing_complete {
                app.close_processing();
            }
        }
        _ => {}
    }
    Ok(())
}

/// Handle a single tick of the Installing view.
pub(crate) fn handle_installing_view(app: &mut App, channels: &mut ProcessingChannels) -> Result<()> {
    if poll(Duration::from_millis(100))? {
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Release {
                handle_installing_input(app, key.code, &channels.current_cancel_tx, &channels.processing_active)?;
            }
        }
    }

    app.tick();

    if channels.processing_active {
        handle_process_completion(app, channels);
    }

    if !channels.processing_active && !app.processing_queue.is_empty() {
        dispatch_next_process(app, channels);
    } else if !channels.processing_active && app.processing_queue.is_empty() && app.needs_refresh && !app.refreshing {
        start_refresh_thread(app, &channels.refresh_tx);
    } else if app.refreshing {
        check_refresh_completion(app, &channels.refresh_rx);
    }

    Ok(())
}

/// Handle a single tick of the Loading view.
pub(crate) fn handle_loading_view(app: &mut App, refresh_rx: &Receiver<Result<RefreshResult>>) -> Result<()> {
    if poll(Duration::from_millis(100))? {
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Release && key.code == KeyCode::Char('q') {
                app.should_quit = true;
            }
        }
    }

    app.tick();

    match refresh_rx.try_recv() {
        Ok(Ok((components, mcp_servers, plugins, cleaned_hooks))) => {
            app.finish_loading(components, mcp_servers, plugins, cleaned_hooks);
        }
        Ok(Err(e)) => {
            app.status_message = Some(format!("Error loading: {}", e));
            app.current_view = View::CliSelection;
        }
        Err(TryRecvError::Empty) => {}
        Err(TryRecvError::Disconnected) => {
            app.status_message = Some("Loading failed".to_string());
            app.current_view = View::CliSelection;
        }
    }

    Ok(())
}

/// Start a background thread to scan all sources for initial loading.
pub(crate) fn start_loading_thread(
    app: &App,
    refresh_tx: &Sender<Result<RefreshResult>>,
) {
    let tx_clone = refresh_tx.clone();
    let source_dir = app.source_dir.clone();
    let sources = app.sources.clone();
    let dest_dir = app.dest_dir.clone();
    let target_cli = app.target_cli.unwrap_or(TargetCli::Claude);

    thread::spawn(move || {
        let cleaned = fs::installer::auto_cleanup_deprecated_hooks(&source_dir, &dest_dir);

        let components = fs::scanner::scan_all_sources(&sources, &dest_dir, target_cli);
        let mcp_result = fs::scanner::scan_all_mcp_sources(&sources, target_cli);
        let plugins = fs::scanner::scan_all_plugin_sources(&sources);

        match (components, mcp_result, plugins) {
            (Ok(c), Ok((m, _mcp_warning)), Ok(p)) => {
                let _ = tx_clone.send(Ok((c, m, p, cleaned)));
            }
            (Err(e), _, _) | (_, Err(e), _) | (_, _, Err(e)) => {
                let _ = tx_clone.send(Err(e));
            }
        }
    });
}
