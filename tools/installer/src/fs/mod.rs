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
/// On Unix, the bare name is resolved through PATH by execvp.
///
/// On Windows, Rust's `Command::new` invokes `CreateProcessW`, which
/// only auto-appends `.exe` when no extension is given — it does NOT
/// honor PATHEXT to also try `.cmd` / `.bat` / `.ps1`. npm-installed
/// CLIs ship as `.cmd` shims (e.g. `%APPDATA%\npm\claude.cmd`), so the
/// bare name silently fails to resolve them and `mcp list` returns no
/// servers, making every MCP look "Not Installed". Probe PATH manually
/// for `.exe` (native install) before `.cmd` / `.bat` (npm shim) so
/// native installs win when both formats are present, and npm installs
/// still resolve when only the shim exists.
///
/// stdin is set to null to prevent blocking on interactive prompts.
pub fn create_cli_command(target_cli: TargetCli) -> Command {
    let cli_name = match target_cli {
        TargetCli::Claude => "claude",
        TargetCli::Codex => "codex",
    };
    let mut cmd = Command::new(resolve_cli_program(cli_name));
    cmd.stdin(Stdio::null());
    cmd
}

/// Resolve the OS-appropriate program name for a CLI.
///
/// On Windows, returns the absolute path to the first existing file
/// matching `<cli_name>.exe`, `<cli_name>.cmd`, or `<cli_name>.bat`
/// in any PATH directory. Returns the bare name when no candidate is
/// found, preserving the original "spawn fails with not-found" error
/// path so callers can surface a clear "CLI not found in PATH" hint.
///
/// On non-Windows targets, returns the bare name unchanged.
fn resolve_cli_program(cli_name: &str) -> std::ffi::OsString {
    #[cfg(windows)]
    {
        if let Some(path) = std::env::var_os("PATH") {
            // Per-directory: prefer .exe (matches CreateProcess default
            // when an extension is absent) before .cmd / .bat (npm-shim
            // style). This keeps native installs winning when both
            // formats exist in the same directory.
            for dir in std::env::split_paths(&path) {
                for ext in [".exe", ".cmd", ".bat"] {
                    let candidate = dir.join(format!("{}{}", cli_name, ext));
                    if candidate.is_file() {
                        return candidate.into_os_string();
                    }
                }
            }
        }
        cli_name.into()
    }
    #[cfg(not(windows))]
    {
        cli_name.into()
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(not(windows))]
    fn unix_resolve_returns_bare_name() {
        let resolved = resolve_cli_program("claude");
        assert_eq!(resolved, std::ffi::OsString::from("claude"));
    }

    #[test]
    #[cfg(windows)]
    fn windows_resolve_falls_back_to_bare_name_when_missing() {
        // A name very unlikely to exist anywhere in PATH.
        let unlikely = "hibi_test_nonexistent_cli_xyz";
        let resolved = resolve_cli_program(unlikely);
        assert_eq!(resolved, std::ffi::OsString::from(unlikely));
    }

    #[test]
    #[cfg(windows)]
    fn windows_resolve_finds_shim_in_path() {
        // Write a fake `.cmd` shim into a temp dir, point PATH at it,
        // and verify resolution returns the absolute path.
        let dir = std::env::temp_dir().join("hibi_resolve_test");
        let _ = std::fs::create_dir_all(&dir);
        let shim = dir.join("hibitestshim.cmd");
        std::fs::write(&shim, b"@echo off\r\n").unwrap();

        let prev_path = std::env::var_os("PATH").unwrap_or_default();
        let mut paths: Vec<std::path::PathBuf> = std::env::split_paths(&prev_path).collect();
        paths.insert(0, dir.clone());
        let new_path = std::env::join_paths(paths).unwrap();
        // Safety: tests are single-threaded inside this module by default;
        // restore PATH after the assertion.
        unsafe { std::env::set_var("PATH", &new_path); }

        let resolved = resolve_cli_program("hibitestshim");

        unsafe { std::env::set_var("PATH", &prev_path); }
        let _ = std::fs::remove_file(&shim);

        assert_eq!(resolved, shim.into_os_string());
    }
}
