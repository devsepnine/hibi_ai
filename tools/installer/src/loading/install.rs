use std::sync::mpsc::{Receiver, TryRecvError};
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
            match result {
                Ok(msg) => app.processing_log.push(msg),
                Err(e) => {
                    let err_msg = e.to_string();
                    if err_msg == "Cancelled by user" {
                        if !app.processing_cancelled() {
                            app.processing_log.push("[WARN] Cancelled by user".into());
                        }
                        app.processing_queue.clear();
                    } else {
                        app.processing_log.push(format!("[ERR] {}", err_msg));
                    }
                }
            }
            app.cancelling = false;

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
            app.cancelling = false;
            app.processing_log
                .push("[ERR] Process thread crashed".to_string());
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
    let action = if app.is_removing {
        "Removing"
    } else {
        "Installing"
    };
    app.processing_log
        .push(format!("{} {}...", action, item_name));

    let tx_clone = channels.process_tx.clone();
    let is_removing = app.is_removing;
    let target_cli = app.target_cli.unwrap_or(TargetCli::Claude);
    let data = match process_exec::prepare(app, idx) {
        Some(d) => d,
        None => {
            channels.processing_active = false;
            app.processing_log
                .push(format!("[ERR] Invalid item index: {}", idx));
            app.processing_progress = Some(app.processing_progress.unwrap_or(0) + 1);
            if app.processing_queue.is_empty() {
                app.start_finish_processing();
            }
            return;
        }
    };

    // Update current_cancel_tx BEFORE spawning the thread
    channels.current_cancel_tx = channels.cancel_tx.clone();
    let cancel_rx_for_thread = channels.take_cancel_rx();

    thread::spawn(move || {
        let result = super::worker_result(|| {
            process_exec::execute(data, is_removing, target_cli, cancel_rx_for_thread)
        });
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
        Ok(Ok(RefreshResult::Components {
            components,
            manifest_warning,
        })) => {
            if let Some(reason) = manifest_warning {
                app.processing_log.push(manifest_warning_line(&reason));
            }
            app.apply_components_refresh(components);
        }
        Ok(Ok(RefreshResult::Mcp(servers, warning))) => {
            if let Some(warning) = warning {
                app.processing_log.push(format!("[WARN] {warning}"));
            }
            app.apply_mcp_refresh(servers);
        }
        Ok(Ok(RefreshResult::Plugins(p))) => app.apply_plugins_refresh(p),
        // InitialLoad is only sent by start_loading_thread which feeds
        // handle_loading_view, not this consumer. Treat the unexpected
        // case defensively by applying all three slices so the UI
        // doesn't end up partially fresh.
        Ok(Ok(RefreshResult::InitialLoad {
            components,
            mcp_servers,
            plugins,
            ..
        })) => {
            app.apply_components_refresh(components);
            app.mcp_servers = mcp_servers;
            app.plugins = plugins;
        }
        Ok(Err(e)) => app.finish_refresh_status(Err(e.context("Refresh failed"))),
        Err(TryRecvError::Empty) => {}
        Err(TryRecvError::Disconnected) => {
            app.finish_refresh_status(Err(anyhow::anyhow!("Refresh thread crashed")));
        }
    }
}

/// Handle a single tick of the Installing view.
pub(crate) fn handle_installing_view(
    app: &mut App,
    channels: &mut ProcessingChannels,
) -> Result<()> {
    if poll(Duration::from_millis(100))? {
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Release {
                installing_key(app, key.code, channels);
            }
        }
    }

    tick_installing(app, channels);
    Ok(())
}

pub(crate) fn tick_installing(app: &mut App, channels: &mut ProcessingChannels) {
    app.tick();

    if channels.processing_active {
        handle_process_completion(app, channels);
    }

    if !channels.processing_active && !app.processing_queue.is_empty() {
        dispatch_next_process(app, channels);
    } else if !channels.processing_active
        && app.processing_queue.is_empty()
        && app.needs_refresh
        && !app.refreshing
    {
        start_refresh_thread(app, &channels.refresh_tx);
    } else if app.refreshing {
        check_refresh_completion(app, &channels.refresh_rx);
    }
}

pub(crate) fn installing_key(app: &mut App, key: KeyCode, channels: &mut ProcessingChannels) {
    if app.processing_complete {
        if matches!(key, KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q')) {
            app.close_processing();
        }
        return;
    }
    if key != KeyCode::Esc
        || !app.can_cancel_processing()
        || (!channels.processing_active && app.processing_queue.is_empty())
    {
        return;
    }
    if !app.processing_queue.is_empty() {
        app.processing_queue.clear();
        app.processing_log.push("[WARN] Cancelled by user".into());
    }
    if channels.processing_active {
        let _ = channels.current_cancel_tx.send(());
        app.cancelling = true;
    } else {
        app.start_finish_processing();
    }
}

