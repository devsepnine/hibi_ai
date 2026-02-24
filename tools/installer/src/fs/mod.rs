pub mod scanner;
pub mod diff;
pub mod installer;

use std::process::{Command, Stdio};
use crate::app::TargetCli;

/// Create a Command to run CLI (Claude or Codex).
/// On Windows, uses cmd.exe /c to properly execute .cmd files
/// stdin is set to null to prevent blocking on interactive prompts.
#[cfg(windows)]
pub fn create_cli_command(target_cli: TargetCli) -> Command {
    let cli_name = match target_cli {
        TargetCli::Claude => "claude",
        TargetCli::Codex => "codex",
    };
    let mut cmd = Command::new("cmd");
    cmd.args(["/c", cli_name]);
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
#[cfg(windows)]
pub fn create_claude_command() -> Command {
    create_cli_command(TargetCli::Claude)
}

/// Deprecated: Use create_cli_command() instead
#[cfg(not(windows))]
pub fn create_claude_command() -> Command {
    create_cli_command(TargetCli::Claude)
}
