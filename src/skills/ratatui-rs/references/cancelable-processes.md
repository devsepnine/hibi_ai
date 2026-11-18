# Cancelable Background Processes in TUI

Production pattern for spawning a child process from a TUI while keeping the UI responsive AND letting the user cancel it, Esc / Ctrl+C without quitting the app.

This is the single most important non-obvious pattern in production TUIs. Naive `Command::output()` blocks the event loop, the user can't even press 'q'.

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

Run this example on a worker. Cancellation kills and reaps the direct child. Jobs that must stop descendants need a process group or a Job Object on the target OS. Direct stderr to a temporary file from the `tempfile` crate so an inherited output handle cannot delay completion. Read the last 8 KiB available at failure. Budget disk space separately for logs written during execution.

```rust
use anyhow::{bail, Context};
use std::io::{Read, Seek, SeekFrom};
use std::process::{Command, Stdio};
use std::sync::mpsc::Receiver;
use std::time::Duration;
use tempfile::NamedTempFile;
use wait_timeout::ChildExt;

pub struct ProcessConfig {
    pub cancel_rx: Receiver<()>,
    pub poll_interval: Duration,
}

pub fn spawn_cancelable_process(
    command: &mut Command,
    config: ProcessConfig,
) -> anyhow::Result<()> {
    let mut diagnostics = NamedTempFile::new()?;
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(diagnostics.reopen()?)
        .spawn()
        .context("spawn background process")?;

    let status = loop {
        if let Some(status) = child.wait_timeout(config.poll_interval)? {
            break status;
        }
        if config.cancel_rx.try_recv().is_ok() {
            if let Some(status) = child.try_wait()? {
                break status;
            }
            if let Err(error) = child.kill() {
                if let Some(status) = child.try_wait()? {
                    break status;
                }
                return Err(error).context("stop background process");
            }
            let status = child.wait().context("reap background process")?;
            if status.success() {
                break status;
            }
            bail!("cancelled by user");
        }
    };

    if status.success() {
        return Ok(());
    }
    const TAIL_BYTES: u64 = 8192;
    let end = diagnostics.as_file().metadata()?.len();
    let start = end.saturating_sub(TAIL_BYTES);
    diagnostics.seek(SeekFrom::Start(start))?;
    let mut tail = Vec::new();
    diagnostics.take(end - start).read_to_end(&mut tail)?;
    bail!("process failed ({status}): {}", String::from_utf8_lossy(&tail))
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

Not every long task is a child process. For an in-process task such as a recursive scan, check the cancel receiver between work units:

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

- works with the std blocking `Command` API, no async wrapper needed,
- composes with `event::poll(timeout)` in the TUI loop,
- is testable with a regular `mpsc::channel` in unit tests.

If the project is already async-first, `tokio::process::Command` + `tokio::select!` is fine. The principle of checking completion first still applies.
