use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::thread;
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, poll};

use crate::app::{App, Tab, TargetCli, View};
use crate::component;
use crate::fs;
use crate::mcp;
use crate::plugin;
use crate::process_exec;

/// Outcome of a scan thread, scoped to the kind of data it actually
/// touched. Used to skip the expensive `claude/codex mcp list` shell
/// call (which dominates refresh wall time) when an install only
/// affected filesystem-backed components or plugin metadata.
pub(crate) enum RefreshResult {
    /// First-time scan after CLI selection. Returns the full picture so
    /// every tab has something to render, and reports any deprecated
    /// hooks that were silently cleaned up during the load.
    InitialLoad {
        components: Vec<component::Component>,
        mcp_servers: Vec<mcp::McpServer>,
        plugins: Vec<plugin::Plugin>,
        cleaned_hooks: Vec<String>,
    },
    /// Refresh limited to filesystem-backed component types — Agents,
    /// Commands, Contexts, Rules, Skills, Hooks, Styles, Statusline,
    /// Config. Pure filesystem ops, sub-100ms typical.
    Components(Vec<component::Component>),
    /// Refresh limited to MCP servers. Always shells out to the CLI
    /// (`mcp list`) and is the slow path — only run when the user
    /// actually installed/removed an MCP server.
    Mcp(Vec<mcp::McpServer>),
    /// Refresh limited to plugins (filesystem only).
    Plugins(Vec<plugin::Plugin>),
}

/// Which scan to run for a post-install refresh, derived from the tab
/// that initiated the install/remove. Kept private to this module so
/// `RefreshResult` remains the single public boundary between scan
/// threads and the app state.
enum RefreshScope {
    Components,
    Mcp,
    Plugins,
}

impl RefreshScope {
    fn for_tab(tab: Tab) -> Self {
        match tab {
            Tab::McpServers => Self::Mcp,
            Tab::Plugins => Self::Plugins,
            _ => Self::Components,
        }
    }
}

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
    pub(crate) preflight_tx: Sender<Result<()>>,
    pub(crate) preflight_rx: Receiver<Result<()>>,
    pub(crate) preflight_active: bool,
}

impl ProcessingChannels {
    pub(crate) fn new() -> Self {
        let (process_tx, process_rx) = mpsc::channel::<Result<String>>();
        let (cancel_tx, cancel_rx) = mpsc::channel::<()>();
        let current_cancel_tx = cancel_tx.clone();
        let (refresh_tx, refresh_rx) = mpsc::channel::<Result<RefreshResult>>();
        let (preflight_tx, preflight_rx) = mpsc::channel::<Result<()>>();

        Self {
            process_tx,
            process_rx,
            cancel_tx,
            cancel_rx,
            current_cancel_tx,
            processing_active: false,
            refresh_tx,
            refresh_rx,
            preflight_tx,
            preflight_rx,
            preflight_active: false,
        }
    }

