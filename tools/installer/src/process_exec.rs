use std::path::PathBuf;
use std::sync::mpsc::Receiver;

use anyhow::Result;

use crate::app::{App, Tab, TargetCli};
use crate::component::Component;
use crate::fs;
use crate::mcp;
use crate::plugin;

/// Data needed for async install/remove on a background thread.
#[derive(Clone)]
pub(crate) enum ProcessData {
    Component {
        component: Component,
        source_dir: PathBuf,
        dest_dir: PathBuf,
    },
    McpServer {
        server: mcp::McpServer,
        scope: mcp::McpScope,
        project_path: Option<String>,
        env_values: Vec<(String, String)>,
    },
    Plugin {
        plugin: plugin::Plugin,
    },
}

/// Extract the display name for the item being processed.
pub(crate) fn get_item_name(app: &App, idx: usize) -> String {
    if app.tab == Tab::McpServers {
        app.mcp_servers.get(idx).map(|s| s.def.name.clone()).unwrap_or_default()
    } else if app.tab == Tab::Plugins {
        app.plugins.get(idx).map(|p| p.def.name.clone()).unwrap_or_default()
    } else {
        app.components.get(idx).map(|c| c.name.clone()).unwrap_or_default()
    }
}

/// Prepare thread-safe data from the current app state for async processing.
/// Returns None if the index is out of bounds.
pub(crate) fn prepare(app: &App, idx: usize) -> Option<ProcessData> {
    if app.tab == Tab::McpServers {
        let server = app.mcp_servers.get(idx)?.clone();
        let env_values = if app.env_input_server_idx == Some(idx) {
            app.env_input_values.clone()
        } else {
            Vec::new()
        };
        let project_path = if app.mcp_scope == mcp::McpScope::Local {
            Some(app.mcp_project_path.clone())
        } else {
            None
        };
        Some(ProcessData::McpServer { server, scope: app.mcp_scope, project_path, env_values })
    } else if app.tab == Tab::Plugins {
        let plugin = app.plugins.get(idx)?.clone();
        Some(ProcessData::Plugin { plugin })
    } else {
        let component = app.components.get(idx)?.clone();
        Some(ProcessData::Component {
            component,
            source_dir: app.source_dir.clone(),
            dest_dir: app.dest_dir.clone(),
        })
    }
}

/// Execute a single install or remove step on a background thread.
pub(crate) fn execute(
    data: ProcessData,
    is_removing: bool,
    target_cli: TargetCli,
    cancel_rx: Receiver<()>,
) -> Result<String> {
    match data {
        ProcessData::McpServer { server, scope, project_path, env_values } => {
            let name = server.def.name.clone();
            let timeout = if is_removing { 30 } else { 120 };

            let result = if is_removing {
                fs::installer::remove_mcp_server(&server, target_cli, timeout, &cancel_rx)
            } else {
                fs::installer::install_mcp_server(
                    &server,
                    fs::installer::McpInstallConfig {
                        scope,
                        project_path: project_path.as_deref(),
                        env_values: &env_values,
                        target_cli,
                        timeout_secs: timeout,
                        cancel_rx: &cancel_rx,
                    },
                )
            };

            format_result(&name, is_removing, result)
        }
        ProcessData::Plugin { plugin } => {
            let name = plugin.def.name.clone();
            let timeout = if is_removing { 30 } else { 60 };

            let result = if is_removing {
                fs::installer::remove_plugin(&plugin, timeout, &cancel_rx)
            } else {
                fs::installer::install_plugin(&plugin, timeout, &cancel_rx)
            };

            format_result(&name, is_removing, result)
        }
        ProcessData::Component { component, source_dir, dest_dir } => {
            let name = component.name.clone();

            let result = if is_removing {
                fs::installer::remove_component(&component, &dest_dir)
            } else {
                fs::installer::install_component(&component, &source_dir, &dest_dir)
            };

            // Component errors are non-fatal: report as [ERR] line, not Err
            let action = if is_removing { "Removed" } else { "Installed" };
            match result {
                Ok(_) => Ok(format!("[OK] {} {}", action, name)),
                Err(e) => Ok(format!("[ERR] {}: {}", name, e)),
            }
        }
    }
}

/// Format a process result into a status message.
fn format_result(name: &str, is_removing: bool, result: Result<()>) -> Result<String> {
    let action = if is_removing { "Removed" } else { "Installed" };
    match result {
        Ok(_) => Ok(format!("[OK] {} {}", action, name)),
        Err(e) => Err(e),
    }
}
