use std::sync::mpsc::Receiver;
use anyhow::Result;

use crate::app::TargetCli;
use crate::mcp::{McpServer, McpScope};
use super::process::{
    spawn_cancelable_process, run_with_timeout, run_cleanup_command,
    ProcessConfig, QUICK_COMMAND_TIMEOUT_SECS,
};
use crate::fs::create_cli_command;

/// Split a command string into arguments.
/// Uses shlex (POSIX rules) on Unix, Windows-aware splitting on Windows.
fn split_command(cmd: &str) -> Option<Vec<String>> {
    #[cfg(windows)]
    {
        // Windows: split on whitespace, respecting double and single-quoted strings.
        // Backslashes are literal (not escape chars) to support Windows paths.
        let mut args = Vec::new();
        let mut current = String::new();
        let mut quote_char: Option<char> = None;
        for ch in cmd.chars() {
            match ch {
                '"' | '\'' if quote_char == Some(ch) => { quote_char = None; }
                '"' | '\'' if quote_char.is_none() => { quote_char = Some(ch); }
                ' ' | '\t' if quote_char.is_none() => {
                    if !current.is_empty() {
                        args.push(std::mem::take(&mut current));
                    }
                }
                _ => current.push(ch),
            }
        }
        if !current.is_empty() {
            args.push(current);
        }
        if quote_char.is_some() { None } else { Some(args) }
    }
    #[cfg(not(windows))]
    {
        shlex::split(cmd)
    }
}

/// Cleanup helper: try to remove MCP server without blocking
/// Returns true if cleanup succeeded, false otherwise
fn cleanup_mcp_installation(server: &McpServer, target_cli: TargetCli) -> bool {
    let mut command = create_cli_command(target_cli);
    command.args(["mcp", "remove", &server.def.name]);
    run_cleanup_command(&mut command)
}

/// Configuration for MCP server installation.
pub struct McpInstallConfig<'a> {
    pub scope: McpScope,
    pub project_path: Option<&'a str>,
    pub env_values: &'a [(String, String)],
    pub target_cli: TargetCli,
    pub timeout_secs: u64,
    pub cancel_rx: &'a Receiver<()>,
}

pub fn install_mcp_server(
    server: &McpServer,
    config: McpInstallConfig,
) -> Result<()> {
    let mut command = create_cli_command(config.target_cli);
    command.arg("mcp").arg("add");

    match config.target_cli {
        TargetCli::Claude => {
            command.arg("--scope").arg(config.scope.display());
            command.arg(&server.def.name);

            for (key, value) in config.env_values {
                command.arg("-e").arg(format!("{}={}", key, value));
            }

            if server.is_http() {
                command.arg("-t").arg("http");
                if let Some(url) = &server.def.url {
                    command.arg(url);
                }
            } else {
                command.arg("--");
                if let Some(cmd_str) = &server.def.command {
                    let parts = split_command(cmd_str)
                        .ok_or_else(|| anyhow::anyhow!("Invalid command syntax: {}", cmd_str))?;
                    for part in parts {
                        command.arg(part);
                    }
                }
            }

            if let Some(path) = config.project_path {
                command.current_dir(path);
            }
        }
        TargetCli::Codex => {
            for (key, value) in config.env_values {
                command.arg("--env").arg(format!("{}={}", key, value));
            }

            command.arg(&server.def.name);

            if server.is_http() {
                if let Some(url) = &server.def.url {
                    command.arg("--url").arg(url);
                }
            } else {
                command.arg("--");
                if let Some(cmd_str) = &server.def.command {
                    let parts = split_command(cmd_str)
                        .ok_or_else(|| anyhow::anyhow!("Invalid command syntax: {}", cmd_str))?;
                    for part in parts {
                        command.arg(part);
                    }
                }
            }
        }
    }

    let server_clone = server.clone();
    spawn_cancelable_process(
        &mut command,
        ProcessConfig {
            timeout_secs: config.timeout_secs,
            cancel_rx: config.cancel_rx,
            item_name: &server.def.name,
            action: "install MCP server",
            cleanup: Some(Box::new(move || cleanup_mcp_installation(&server_clone, config.target_cli))),
        },
    )
}

pub fn remove_mcp_server(
    server: &McpServer,
    target_cli: TargetCli,
    timeout_secs: u64,
    cancel_rx: &Receiver<()>,
) -> Result<()> {
    let mut command = create_cli_command(target_cli);
    command.args(["mcp", "remove", &server.def.name]);

    spawn_cancelable_process(
        &mut command,
        ProcessConfig {
            timeout_secs,
            cancel_rx,
            item_name: &server.def.name,
            action: "remove MCP server",
            cleanup: None,
        },
    )
}

/// Ensure the plugin's marketplace is registered before installation.
///
/// Step 1: Quick check if already added (non-cancelable, short timeout).
/// Step 2: If not found, add it using `spawn_cancelable_process` (cancelable, respects timeout).
pub(super) fn ensure_marketplace_added(
    marketplace: &str,
    source: &str,
    timeout_secs: u64,
    cancel_rx: &Receiver<()>,
) -> Result<()> {
    // Step 1: Check if marketplace is already added (quick, non-cancelable)
    let mut list_cmd = create_cli_command(TargetCli::Claude);
    list_cmd.args(["plugin", "marketplace", "list"]);

    match run_with_timeout(&mut list_cmd, QUICK_COMMAND_TIMEOUT_SECS) {
        Ok(list_output) if list_output.status.success() => {
            let stdout = String::from_utf8_lossy(&list_output.stdout);
            if stdout.contains(marketplace) {
                return Ok(()); // Already added
            }
        }
        Ok(_) => {} // Non-success status: fall through to add
        Err(_) => {} // Timeout or spawn error: fall through to add
    }

    // Step 2: Add the marketplace (cancelable, full timeout)
    let mut command = create_cli_command(TargetCli::Claude);
    command.args(["plugin", "marketplace", "add", source]);

    spawn_cancelable_process(
        &mut command,
        ProcessConfig {
            timeout_secs,
            cancel_rx,
            item_name: marketplace,
            action: "add marketplace",
            cleanup: None,
        },
    )
}
