//! The three views that tick while a background thread works.
//!
//! Split one module per thread the view waits on — channel plumbing, scan
//! payloads, first load, install processing, preflight probe — so each tick
//! function sits next to the state it drives. Only the five items `main.rs`
//! and `cli.rs` actually call are re-exported; everything else stays
//! `pub(super)` inside this tree.

mod channels;
mod initial_load;
mod install;
mod preflight;
mod scan;

pub(crate) use channels::ProcessingChannels;
pub(crate) use initial_load::{handle_loading_view, start_loading_thread};
pub(crate) use install::handle_installing_view;
pub(crate) use preflight::handle_preflighting_view;
pub(crate) use scan::RefreshResult;

#[cfg(feature = "gui")]
pub(crate) use initial_load::tick_loading;
#[cfg(feature = "gui")]
pub(crate) use install::{installing_key, tick_installing};
#[cfg(feature = "gui")]
pub(crate) use preflight::{preflighting_key, tick_preflighting};

fn worker_result<T>(job: impl FnOnce() -> anyhow::Result<T>) -> anyhow::Result<T> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(job))
        .unwrap_or_else(|_| Err(anyhow::anyhow!("Background worker panicked")))
}

#[cfg(test)]
mod worker_tests {
    use super::worker_result;

    #[test]
    fn panic_is_a_deliverable_error_instead_of_an_empty_live_channel() {
        let (tx, rx) = std::sync::mpsc::channel();
        let retained_sender = tx.clone();
        tx.send(worker_result::<()>(|| panic!("worker failure")))
            .unwrap();
        let error = rx.try_recv().unwrap().unwrap_err();
        assert!(error.to_string().contains("worker panicked"));
        drop(retained_sender);
    }

    #[test]
    fn normal_worker_errors_keep_context() {
        let error = worker_result::<()>(|| Err(anyhow::anyhow!("Permission denied"))).unwrap_err();
        assert_eq!(error.to_string(), "Permission denied");
    }
}
