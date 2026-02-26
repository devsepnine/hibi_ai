pub mod scanner;
pub mod diff;
pub mod installer;

use std::process::{Command, Stdio};
use crate::app::TargetCli;

/// Create a Command to run CLI (Claude or Codex).
///
/// On Windows, invokes the `.cmd` file directly (e.g., `claude.cmd`) instead of
/// using `cmd /c`, which would re-interpret shell metacharacters in arguments
/// and enable command injection via crafted YAML inputs.
///
/// stdin is set to null to prevent blocking on interactive prompts.
#[cfg(windows)]
pub fn create_cli_command(target_cli: TargetCli) -> Command {
    let cli_name = match target_cli {
        TargetCli::Claude => "claude.cmd",
        TargetCli::Codex => "codex.cmd",
    };
    let mut cmd = Command::new(cli_name);
    cmd.stdin(Stdio::null());
    cmd
}

#[cfg(not(windows))]
pub fn create_cli_command(target_cli: TargetCli) -> Command {
    let cli_name = match target_cli {
        TargetCli::Claude => "claude",
        TargetCli::Codex => "codex",
    };
    let mut cmd = Command::new(cli_name);
    cmd.stdin(Stdio::null());
    cmd
}

/// Deprecated: Use create_cli_command() instead
pub fn create_claude_command() -> Command {
    create_cli_command(TargetCli::Claude)
}
