use std::path::Path;
use anyhow::Result;

use crate::app::TargetCli;
use crate::mcp::{McpCatalog, McpServer, McpStatus};
use crate::fs::create_cli_command;
use super::validation::validate_mcp_server;

/// Timeout for MCP server scan (seconds).
/// Health checks across multiple servers can be slow; 30s is a reasonable ceiling.
const MCP_SCAN_TIMEOUT_SECS: u64 = 30;

/// Scan MCP catalog and mark each server as installed or not.
pub(super) fn scan_with_installed(source_dir: &Path, installed: &[String]) -> Result<Vec<McpServer>> {
    let catalog_path = source_dir.join("mcps/mcps.yaml");
    if !catalog_path.exists() {
        return Ok(Vec::new());
    }

    let content = std::fs::read_to_string(&catalog_path)?;
    let catalog: McpCatalog = serde_yaml::from_str(&content)?;

    let servers = catalog
        .servers
        .into_iter()
        .filter_map(|def| {
            if validate_mcp_server(&def).is_some() {
                return None;
            }
            let status = if installed.contains(&def.name) {
                McpStatus::Installed
            } else {
                McpStatus::NotInstalled
            };
            Some(McpServer::new(def, status))
        })
        .collect();

    Ok(servers)
}

/// Query the Claude CLI for installed MCP servers.
pub(super) fn get_installed_claude_servers() -> (Vec<String>, Option<String>) {
    let mut cmd = create_cli_command(TargetCli::Claude);
    cmd.args(["mcp", "list"]);

    match crate::fs::run_with_timeout(&mut cmd, MCP_SCAN_TIMEOUT_SECS) {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let servers = stdout
                .lines()
                .filter(|line| {
                    let trimmed = line.trim();
                    !trimmed.is_empty()
                        && !trimmed.starts_with("Checking")
                        && trimmed.contains(':')
                })
                .filter_map(|line| {
                    let name = line.trim().split(':').next()?.trim();
                    if name.is_empty() { return None; }
                    Some(name.to_string())
                })
                .collect();
            (servers, None)
        }
        Ok(ref out) => (Vec::new(), format_scan_error("Claude", out)),
        Err(e) => (Vec::new(), format_spawn_error(e)),
    }
}

/// Query the Codex CLI for installed MCP servers.
pub(super) fn get_installed_codex_servers() -> (Vec<String>, Option<String>) {
    let mut cmd = create_cli_command(TargetCli::Codex);
    cmd.args(["mcp", "list"]);

    match crate::fs::run_with_timeout(&mut cmd, MCP_SCAN_TIMEOUT_SECS) {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let servers = stdout
                .lines()
                .filter_map(|line| {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        return None;
                    }
                    // Skip header lines (contain "Name" or "Command" or "Url" columns)
                    if trimmed.starts_with("Name") && (trimmed.contains("Command") || trimmed.contains("Url")) {
                        return None;
                    }
                    let name = trimmed.split_whitespace().next()?.trim();
                    if name.is_empty() || name == "Name" {
                        return None;
                    }
                    Some(name.to_string())
                })
                .collect();
            (servers, None)
        }
        Ok(ref out) => (Vec::new(), format_scan_error("Codex", out)),
        Err(e) => (Vec::new(), format_spawn_error(e)),
    }
}

/// Format a warning when `mcp list` exits with a non-success status code.
fn format_scan_error(cli_label: &str, result: &std::process::Output) -> Option<String> {
    let stderr = String::from_utf8_lossy(&result.stderr);
    let code = result.status.code().map(|c| c.to_string()).unwrap_or_else(|| "unknown".to_string());
    Some(format!("MCP scan: {} CLI exited with code {}: {}", cli_label, code, stderr.trim()))
}

/// Format a warning when spawning/running the CLI command fails entirely.
fn format_spawn_error(e: anyhow::Error) -> Option<String> {
    let err_str = e.to_string();
    let hint = if err_str.contains("timed out") {
        " (health check timeout)"
    } else if err_str.contains("os error 2") || err_str.contains("not found") || err_str.contains("The system cannot find") {
        " (CLI not found in PATH)"
    } else {
        ""
    };
    Some(format!("MCP scan failed: {}{}", err_str, hint))
}
