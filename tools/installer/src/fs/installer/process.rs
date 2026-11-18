use anyhow::Result;
use command_group::{CommandGroup, GroupChild};
use std::io::{BufRead, BufReader, Read};
use std::process::{Command, Stdio};
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::{Duration, Instant};
use wait_timeout::ChildExt;

/// Maximum stderr capture size (1 MB) to prevent memory exhaustion.
pub(super) const MAX_STDERR_BYTES: usize = 1_024 * 1_024;

/// Default timeout for quick, non-cancelable operations (e.g., marketplace list).
pub(super) const QUICK_COMMAND_TIMEOUT_SECS: u64 = 15;

/// Wait time after killing a child process (milliseconds).
const KILL_WAIT_MS: u64 = 500;

/// Polling interval for process completion checks (milliseconds).
const POLL_INTERVAL_MS: u64 = 100;

fn stop_group(child: &mut GroupChild) {
    let _ = child.kill();
    let _ = child
        .inner()
        .wait_timeout(Duration::from_millis(KILL_WAIT_MS));
}

/// Format a process failure error message from stderr output.
fn format_process_error(action: &str, item_name: &str, stderr_output: &str) -> String {
    let summary = format!("Failed to {action} {item_name}");
    let detail = stderr_output.trim();
    if detail.is_empty() {
        return summary;
    }
    // Codex includes raw TOML source lines in its configuration error chain.
    if detail.starts_with("Error: failed to load bootstrap configuration")
        || detail.starts_with("Error: failed to load MCP servers")
        || detail.contains("config.toml:")
        || detail.contains("TOML parse error")
    {
        return format!("{summary}. Check the Codex config.toml with `codex mcp list`.");
    }
    format!("{summary}: {detail}")
}

use crate::fs::enrich_spawn_error;
/// Re-export the shared `run_with_timeout` for use within the installer module.
pub(super) use crate::fs::run_with_timeout;

/// Configuration for a cancelable process operation.
pub(super) struct ProcessConfig<'a> {
    pub timeout_secs: u64,
    pub cancel_rx: &'a Receiver<()>,
    pub item_name: &'a str,
    pub action: &'a str,
}

/// Spawn a process with timeout and cancellation support.
///
/// Captures stdout/stderr in background threads to prevent pipe blocking.
/// Stderr is capped at `MAX_STDERR_BYTES` to prevent memory exhaustion.
pub(super) fn spawn_cancelable_process(command: &mut Command, config: ProcessConfig) -> Result<()> {
    run_until_output(command, config, None).map(|_| ())
}

#[derive(PartialEq, Debug)]
pub(super) enum ProcessExit {
    Finished,
    MatchedOutput,
}

/// Some CLIs persist a registration before entering interactive login.
pub(super) fn run_until_output(
    command: &mut Command,
    config: ProcessConfig,
    marker: Option<&'static str>,
) -> Result<ProcessExit> {
    if config.cancel_rx.try_recv().is_ok() {
        anyhow::bail!("Cancelled by user");
    }

    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .group_spawn()
        .map_err(|e| enrich_spawn_error(command, e))?;

    let stdout = child.inner().stdout.take().unwrap();
    let stderr = child.inner().stderr.take().unwrap();

    let (matched_tx, matched_rx) = std::sync::mpsc::channel();
    thread::spawn(move || {
        let mut reader = BufReader::new(stdout);
        if let Some(prefix) = marker {
            for line in reader.by_ref().lines().map_while(Result::ok) {
                if line.starts_with(prefix) {
                    let _ = matched_tx.send(());
                    break;
                }
            }
        }
        let _ = std::io::copy(&mut reader, &mut std::io::sink());
    });

    let (stderr_tx, stderr_rx) = std::sync::mpsc::channel();
    thread::spawn(move || {
        let mut reader = BufReader::new(stderr);
        let mut bytes = Vec::new();
        let _ = reader
            .by_ref()
            .take(MAX_STDERR_BYTES as u64 + 1)
            .read_to_end(&mut bytes);
        let truncated = bytes.len() > MAX_STDERR_BYTES;
        bytes.truncate(MAX_STDERR_BYTES);
        let mut message = String::from_utf8_lossy(&bytes).into_owned();
        if truncated {
            message.push_str("\n[... output truncated]");
        }
        let _ = stderr_tx.send(message);
        let _ = std::io::copy(&mut reader, &mut std::io::sink());
    });

    let start = Instant::now();
    let poll = Duration::from_millis(POLL_INTERVAL_MS);
    loop {
        // Allow an immediate OAuth error to finish before interrupting interactive login.
        let matched = matched_rx.try_recv().is_ok();
        let status = if matched {
            child.inner().wait_timeout(poll)
        } else {
            child.inner().try_wait()
        };
        let status = status.inspect_err(|_| stop_group(&mut child))?;
        if let Some(status) = status {
            // A descendant can keep output pipes open after the CLI has finished.
            stop_group(&mut child);
            if status.success() {
                return Ok(ProcessExit::Finished);
            }
            let error = stderr_rx
                .recv_timeout(Duration::from_millis(KILL_WAIT_MS))
                .unwrap_or_default();
            anyhow::bail!(format_process_error(
                config.action,
                config.item_name,
                &error
            ));
        }
        if config.cancel_rx.try_recv().is_ok() {
            stop_group(&mut child);
            anyhow::bail!("Cancelled by user");
        }
        if start.elapsed() >= Duration::from_secs(config.timeout_secs) {
            stop_group(&mut child);
            anyhow::bail!(
                "Timed out after {}s trying to {} {}",
                config.timeout_secs,
                config.action,
                config.item_name
            );
        }
        if matched {
            stop_group(&mut child);
            return Ok(ProcessExit::MatchedOutput);
        }
        thread::sleep(poll);
    }
}

