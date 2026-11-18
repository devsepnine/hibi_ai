use anyhow::Result;
use std::path::Path;

use super::validation::validate_mcp_server;
use crate::fs::create_cli_command;
use crate::mcp::{McpCatalog, McpServer, McpStatus};
use crate::target::TargetCli;

/// Scan MCP catalog and mark each server as installed or not.
pub(super) fn scan_with_installed(
    source_dir: &Path,
    installed: Option<&[String]>,
) -> Result<Vec<McpServer>> {
    let catalog_path = source_dir.join("mcps/mcps.yaml");
    if !catalog_path.exists() {
        return Ok(Vec::new());
    }

    let content = std::fs::read_to_string(&catalog_path)?;
    let catalog: McpCatalog = serde_yaml_bw::from_str(&content)?;

    let servers = catalog
        .servers
        .into_iter()
        .filter_map(|def| {
            if validate_mcp_server(&def).is_some() {
                return None;
            }
            let status = match installed {
                Some(names) if names.contains(&def.name) => McpStatus::Installed,
                Some(_) => McpStatus::NotInstalled,
                None => McpStatus::Unknown,
            };
            Some(McpServer::new(def, status))
        })
        .collect();

    Ok(servers)
}

/// Query the Claude CLI for installed MCP servers.
pub(super) fn get_installed_claude_servers(
    project: Option<&Path>,
) -> (Vec<String>, Option<String>) {
    let mut cmd = create_cli_command(TargetCli::Claude);
    cmd.args(["mcp", "list"]);
    if let Some(path) = project {
        cmd.current_dir(path);
    }

    match crate::fs::mcp_list_output(&mut cmd, "claude mcp list") {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out);
            let servers = stdout
                .lines()
                .filter(|line| {
                    let trimmed = line.trim();
                    !trimmed.is_empty() && !trimmed.starts_with("Checking") && trimmed.contains(':')
                })
                .filter_map(|line| {
                    let name = line.trim().split(':').next()?.trim();
                    if name.is_empty() {
                        return None;
                    }
                    Some(name.to_string())
                })
                .collect();
            (servers, None)
        }
        Err(e) => (Vec::new(), format_spawn_error(e)),
    }
}

/// Format a warning when the client cannot list its registrations.
fn format_spawn_error(e: anyhow::Error) -> Option<String> {
    let err_str = e.to_string();
    let hint = if err_str.contains("timed out") {
        " (health check timeout)"
    } else if err_str.contains("os error 2")
        || err_str.contains("not found")
        || err_str.contains("The system cannot find")
    {
        " (CLI not found in PATH)"
    } else {
        ""
    };
    Some(format!("MCP scan failed: {}{}", err_str, hint))
}