    /// Replace the preflight channel pair with a fresh one. Called before
    /// each new preflight so any stale send from a previously-cancelled
    /// thread cannot bleed into the next attempt's result.
    pub(crate) fn reset_preflight_channel(&mut self) {
        let (tx, rx) = mpsc::channel::<Result<()>>();
        self.preflight_tx = tx;
        self.preflight_rx = rx;
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

/// Start a background thread to rescan only the data the just-finished
/// install/remove could have changed.
///
/// Before this was scope-aware, every refresh re-ran all three scans
/// including `claude/codex mcp list` — a CLI shell-out with up to a
/// 30-second timeout — even when the user installed e.g. an Agent
/// (filesystem-only). Now we look at `app.tab` to pick the single scan
/// that's actually relevant, dropping non-MCP refresh latency from
/// "several seconds" to "tens of milliseconds".
fn start_refresh_thread(app: &mut App, refresh_tx: &Sender<Result<RefreshResult>>) {
    app.refreshing = true;

    let tx_clone = refresh_tx.clone();
    let sources = app.sources.clone();
    let dest_dir = app.dest_dir.clone();
    let target_cli = app.target_cli.unwrap_or(TargetCli::Claude);
    let scope = RefreshScope::for_tab(app.tab);

    thread::spawn(move || {
        let result = match scope {
            RefreshScope::Components => fs::scanner::scan_all_sources(&sources, &dest_dir, target_cli)
                .map(RefreshResult::Components),
            RefreshScope::Mcp => fs::scanner::scan_all_mcp_sources(&sources, target_cli)
                .map(|(servers, _warning)| RefreshResult::Mcp(servers)),
            RefreshScope::Plugins => fs::scanner::scan_all_plugin_sources(&sources)
                .map(RefreshResult::Plugins),
        };
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
        Ok(Ok(RefreshResult::Components(c))) => app.apply_components_refresh(c),
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
        Ok(Ok(RefreshResult::InitialLoad { components, mcp_servers, plugins, cleaned_hooks })) => {
            app.finish_loading(components, mcp_servers, plugins, cleaned_hooks);
        }
        // The refresh channel is shared with start_refresh_thread, but
        // that thread only runs from the Installing view; the Loading
        // view should never see a Components/Mcp/Plugins variant. If
        // somehow one arrives, bail back to CLI selection rather than
        // half-populating the screen.
        Ok(Ok(_unexpected)) => {
            app.status_message = Some("Unexpected refresh payload during load".to_string());
            app.current_view = View::CliSelection;
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

/// Spawn the CLI `--version` probe on a background thread.
///
/// Hoists the blocking `preflight_cli_available` off the TUI tick so input
/// and rendering stay responsive during the 8-second budget. The result
/// is delivered to `preflight_tx`; `handle_preflighting_view` polls the
/// matching receiver each tick.
fn start_preflight_thread(
    app: &App,
    preflight_tx: &Sender<Result<()>>,
) {
    let tx = preflight_tx.clone();
    let target_cli = app.target_cli.unwrap_or(TargetCli::Claude);
    thread::spawn(move || {
        let result = fs::installer::preflight_cli_available(target_cli);
        let _ = tx.send(result);
    });
}

/// Handle a single tick of the Preflighting view.
///
/// Spawns the background CLI probe on first entry (idempotent via
/// `preflight_active`), then polls the channel each tick. On success,
/// hands off to `complete_install_setup` / `complete_remove_setup`
/// which continue the original synchronous flow (MCP env var prompt
/// or direct transition to Installing). On failure, sets a status
/// message and returns to the List view. Esc cancels: the channel is
/// reset so the in-flight thread's result is discarded.
pub(crate) fn handle_preflighting_view(
    app: &mut App,
    channels: &mut ProcessingChannels,
) -> Result<()> {
    // Idempotent spawn: keeps the main loop's match arm a one-liner
    // and ensures the probe always starts in the same place that
    // checks for its completion.
    if !channels.preflight_active {
        channels.reset_preflight_channel();
        start_preflight_thread(app, &channels.preflight_tx);
        channels.preflight_active = true;
    }

    if poll(Duration::from_millis(100))? {
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Release {
                handle_preflighting_input(app, key.code, channels);
            }
        }
    }

    app.tick();

    if !channels.preflight_active {
        // User cancelled this tick; the channel was already reset.
        return Ok(());
    }

    match channels.preflight_rx.try_recv() {
        Ok(Ok(())) => {
            channels.preflight_active = false;
            if app.is_removing {
                app.complete_remove_setup();
            } else {
                app.complete_install_setup()?;
            }
        }
        Ok(Err(e)) => {
            channels.preflight_active = false;
            let verb = if app.is_removing { "removal" } else { "install" };
            app.status_message = Some(format!("Cannot start {}: {}", verb, e));
            app.processing_queue.clear();
            app.is_removing = false;
            app.current_view = View::List;
        }
        Err(TryRecvError::Empty) => {}
        Err(TryRecvError::Disconnected) => {
            channels.preflight_active = false;
            app.status_message = Some("Preflight thread crashed".to_string());
            app.processing_queue.clear();
            app.is_removing = false;
            app.current_view = View::List;
        }
    }

    Ok(())
}

fn handle_preflighting_input(
    app: &mut App,
    key: KeyCode,
    channels: &mut ProcessingChannels,
) {
    match key {
        KeyCode::Esc => {
            // Drop the in-flight thread's result by resetting the channel.
            // The thread will finish and try to send on a closed channel;
            // its send returns Err and is discarded.
            channels.preflight_active = false;
            channels.reset_preflight_channel();
            app.processing_queue.clear();
            app.is_removing = false;
            app.cancelling = false;
            app.status_message = Some("Cancelled".to_string());
            app.current_view = View::List;
        }
        KeyCode::Char('q') => {
            app.should_quit = true;
        }
        _ => {}
    }
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

        let result = match (components, mcp_result, plugins) {
            (Ok(c), Ok((m, _mcp_warning)), Ok(p)) => Ok(RefreshResult::InitialLoad {
                components: c,
                mcp_servers: m,
                plugins: p,
                cleaned_hooks: cleaned,
            }),
            (Err(e), _, _) | (_, Err(e), _) | (_, _, Err(e)) => Err(e),
        };
        let _ = tx_clone.send(result);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reset_preflight_channel_discards_pending_sends() {
        // Simulate the cancel path: a thread sent a result, then the
        // user cancelled and the channel was reset. The next preflight
        // must see an empty channel, not the stale Ok from the previous
        // attempt — otherwise pressing 'i' twice would skip the probe.
        let mut channels = ProcessingChannels::new();
        let stale_tx = channels.preflight_tx.clone();
        let _ = stale_tx.send(Ok(()));

        channels.reset_preflight_channel();

        match channels.preflight_rx.try_recv() {
            Err(TryRecvError::Empty) => {}
            other => panic!("expected Empty after reset, got {:?}", other.map(|_| "Ok(...)")),
        }
    }

    #[test]
    fn reset_preflight_channel_keeps_new_sender_paired() {
        // After reset, sends on the new sender must reach the new receiver.
        // Confirms reset doesn't leave the struct in a half-wired state.
        let mut channels = ProcessingChannels::new();
        channels.reset_preflight_channel();

        let fresh_tx = channels.preflight_tx.clone();
        fresh_tx.send(Ok(())).expect("send on fresh channel");

        match channels.preflight_rx.try_recv() {
            Ok(Ok(())) => {}
            other => panic!("expected Ok(Ok(())), got {:?}", other.map(|_| "?")),
        }
    }

    #[test]
    fn refresh_scope_routes_by_tab() {
        // Locks in the optimization: only MCP/Plugin tabs trigger the
        // matching scan. Every component tab routes to the cheap
        // filesystem-only Components scan, skipping `mcp list` entirely.
        assert!(matches!(RefreshScope::for_tab(Tab::McpServers), RefreshScope::Mcp));
        assert!(matches!(RefreshScope::for_tab(Tab::Plugins), RefreshScope::Plugins));
        for tab in [
            Tab::Agents, Tab::Commands, Tab::Contexts, Tab::Rules, Tab::Skills,
            Tab::Hooks, Tab::OutputStyles, Tab::Statusline, Tab::Config,
        ] {
            assert!(
                matches!(RefreshScope::for_tab(tab), RefreshScope::Components),
                "tab {:?} should route to Components scope",
                tab,
            );
        }
    }
}
