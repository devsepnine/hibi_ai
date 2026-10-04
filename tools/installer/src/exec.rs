//! Shared runner for short external commands, used by `fs` and `source`
//! (`docs/ARCHITECTURE.md` `arch-process-spawn-in-fs`). Imports nothing
//! internal so the leaf `source` module can use it.

use std::io::Read;
use std::process::{Command, Output, Stdio};
use std::time::Duration;

use anyhow::Result;
use wait_timeout::ChildExt;

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
        .spawn();
    let mut child = match spawned {
        Ok(child) => child,
        Err(err) => return Err(on_spawn_error(command, err)),
    };

    let Some(status) = child.wait_timeout(Duration::from_secs(timeout_secs))? else {
        let _ = child.kill();
        let _ = child.wait();
        return Ok(None);
    };

    // A pipe that fails after exit still yields what was captured; callers
    // decide on `status` first.
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    if let Some(mut pipe) = child.stdout.take() {
        let _ = pipe.read_to_end(&mut stdout);
    }
    if let Some(mut pipe) = child.stderr.take() {
        let _ = pipe.read_to_end(&mut stderr);
    }
    Ok(Some(Output {
        status,
        stdout,
        stderr,
    }))
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