#[cfg(all(test, unix))]
mod registration_tests {
    use super::*;

    #[test]
    fn queued_cancel_skips_registration() {
        let (tx, rx) = std::sync::mpsc::channel();
        tx.send(()).unwrap();
        let err = run_until_output(
            &mut Command::new("hibi-nonexistent-registration-cli"),
            ProcessConfig {
                timeout_secs: 5,
                cancel_rx: &rx,
                item_name: "fixture",
                action: "register",
            },
            None,
        )
        .unwrap_err();
        assert_eq!(err.to_string(), "Cancelled by user");
    }

    fn run_script(script: &str, cancel: &Receiver<()>) -> Result<ProcessExit> {
        run_until_output(
            Command::new("sh").args(["-c", script]),
            ProcessConfig {
                timeout_secs: 4,
                cancel_rx: cancel,
                item_name: "fixture",
                action: "register",
            },
            Some("login pending"),
        )
    }

    #[test]
    fn config_parse_failure_omits_the_quoted_config_line() {
        let detail = "Error: failed to load MCP servers from fixture\nCaused by: TOML parse error at line 2\n2 | key = synthetic-value";
        let error = format_process_error("remove MCP server", "fixture", detail);
        assert!(error.contains("codex mcp list"));
        assert!(!error.contains("synthetic-value"));
    }

    #[test]
    fn exited_cli_status_takes_precedence_over_login_output() {
        let (_tx, rx) = std::sync::mpsc::channel();
        let err =
            run_script("echo 'login pending'; echo 'login failed' >&2; exit 1", &rx).unwrap_err();
        assert!(err.to_string().contains("login failed"));
        assert_eq!(
            run_script("echo 'login pending'; exit 0", &rx).unwrap(),
            ProcessExit::Finished
        );
    }

    #[test]
    fn successful_exit_finishes_even_if_a_descendant_holds_the_pipes() {
        let (_tx, rx) = std::sync::mpsc::channel();
        assert_eq!(
            run_script("sleep 8 & exit 0", &rx).unwrap(),
            ProcessExit::Finished
        );
    }

    #[test]
    fn interactive_login_can_resume_after_registration_without_rollback() {
        let (_, rx) = std::sync::mpsc::channel();
        let dir = tempfile::tempdir().unwrap();
        let sentinel = dir.path().join("orphan");
        let mut child = Command::new("sh");
        child
            .args([
                "-c",
                r#"(sleep 1; touch "$1") & echo 'login pending'; wait"#,
                "wrapper",
            ])
            .arg(&sentinel);
        let result = run_until_output(
            &mut child,
            ProcessConfig {
                timeout_secs: 5,
                cancel_rx: &rx,
                item_name: "fixture",
                action: "register",
            },
            Some("login pending"),
        );
        assert_eq!(result.unwrap(), ProcessExit::MatchedOutput);
        thread::sleep(Duration::from_millis(1100));
        assert!(!sentinel.exists());
    }
}
