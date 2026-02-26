use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use std::thread;
use std::io::{BufRead, BufReader};
use std::sync::mpsc::Receiver;
use anyhow::Result;
use wait_timeout::ChildExt;

/// Maximum stderr capture size (1 MB) to prevent memory exhaustion.
pub(super) const MAX_STDERR_BYTES: usize = 1_024 * 1_024;

/// Default timeout for quick, non-cancelable operations (e.g., marketplace list).
pub(super) const QUICK_COMMAND_TIMEOUT_SECS: u64 = 15;

/// Timeout for cleanup commands (seconds).
const CLEANUP_TIMEOUT_SECS: u64 = 10;

/// Wait time after killing a child process (milliseconds).
const KILL_WAIT_MS: u64 = 500;

/// Polling interval for process completion checks (milliseconds).
const POLL_INTERVAL_MS: u64 = 100;

/// Run a cleanup command with a 10-second timeout.
/// Kills the child process if it exceeds the timeout to prevent orphans.
pub(super) fn run_cleanup_command(command: &mut Command) -> bool {
    let result = command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .and_then(|mut child| {
            child.wait_timeout(Duration::from_secs(CLEANUP_TIMEOUT_SECS))
                .map(|status| match status {
                    Some(s) => s.success(),
                    None => {
                        // Timeout: kill to prevent orphaned process
                        let _ = child.kill();
                        false
                    }
                })
        });

    result.unwrap_or(false)
}

/// Kill a child process and wait briefly for it to terminate.
fn kill_and_cleanup(
    child: &mut std::process::Child,
    cleanup: Option<Box<dyn FnOnce() -> bool>>,
) {
    let _ = child.kill();
    let _ = child.wait_timeout(Duration::from_millis(KILL_WAIT_MS));
    if let Some(do_cleanup) = cleanup {
        // Cleanup failure is communicated via "cleanup may be incomplete" in the bail! message.
        // No eprintln! here -- it would corrupt the TUI display.
        let _ = do_cleanup();
    }
}

/// Format a process failure error message from stderr output.
fn format_process_error(action: &str, item_name: &str, stderr_output: &str) -> String {
    if stderr_output.trim().is_empty() {
        format!("Failed to {} {}", action, item_name)
    } else {
        format!("Failed to {} {}: {}", action, item_name, stderr_output.trim())
    }
}

/// Run a command with a timeout, returning its output.
/// For quick, non-cancelable operations like checking marketplace status.
pub(super) fn run_with_timeout(command: &mut Command, timeout_secs: u64) -> Result<std::process::Output> {
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    match child.wait_timeout(Duration::from_secs(timeout_secs))? {
        Some(status) => {
            // Process exited; collect remaining pipe data
            let mut stdout_data = Vec::new();
            let mut stderr_data = Vec::new();
            if let Some(mut stdout) = child.stdout.take() {
                let _ = std::io::Read::read_to_end(&mut stdout, &mut stdout_data);
            }
            if let Some(mut stderr) = child.stderr.take() {
                let _ = std::io::Read::read_to_end(&mut stderr, &mut stderr_data);
            }

            Ok(std::process::Output {
                status,
                stdout: stdout_data,
                stderr: stderr_data,
            })
        }
        None => {
            // Timeout: kill to prevent orphaned process
            let _ = child.kill();
            let _ = child.wait();
            anyhow::bail!("Command timed out after {}s", timeout_secs);
        }
    }
}

/// Configuration for a cancelable process operation.
pub(super) struct ProcessConfig<'a> {
    pub timeout_secs: u64,
    pub cancel_rx: &'a Receiver<()>,
    pub item_name: &'a str,
    pub action: &'a str,
    pub cleanup: Option<Box<dyn FnOnce() -> bool>>,
}

/// Spawn a process with timeout and cancellation support.
///
/// Captures stdout/stderr in background threads to prevent pipe blocking.
/// Stderr is capped at `MAX_STDERR_BYTES` to prevent memory exhaustion.
/// If `cleanup` is provided, it will be called on timeout or cancellation.
pub(super) fn spawn_cancelable_process(
    command: &mut Command,
    config: ProcessConfig,
) -> Result<()> {
    let has_cleanup = config.cleanup.is_some();
    let mut cleanup_slot = config.cleanup;

    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let stdout_thread = thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            let _ = line;
        }
    });

    let stderr_thread = thread::spawn(move || {
        let reader = BufReader::new(stderr);
        let mut output = String::new();
        for line in reader.lines() {
            if let Ok(line) = line {
                if output.len() + line.len() + 1 > MAX_STDERR_BYTES {
                    output.push_str("\n[... output truncated]");
                    break;
                }
                output.push_str(&line);
                output.push('\n');
            }
        }
        output
    });

    let timeout_duration = Duration::from_secs(config.timeout_secs);
    let start_time = Instant::now();

    loop {
        if start_time.elapsed() >= timeout_duration {
            kill_and_cleanup(&mut child, cleanup_slot.take());
            // Threads exit naturally when pipes close after kill; no join needed on abort paths.
            if has_cleanup {
                anyhow::bail!("Installation timed out after {}s (cleanup may be incomplete)", config.timeout_secs);
            } else {
                anyhow::bail!("Removal timed out after {}s", config.timeout_secs);
            }
        }

        // Check process completion BEFORE cancel signal.
        // This prevents silently rolling back a successful operation when
        // a cancel signal arrives in the same tick as process completion.
        match child.wait_timeout(Duration::from_millis(POLL_INTERVAL_MS)) {
            Ok(Some(status)) => {
                let _ = stdout_thread.join();
                let stderr_output = stderr_thread.join().unwrap_or_default();

                if !status.success() {
                    anyhow::bail!(format_process_error(config.action, config.item_name, &stderr_output));
                }
                return Ok(());
            }
            Ok(None) => {}
            Err(e) => {
                let _ = child.kill();
                anyhow::bail!("System error during wait: {}", e);
            }
        }

        match config.cancel_rx.try_recv() {
            Ok(_) => {
                kill_and_cleanup(&mut child, cleanup_slot.take());
                if has_cleanup {
                    anyhow::bail!("Cancelled by user (cleanup may be incomplete)");
                } else {
                    anyhow::bail!("Cancelled by user");
                }
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {}
            Err(std::sync::mpsc::TryRecvError::Empty) => {}
        }
    }
}
