# Cancelable Background Processes in TUI

Production pattern for spawning a child process from a TUI while keeping the UI responsive AND letting the user cancel it (Esc / Ctrl+C without quitting the app).

This is the single most important non-obvious pattern in production TUIs. Naive `Command::output()` blocks the event loop — the user can't even press 'q'.

## The Race Condition That Bites Everyone

You spawn a child, you read a cancel signal, you kill the child. Sounds simple. The bug:

```text
T0: child finishes successfully (status=0), wrote results to disk
T1: user presses Esc (sends cancel signal)
T2: your loop checks cancel FIRST, sees the signal, kills the (already-dead) child,
    rolls back the work the user just successfully completed
```

**Rule**: in each loop iteration, check **completion first, cancel second**. A successful exit wins over a late cancel.

## Reference Implementation

```rust
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::mpsc::{Receiver, TryRecvError};
use std::thread;
use std::time::Duration;
use wait_timeout::ChildExt;

pub struct ProcessConfig {
    pub cancel_rx: Receiver<()>,
    pub poll_interval: Duration,
}

pub fn spawn_cancelable_process(
    command: &mut Command,
    config: ProcessConfig,
) -> anyhow::Result<()> {
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    // Drain stdout/stderr so the child doesn't block on a full pipe.
    // These threads exit naturally when the child closes its handles
    // (either normal exit or kill). DO NOT join them yet — see Gotchas.
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    let stdout_thread = thread::spawn(move || {
        for line in BufReader::new(stdout).lines().flatten() {
            // forward via channel if you want live output
            let _ = line;
        }
    });
    let stderr_thread = thread::spawn(move || {
        BufReader::new(stderr).lines().flatten().collect::<Vec<_>>()
    });

    loop {
        // (1) COMPLETION CHECK FIRST — wins races against late cancel.
        match child.wait_timeout(config.poll_interval)? {
            Some(status) => {
                let _ = stdout_thread.join();
                let stderr_lines = stderr_thread.join().unwrap_or_default();
                if status.success() {
                    return Ok(());
                }
                anyhow::bail!("process failed: {}", stderr_lines.join("\n"));
            }
            None => {} // still running
        }

        // (2) CANCEL CHECK SECOND.
        match config.cancel_rx.try_recv() {
            Ok(()) => {
                // Best-effort kill; child may have just exited.
                let _ = child.kill();
                let _ = child.wait();
                anyhow::bail!("cancelled by user");
            }
            Err(TryRecvError::Empty) => continue,
            Err(TryRecvError::Disconnected) => continue, // owner dropped, treat as no-cancel
        }
    }
}
```

## Wiring It Into the TUI Loop

The TUI side: own a "current cancel sender" so each spawn gets its own fresh channel. Replacing the receiver per spawn means cancellation of job N doesn't kill job N+1.

```rust
use std::sync::mpsc::{self, Sender};

pub struct Channels {
    /// Replaced on each spawn — keeps per-job isolation.
    pub current_cancel_tx: Sender<()>,
    pub processing_active: bool,
}

impl Channels {
    /// Take the receiver for a new spawn. Replaces internal tx so the
    /// next call to `cancel()` only affects the NEXT job.
    pub fn take_cancel_rx(&mut self) -> mpsc::Receiver<()> {
        let (new_tx, new_rx) = mpsc::channel();
        self.current_cancel_tx = new_tx;
        new_rx
    }

    pub fn cancel(&self) {
        let _ = self.current_cancel_tx.send(());
    }
}
```

**TUI key handler:**

```rust
match key.code {
    KeyCode::Esc | KeyCode::Char('q') if app.processing_active => {
        app.channels.cancel();
        // Don't quit the app — just cancel the in-flight job.
    }
    KeyCode::Char('q') => app.should_quit = true,
    _ => {}
}
```

## Cooperative Cancellation in Pure-Rust Tasks

Not every long task is a child process. For an in-process task (e.g., recursive scan), check the cancel receiver between work units:

```rust
fn scan_dir(root: &Path, cancel_rx: &Receiver<()>) -> anyhow::Result<Vec<PathBuf>> {
    let mut results = Vec::new();
    for entry in walkdir::WalkDir::new(root) {
        // Cooperative check between iterations.
        if cancel_rx.try_recv().is_ok() {
            anyhow::bail!("cancelled");
        }
        results.push(entry?.into_path());
    }
    Ok(results)
}
```

**Granularity rule**: check cancel at a loop boundary that runs at least once per ~100ms. More often = more responsive cancel; less often = wasted CPU on `try_recv`.

## Why This Pattern, Not `tokio::select!`

Many tutorials reach for async + `tokio::select!`. That works, but it forces every TUI to depend on tokio and rewrite all sync code. The mpsc-channel pattern above:

- works with the std blocking `Command` API (no async wrapper needed),
- composes with `event::poll(timeout)` in the TUI loop,
- is testable with a regular `mpsc::channel` in unit tests.

If the project is already async-first, `tokio::process::Command` + `tokio::select!` is fine — the principle (check completion first) still applies.
