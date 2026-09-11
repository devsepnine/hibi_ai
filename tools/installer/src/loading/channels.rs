use std::sync::mpsc::{self, Receiver, Sender};

use anyhow::Result;

use super::scan::RefreshResult;

/// Bundles all channels and state for async process management.
pub(crate) struct ProcessingChannels {
    pub(super) process_tx: Sender<Result<String>>,
    pub(super) process_rx: Receiver<Result<String>>,
    pub(super) cancel_tx: Sender<()>,
    cancel_rx: Receiver<()>,
    pub(super) current_cancel_tx: Sender<()>,
    pub(super) processing_active: bool,
    pub(crate) refresh_tx: Sender<Result<RefreshResult>>,
    pub(crate) refresh_rx: Receiver<Result<RefreshResult>>,
    pub(super) preflight_tx: Sender<Result<()>>,
    pub(super) preflight_rx: Receiver<Result<()>>,
    pub(super) preflight_active: bool,
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
    pub(super) fn reset_preflight_channel(&mut self) {
        let (tx, rx) = mpsc::channel::<Result<()>>();
        self.preflight_tx = tx;
        self.preflight_rx = rx;
    }

    /// Replace the cancel channel pair, returning the old receiver for thread use.
    pub(super) fn take_cancel_rx(&mut self) -> Receiver<()> {
        let (new_tx, new_rx) = mpsc::channel::<()>();
        let old_rx = std::mem::replace(&mut self.cancel_rx, new_rx);
        self.cancel_tx = new_tx;
        old_rx
    }

    /// Reset the cancel channel (used after process completion).
    pub(super) fn reset_cancel_channel(&mut self) {
        let (new_tx, new_rx) = mpsc::channel::<()>();
        self.cancel_tx = new_tx;
        self.cancel_rx = new_rx;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc::TryRecvError;

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
}
