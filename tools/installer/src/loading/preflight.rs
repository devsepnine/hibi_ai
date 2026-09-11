use std::sync::mpsc::{Sender, TryRecvError};
use std::thread;
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, poll};

use crate::app::{App, TargetCli, View};
use crate::fs;

use super::channels::ProcessingChannels;

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
