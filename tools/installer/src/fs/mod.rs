pub mod scanner;
pub mod diff;
pub mod installer;

use std::process::{Command, Stdio};
use std::time::Duration;
use anyhow::Result;
use wait_timeout::ChildExt;
use crate::app::TargetCli;

/// Application version string, derived from Cargo.toml at compile time.
pub const VERSION: &str = concat!("v", env!("CARGO_PKG_VERSION"));

/// Create a Command to run CLI (Claude or Codex).
///
/// Uses bare command names without file extensions. On Windows, the OS
/// resolves the extension via PATHEXT (.EXE > .CMD > .BAT), so `claude`
/// will find `claude.exe` (native install) or `claude.cmd` (npm install)
/// automatically. This avoids hardcoding `.cmd` which fails when only
/// `.exe` is present.
///
/// stdin is set to null to prevent blocking on interactive prompts.
pub fn create_cli_command(target_cli: TargetCli) -> Command {
    let cli_name = match target_cli {
        TargetCli::Claude => "claude",
        TargetCli::Codex => "codex",
    };
    let mut cmd = Command::new(cli_name);
    cmd.stdin(Stdio::null());
    cmd
}

/// Run a command with a timeout, returning its output.
/// Kills the child process if it exceeds the timeout to prevent orphans.
///
/// Note: stdin is set to null as defense-in-depth. Callers typically use
/// `create_cli_command()` which already sets null stdin, but direct callers
/// or future code paths are also protected from interactive prompt hangs.
pub(crate) fn run_with_timeout(command: &mut Command, timeout_secs: u64) -> Result<std::process::Output> {
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    match child.wait_timeout(Duration::from_secs(timeout_secs))? {
        Some(status) => {
            let mut stdout_data = Vec::new();
            let mut stderr_data = Vec::new();
            // Partial reads are acceptable: if the pipe fails after process exit,
            // we still return whatever was captured. Callers check status first.
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
            let _ = child.kill();
            let _ = child.wait();
            anyhow::bail!("Command timed out after {}s", timeout_secs);
        }
    }
}
