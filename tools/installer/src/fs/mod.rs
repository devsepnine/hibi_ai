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
/// On Unix, first walks `PATH` (mirroring `execvp`), then probes a list
/// of well-known install locations whose presence is independent of the
/// shell rc state — Claude Code's official `install.sh` writes the binary
/// to `~/.claude/local/bin` but only adds it to `PATH` via `.zshrc`, so a
/// terminal that did not source the rc file misses it. Homebrew /
/// npm-global / volta-style locations are probed for the same reason.
/// Returns the bare name when nothing is found, so the caller's
/// `enrich_spawn_error` path still surfaces the "not in PATH" message.
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
        // 1) PATH walk — same lookup execvp would perform, done up front
        //    so we know whether to attempt the fallback.
        if let Some(path) = std::env::var_os("PATH") {
            for dir in std::env::split_paths(&path) {
                let candidate = dir.join(cli_name);
                if candidate.is_file() {
                    return candidate.into_os_string();
                }
            }
        }

        // 2) Known install locations. Order is "most likely first" so a
        //    real binary wins before a stale homebrew leftover. Each entry
        //    only enters the probe if HOME is set / the path is valid.
        let home = std::env::var_os("HOME").map(std::path::PathBuf::from);
        let fallbacks: [Option<std::path::PathBuf>; 6] = [
            home.as_ref().map(|h| h.join(".claude/local/bin").join(cli_name)),
            Some(std::path::PathBuf::from("/opt/homebrew/bin").join(cli_name)),
            Some(std::path::PathBuf::from("/usr/local/bin").join(cli_name)),
            home.as_ref().map(|h| h.join(".npm-global/bin").join(cli_name)),
            home.as_ref().map(|h| h.join(".local/bin").join(cli_name)),
            home.as_ref().map(|h| h.join(".bun/bin").join(cli_name)),
        ];
        for candidate in fallbacks.into_iter().flatten() {
            if candidate.is_file() {
                return candidate.into_os_string();
            }
        }
        cli_name.into()
    }
}

/// Public docs URL surfaced in ENOENT hints. Centralised so a future
/// rename of the upstream doc site only touches one line.
const CLI_INSTALL_DOCS_URL: &str = "https://docs.claude.com/en/docs/claude-code/setup";

/// Convert a spawn error into a message users can act on.
///
/// Rust surfaces a missing PATH binary as `No such file or directory (os error 2)`,
/// which gives no clue that the actual problem is a missing CLI. When the spawn
/// error is `ErrorKind::NotFound`, attach the program name and a `type -a` hint
/// as the *outer* context while keeping the original `io::Error` in the chain.
/// Callers that just `Display` the error see the helpful message; future
/// callers that want to branch on `ErrorKind` can still `downcast_ref`.
pub(crate) fn enrich_spawn_error(command: &Command, err: std::io::Error) -> anyhow::Error {
    if err.kind() == std::io::ErrorKind::NotFound {
        let program = command.get_program();
        let name = std::path::Path::new(program)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("CLI")
            .to_string();
        anyhow::Error::from(err).context(format!(
            "'{name}' CLI not found in PATH. Install it (e.g. {CLI_INSTALL_DOCS_URL}) and verify with `type -a {name}` from the same terminal before retrying."
        ))
    } else {
        anyhow::Error::from(err).context("Failed to launch process")
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
        .spawn()
        .map_err(|e| enrich_spawn_error(command, e))?;

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
    fn unix_resolve_falls_back_to_bare_name_when_missing() {
        // Name that cannot exist in PATH or any known install location.
        let unlikely = "hibi_test_nonexistent_cli_xyz";
        let resolved = resolve_cli_program(unlikely);
        assert_eq!(resolved, std::ffi::OsString::from(unlikely));
    }

    #[test]
    #[cfg(not(windows))]
    fn unix_resolve_finds_binary_in_path() {
        // Write a fake executable into a temp dir, prepend PATH, and
        // verify resolution returns its absolute path (PATH walk wins
        // before the known-location fallback list).
        use std::os::unix::fs::PermissionsExt;
        let dir = std::env::temp_dir().join("hibi_resolve_unix_test");
        let _ = std::fs::create_dir_all(&dir);
        let bin = dir.join("hibitestbin");
        std::fs::write(&bin, b"#!/bin/sh\nexit 0\n").unwrap();
        std::fs::set_permissions(&bin, std::fs::Permissions::from_mode(0o755)).unwrap();

        let prev_path = std::env::var_os("PATH").unwrap_or_default();
        let mut paths: Vec<std::path::PathBuf> = std::env::split_paths(&prev_path).collect();
        paths.insert(0, dir.clone());
        let new_path = std::env::join_paths(paths).unwrap();
        unsafe { std::env::set_var("PATH", &new_path); }

        let resolved = resolve_cli_program("hibitestbin");

        unsafe { std::env::set_var("PATH", &prev_path); }
        let _ = std::fs::remove_file(&bin);

        assert_eq!(resolved, bin.into_os_string());
    }

    #[test]
    fn enrich_spawn_error_rewrites_not_found() {
        let cmd = Command::new("definitely_not_a_real_cli_xyz");
        let err = std::io::Error::from(std::io::ErrorKind::NotFound);
        let enriched = enrich_spawn_error(&cmd, err);
        let msg = format!("{}", enriched);
        assert!(msg.contains("definitely_not_a_real_cli_xyz"));
        assert!(msg.contains("not found in PATH"));
        assert!(msg.contains("type -a"));
        assert!(!msg.contains("os error 2"));
    }

    #[test]
    fn enrich_spawn_error_preserves_other_errors() {
        let cmd = Command::new("claude");
        let err = std::io::Error::from(std::io::ErrorKind::PermissionDenied);
        let enriched = enrich_spawn_error(&cmd, err);
        let msg = format!("{}", enriched);
        assert!(msg.contains("Failed to launch process"));
        assert!(!msg.contains("not found in PATH"));
    }

    #[test]
    fn enrich_spawn_error_keeps_io_error_in_chain() {
        // Callers that want to branch on `io::ErrorKind` must still be
        // able to downcast through the chain — `.context()` keeps the
        // original `io::Error` as the root, unlike a fresh `anyhow!()`.
        let cmd = Command::new("definitely_not_a_real_cli_xyz");
        let err = std::io::Error::from(std::io::ErrorKind::NotFound);
        let enriched = enrich_spawn_error(&cmd, err);

        let io_err = enriched.chain()
            .find_map(|e| e.downcast_ref::<std::io::Error>())
            .expect("io::Error should remain reachable in the error chain");
        assert_eq!(io_err.kind(), std::io::ErrorKind::NotFound);
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
