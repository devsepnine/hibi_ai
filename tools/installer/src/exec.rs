//! Shared runner for short external commands, used by `fs` and `source`
//! (`docs/ARCHITECTURE.md` `arch-process-spawn-in-fs`). Imports nothing
//! internal so the leaf `source` module can use it.

use std::io::Read;
use std::process::{Command, Output, Stdio};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::Result;
use command_group::CommandGroup;
use wait_timeout::ChildExt;

// Read both pipes while the CLI runs. A large MCP inventory can fill one before it exits.
fn drain(mut pipe: impl Read + Send + 'static) -> Receiver<std::io::Result<Vec<u8>>> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let mut bytes = Vec::new();
        let result = pipe.read_to_end(&mut bytes).map(|_| bytes);
        let _ = tx.send(result);
    });
    rx
}

/// Run `command` and return its output, or `None` when it outlives
/// `timeout_secs`, in which case the child is killed rather than orphaned.
///
/// stdin is null so a command that prompts fails instead of hanging the TUI.
/// `on_spawn_error` turns a launch failure into the caller's message, because
/// a missing CLI and a missing `git` need different advice.
pub fn run_with_timeout(
    command: &mut Command,
    timeout_secs: u64,
    on_spawn_error: impl FnOnce(&Command, std::io::Error) -> anyhow::Error,
) -> Result<Option<Output>> {
    let spawned = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .group_spawn();
    let mut child = match spawned {
        Ok(child) => child,
        Err(err) => return Err(on_spawn_error(command, err)),
    };

    let stdout = drain(child.inner().stdout.take().expect("stdout was piped"));
    let stderr = drain(child.inner().stderr.take().expect("stderr was piped"));
    let timeout = Duration::from_secs(timeout_secs);
    let start = Instant::now();
    let capture = || -> Result<Option<Output>> {
        let Some(status) = child.inner().wait_timeout(timeout)? else {
            return Ok(None);
        };
        let read = |rx: Receiver<std::io::Result<Vec<u8>>>| -> Result<Option<Vec<u8>>> {
            match rx.recv_timeout(timeout.saturating_sub(start.elapsed())) {
                Ok(output) => Ok(Some(output?)),
                Err(RecvTimeoutError::Timeout) => Ok(None),
                Err(RecvTimeoutError::Disconnected) => anyhow::bail!("CLI output reader stopped"),
            }
        };
        let Some(stdout) = read(stdout)? else {
            return Ok(None);
        };
        let Some(stderr) = read(stderr)? else {
            return Ok(None);
        };
        Ok(Some(Output {
            status,
            stdout,
            stderr,
        }))
    };
    let output = capture();
    if !matches!(output, Ok(Some(_))) {
        let _ = child.kill();
        let _ = child.inner().wait_timeout(Duration::from_secs(1));
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_finished_command_returns_its_output() {
        let mut cmd = Command::new(std::env::current_exe().unwrap());
        cmd.arg("--list");

        let output = run_with_timeout(&mut cmd, 30, |_, e| e.into())
            .unwrap()
            .expect("the test binary lists its tests well within the timeout");

        assert!(output.status.success());
        assert!(
            String::from_utf8_lossy(&output.stdout)
                .contains("a_finished_command_returns_its_output"),
            "listing should name this test"
        );
    }

    #[cfg(unix)]
    #[test]
    fn an_inherited_output_pipe_cannot_extend_the_command_timeout() {
        let mut cmd = Command::new("sh");
        cmd.args(["-c", "sleep 15 & exit 0"]);
        let started = Instant::now();
        assert!(
            run_with_timeout(&mut cmd, 1, |_, e| e.into())
                .unwrap()
                .is_none()
        );
        assert!(started.elapsed() < Duration::from_secs(10));
    }

    /// Not a real test: the timeout test below relaunches this binary to
    /// run only this, as a child that outlives its timeout on any OS.
    #[test]
    #[ignore]
    fn sleeper_for_the_timeout_test() {
        std::thread::sleep(Duration::from_secs(30));
    }

    #[test]
    fn a_command_past_its_timeout_is_killed_and_returns_none() {
        let mut cmd = Command::new(std::env::current_exe().unwrap());
        cmd.args([
            "--ignored",
            "--exact",
            "exec::tests::sleeper_for_the_timeout_test",
        ]);

        let started = std::time::Instant::now();
        let result = run_with_timeout(&mut cmd, 1, |_, e| e.into()).unwrap();

        assert!(
            result.is_none(),
            "a child past its timeout must come back as None"
        );
        assert!(
            started.elapsed() < Duration::from_secs(10),
            "the child must be killed, not awaited"
        );
    }

    #[test]
    fn a_launch_failure_goes_through_the_callers_mapping() {
        let mut cmd = Command::new("definitely_not_a_real_program_xyz");

        let err = run_with_timeout(&mut cmd, 5, |_, e| anyhow::anyhow!("mapped: {}", e.kind()))
            .unwrap_err();

        assert_eq!(err.to_string(), "mapped: entity not found");
    }
}