#[cfg(test)]
mod gui_tests {
    use super::*;
    use crate::app::View;
    use crate::app::test_support::fresh_app;

    #[test]
    fn final_summary_includes_refresh_failure_and_keeps_the_cause() {
        for disconnected in [false, true] {
            let mut app = fresh_app();
            app.processing_progress = Some(1);
            app.processing_total = Some(1);
            app.processing_log.push("[ERR] Write failed".into());
            app.start_finish_processing();
            let (tx, rx) = std::sync::mpsc::channel();
            if !disconnected {
                tx.send(Err(
                    anyhow::anyhow!("Directory missing").context("Cannot read catalog")
                ))
                .unwrap();
            }
            drop(tx);
            check_refresh_completion(&mut app, &rx);
            assert!(app.processing_complete);
            assert_eq!(
                app.processing_log.last().unwrap(),
                "[INFO] Installation finished: 1 of 1 items attempted; errors: 2"
            );
            assert_eq!(
                app.processing_log
                    .iter()
                    .filter(|line| line.starts_with("[INFO] Installation finished"))
                    .count(),
                1
            );
            let detail = if disconnected {
                "Refresh thread crashed"
            } else {
                "Cannot read catalog: Directory missing"
            };
            assert!(app.processing_log.iter().any(|line| line.contains(detail)));
        }
    }

    #[test]
    fn panicked_process_delivers_error_and_finishes_queue() {
        let mut app = fresh_app();
        app.current_view = View::Installing;
        let mut channels = ProcessingChannels::new();
        channels.processing_active = true;
        channels
            .process_tx
            .send(super::super::worker_result(|| {
                panic!("process failed");
            }))
            .unwrap();
        handle_process_completion(&mut app, &mut channels);
        assert!(!channels.processing_active);
        assert!(app.needs_refresh);
        assert!(
            app.processing_log
                .iter()
                .any(|line| line.contains("worker panicked"))
        );
    }

    #[test]
    fn invalid_last_queue_index_still_refreshes_and_settles() {
        let mut app = fresh_app();
        app.current_view = View::Installing;
        app.processing_queue = vec![99];
        let mut channels = ProcessingChannels::new();
        dispatch_next_process(&mut app, &mut channels);
        assert!(app.processing_queue.is_empty());
        assert!(!channels.processing_active);
        assert!(app.needs_refresh);
        assert!(
            app.processing_log
                .iter()
                .any(|line| line.contains("Invalid item index"))
        );
    }

    #[test]
    fn cancelling_active_process_clears_remaining_queue_and_signals_cleanup() {
        let mut app = fresh_app();
        app.current_view = View::Installing;
        app.processing_queue = vec![1, 2];
        let mut channels = ProcessingChannels::new();
        let (cancel_tx, cancel_rx) = std::sync::mpsc::channel();
        channels.current_cancel_tx = cancel_tx;
        channels.processing_active = true;
        installing_key(&mut app, KeyCode::Esc, &mut channels);
        assert!(app.processing_queue.is_empty());
        assert!(app.cancelling);
        assert!(cancel_rx.try_recv().is_ok());
        assert!(!app.processing_complete);
        assert_eq!(app.current_view, View::Installing);
        installing_key(&mut app, KeyCode::Esc, &mut channels);
        assert!(cancel_rx.try_recv().is_err());
        channels
            .process_tx
            .send(Ok("[OK] Finished current item".into()))
            .unwrap();
        handle_process_completion(&mut app, &mut channels);
        assert!(!app.cancelling);
        assert!(app.processing_cancelled());
        assert_eq!(app.processing_progress, Some(1));
        assert!(app.needs_refresh);
    }

    #[test]
    fn cancel_before_first_tick_leaves_every_item_unattempted() {
        let mut app = fresh_app();
        app.current_view = View::Installing;
        app.processing_queue = vec![0, 1];
        app.processing_progress = Some(0);
        let mut channels = ProcessingChannels::new();
        installing_key(&mut app, KeyCode::Esc, &mut channels);
        assert!(app.processing_queue.is_empty());
        assert_eq!(app.processing_progress, Some(0));
        assert!(app.needs_refresh);
        assert!(app.processing_cancelled());
    }

    #[test]
    fn last_worker_result_decides_whether_cancellation_took_effect() {
        for cancelled in [false, true] {
            let mut app = fresh_app();
            app.current_view = View::Installing;
            let mut channels = ProcessingChannels::new();
            channels.processing_active = true;
            installing_key(&mut app, KeyCode::Esc, &mut channels);
            let result = if cancelled {
                Err(anyhow::anyhow!("Cancelled by user"))
            } else {
                Ok("[OK] Installed".into())
            };
            channels.process_tx.send(result).unwrap();
            handle_process_completion(&mut app, &mut channels);
            assert_eq!(app.processing_cancelled(), cancelled);
            assert_eq!(app.processing_progress, Some(1));
            assert!(!app.cancelling);
        }
    }
}
