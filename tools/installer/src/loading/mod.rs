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
