use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::thread;
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, poll};

use crate::app::{App, TargetCli, View};
use crate::fs;

use super::scan::RefreshResult;

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
