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
        app.mcp_servers
            .get(idx)
            .map(|s| s.def.name.clone())
            .unwrap_or_default()
    } else if app.tab == Tab::Plugins {
        app.plugins.get(idx).map(|p| p.def.id()).unwrap_or_default()
    } else {
        app.components
            .get(idx)
            .map(|c| c.name.clone())
            .unwrap_or_default()
    }
}

/// Prepare thread-safe data from the current app state for async processing.
/// Returns None if the index is out of bounds.
pub(crate) fn prepare(app: &App, idx: usize) -> Option<ProcessData> {
    if app.tab == Tab::McpServers {
        let server = app.mcp_servers.get(idx)?.clone();
        let env_values = app.mcp_env_values.get(&idx).cloned().unwrap_or_default();
        let project_path = app.local_mcp_project().map(str::to_owned);
        Some(ProcessData::McpServer {
            server,
            scope: app.mcp_scope,
            project_path,
            env_values,
        })
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
        ProcessData::McpServer {
            server,
            scope,
            project_path,
            env_values,
        } => {
            let name = server.def.name.clone();
            let timeout = if is_removing { 30 } else { 120 };

            if is_removing {
                let result = fs::installer::remove_mcp_server(
                    &server,
                    target_cli,
                    scope,
                    project_path.as_deref(),
                    timeout,
                    &cancel_rx,
                );
                return format_result(&name, true, result);
            }
            let result = fs::installer::install_mcp_server(
                &server,
                fs::installer::McpInstallConfig {
                    scope,
                    project_path: project_path.as_deref(),
                    env_values: &env_values,
                    target_cli,
                    timeout_secs: timeout,
                    cancel_rx: &cancel_rx,
                },
            )?;
            match result {
                fs::installer::McpRegistration::AlreadyRegistered => {
                    Ok(format!("[SKIP] {name} is already registered."))
                }
                fs::installer::McpRegistration::Registered(hint) => Ok(format!(
                    "[OK] Registered {name}{}",
                    hint.map(|s| format!(". {s}")).unwrap_or_default()
                )),
            }
        }
        ProcessData::Plugin { plugin } => {
            let name = plugin.def.id();
            let timeout = if is_removing { 30 } else { 60 };

            let result = if is_removing {
                fs::installer::remove_plugin(&plugin, timeout, &cancel_rx)
            } else {
                fs::installer::install_plugin(&plugin, timeout, &cancel_rx)
            };

            format_result(&name, is_removing, result)
        }
        ProcessData::Component {
            component,
            source_dir,
            dest_dir,
        } => {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::test_support::{fresh_app, make_mcp_server};

    #[test]
    fn mcp_credentials_are_isolated_by_server() {
        let mut app = fresh_app();
        app.tab = Tab::McpServers;
        app.mcp_servers = vec![make_mcp_server(), make_mcp_server(), make_mcp_server()];
        app.mcp_env_values
            .insert(0, vec![("TOKEN".into(), "first".into())]);
        app.mcp_env_values
            .insert(1, vec![("TOKEN".into(), "second".into())]);
        for (index, expected) in [(0, Some("first")), (1, Some("second")), (2, None)] {
            let Some(ProcessData::McpServer { env_values, .. }) = prepare(&app, index) else {
                panic!("expected MCP process data");
            };
            assert_eq!(
                env_values.first().map(|(_, value)| value.as_str()),
                expected
            );
        }
        assert!(prepare(&app, usize::MAX).is_none());
    }
}
