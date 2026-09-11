use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::thread;
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, poll};

use crate::app::{App, TargetCli};
use crate::process_exec;

use super::channels::ProcessingChannels;
use super::scan::{RefreshResult, manifest_warning_line, start_refresh_thread};

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

/// Check if a refresh thread has completed and apply results.
///
/// Dispatches by `RefreshResult` variant so that, e.g., a Components
/// refresh only swaps `app.components` and rebuilds the affected tree
/// views — `app.mcp_servers` and `app.plugins` are left untouched.
fn check_refresh_completion(app: &mut App, refresh_rx: &Receiver<Result<RefreshResult>>) {
    match refresh_rx.try_recv() {
        Ok(Ok(RefreshResult::Components { components, manifest_warning })) => {
            if let Some(reason) = manifest_warning {
                app.processing_log.push(manifest_warning_line(&reason));
            }
            app.apply_components_refresh(components);
        }
        Ok(Ok(RefreshResult::Mcp(m))) => app.apply_mcp_refresh(m),
        Ok(Ok(RefreshResult::Plugins(p))) => app.apply_plugins_refresh(p),
        // InitialLoad is only sent by start_loading_thread which feeds
        // handle_loading_view, not this consumer. Treat the unexpected
        // case defensively by applying all three slices so the UI
        // doesn't end up partially fresh.
        Ok(Ok(RefreshResult::InitialLoad { components, mcp_servers, plugins, .. })) => {
            app.apply_components_refresh(components);
            app.mcp_servers = mcp_servers;
            app.plugins = plugins;
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
