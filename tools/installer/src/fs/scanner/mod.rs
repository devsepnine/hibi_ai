mod validation;

use std::path::Path;
use anyhow::Result;
use walkdir::WalkDir;

use crate::app::TargetCli;
use crate::component::{Component, ComponentType, HookConfig, InstallStatus};
use crate::mcp::{McpCatalog, McpServer, McpStatus};
use crate::plugin::{parse_plugins_yaml, Plugin, PluginDef, PluginStatus};
use crate::fs::{create_claude_command, create_cli_command};
use validation::{validate_mcp_server, validate_plugin};

pub fn scan_components(source_dir: &Path, dest_dir: &Path, target_cli: TargetCli) -> Result<Vec<Component>> {
    let mut components = Vec::new();

    match target_cli {
        TargetCli::Claude => {
            // Scan all components for Claude Code
            scan_directory(
                &source_dir.join("agents"),
                &dest_dir.join("agents"),
                ComponentType::Agents,
                &mut components,
            )?;

            scan_directory(
                &source_dir.join("commands"),
                &dest_dir.join("commands"),
                ComponentType::Commands,
                &mut components,
            )?;

            scan_directory(
                &source_dir.join("contexts"),
                &dest_dir.join("contexts"),
                ComponentType::Contexts,
                &mut components,
            )?;

            scan_directory(
                &source_dir.join("rules"),
                &dest_dir.join("rules"),
                ComponentType::Rules,
                &mut components,
            )?;

            scan_directory(
                &source_dir.join("skills"),
                &dest_dir.join("skills"),
                ComponentType::Skills,
                &mut components,
            )?;

            scan_directory(
                &source_dir.join("output-styles"),
                &dest_dir.join("output-styles"),
                ComponentType::OutputStyles,
                &mut components,
            )?;

            scan_statusline(source_dir, dest_dir, &mut components)?;
            scan_hooks(source_dir, dest_dir, &mut components)?;
            add_config_files(source_dir, dest_dir, target_cli, &mut components)?;
        }
        TargetCli::Codex => {
            // Only scan skills for Codex CLI
            scan_directory(
                &source_dir.join("skills"),
                &dest_dir.join("skills"),
                ComponentType::Skills,
                &mut components,
            )?;

            add_config_files(source_dir, dest_dir, target_cli, &mut components)?;
        }
    }

    Ok(components)
}

fn scan_directory(
    source_dir: &Path,
    dest_dir: &Path,
    component_type: ComponentType,
    components: &mut Vec<Component>,
) -> Result<()> {
    if !source_dir.exists() {
        return Ok(());
    }

    for entry in WalkDir::new(source_dir)
        .min_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        // Skip files ending with -ko.md
        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
            if file_name.ends_with("-ko.md") {
                continue;
            }
        }

        let relative = path.strip_prefix(source_dir)?;

        // Security: reject path traversal attempts
        if relative.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
            continue;
        }

        let dest_path = dest_dir.join(relative);
        let name = relative.to_string_lossy().to_string();

        let status = determine_status(path, &dest_path)?;

        components.push(Component::new(
            component_type.clone(),
            name,
            path.to_path_buf(),
            dest_path,
            status,
        ));
    }

    Ok(())
}

fn scan_statusline(source_dir: &Path, dest_dir: &Path, components: &mut Vec<Component>) -> Result<()> {
    let statusline_dir = source_dir.join("statusline");
    if !statusline_dir.exists() {
        return Ok(());
    }

    // Select OS-specific binary
    let binary_name = if cfg!(windows) {
        "statusline.exe"
    } else if cfg!(target_os = "macos") {
        "statusline_macos"
    } else {
        "statusline_linux"
    };

    let binary_path = statusline_dir.join(binary_name);
    if !binary_path.exists() {
        return Ok(());
    }

    let dest_path = dest_dir.join("statusline").join(binary_name);
    let status = determine_status(&binary_path, &dest_path)?;

    components.push(Component::new(
        ComponentType::Statusline,
        binary_name.to_string(),
        binary_path,
        dest_path,
        status,
    ));

    Ok(())
}

fn scan_hooks(source_dir: &Path, dest_dir: &Path, components: &mut Vec<Component>) -> Result<()> {
    let hooks_dir = source_dir.join("hooks");
    if !hooks_dir.exists() {
        return Ok(());
    }

    // Scan hook directories
    for entry in std::fs::read_dir(&hooks_dir)? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let hook_yaml = path.join("hook.yaml");
        if !hook_yaml.exists() {
            continue;
        }

        // Read hook.yaml
        let config_content = std::fs::read_to_string(&hook_yaml)?;
        let config: HookConfig = serde_yaml::from_str(&config_content)?;

        // Find binary in the hook directory
        let hook_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        // Select OS-specific binary using HookConfig method
        let binary_name = config.binary_name();

        let binary_path = path.join(&binary_name);
        if !binary_path.exists() {
            continue;
        }

        let dest_path = dest_dir.join("hooks").join(&binary_name);
        let status = determine_status(&binary_path, &dest_path)?;

        let component = Component::new(
            ComponentType::Hooks,
            hook_name.to_string(),
            binary_path,
            dest_path,
            status,
        ).with_hook_config(config);

        components.push(component);
    }

    Ok(())
}

fn add_config_files(
    source_dir: &Path,
    dest_dir: &Path,
    target_cli: TargetCli,
    components: &mut Vec<Component>,
) -> Result<()> {
    // CLI-specific config files
    let config_files = match target_cli {
        TargetCli::Claude => vec!["settings.json", "CLAUDE.md"],
        TargetCli::Codex => vec!["AGENTS.md"],
    };

    for file in config_files {
        let source_path = source_dir.join(file);
        if source_path.exists() {
            let dest_path = dest_dir.join(file);

            // settings.json is always Managed (auto-merged)
            let status = if file == "settings.json" && dest_path.exists() {
                InstallStatus::Managed
            } else {
                determine_status(&source_path, &dest_path)?
            };

            components.push(Component::new(
                ComponentType::ConfigFile,
                file.to_string(),
                source_path,
                dest_path,
                status,
            ));
        }
    }

    Ok(())
}

fn determine_status(source: &Path, dest: &Path) -> Result<InstallStatus> {
    if !dest.exists() {
        return Ok(InstallStatus::New);
    }

    // Quick check: compare file size first (avoid reading large files)
    let source_meta = std::fs::metadata(source)?;
    let dest_meta = std::fs::metadata(dest)?;

    // If sizes differ, definitely modified
    if source_meta.len() != dest_meta.len() {
        return Ok(InstallStatus::Modified);
    }

    // Sizes match - compare contents for accuracy
    let source_content = std::fs::read(source)?;
    let dest_content = std::fs::read(dest)?;

    if source_content == dest_content {
        Ok(InstallStatus::Unchanged)
    } else {
        Ok(InstallStatus::Modified)
    }
}

pub fn scan_mcp_servers(source_dir: &Path, target_cli: TargetCli, _dest_dir: &Path) -> Result<Vec<McpServer>> {
    // Both CLIs use the same catalog
    let catalog_path = source_dir.join("mcps/mcps.yaml");

    if !catalog_path.exists() {
        return Ok(Vec::new());
    }

    let content = std::fs::read_to_string(&catalog_path)?;
    let catalog: McpCatalog = serde_yaml::from_str(&content)?;

    // Get installed MCP servers based on CLI
    let installed = match target_cli {
        TargetCli::Claude => get_installed_claude_mcp_servers(),
        TargetCli::Codex => get_installed_codex_mcp_servers(),
    };

    let servers = catalog
        .servers
        .into_iter()
        .filter_map(|def| {
            if let Some(warning) = validate_mcp_server(&def) {
                // Skip invalid entries with a warning (not eprintln! in TUI)
                // The warning is silently dropped; future work could surface this in the UI
                let _ = warning;
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

fn get_installed_claude_mcp_servers() -> Vec<String> {
    let mut cmd = create_claude_command();
    cmd.args(["mcp", "list"]);
    let output = cmd.output();

    match output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            stdout
                .lines()
                .skip(1) // Skip "Checking MCP server health..." line
                .filter_map(|line| {
                    let trimmed = line.trim();
                    // Skip empty lines
                    if trimmed.is_empty() {
                        return None;
                    }
                    // Parse "servername: command - status" format
                    // Extract server name before the first ':'
                    let name = trimmed.split(':').next()?.trim();
                    // Filter out lines that don't have the expected format
                    if name.is_empty() || !trimmed.contains(':') {
                        return None;
                    }
                    Some(name.to_string())
                })
                .collect()
        }
        _ => Vec::new(),
    }
}

fn get_installed_codex_mcp_servers() -> Vec<String> {
    let mut cmd = create_cli_command(TargetCli::Codex);
    cmd.args(["mcp", "list"]);
    let output = cmd.output();

    match output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            stdout
                .lines()
                .filter_map(|line| {
                    let trimmed = line.trim();

                    // Skip empty lines
                    if trimmed.is_empty() {
                        return None;
                    }

                    // Skip header lines (contain "Name" or "Command" or "Url" columns)
                    if trimmed.starts_with("Name") && (trimmed.contains("Command") || trimmed.contains("Url")) {
                        return None;
                    }

                    // Extract first column (server name) from whitespace-separated table
                    let name = trimmed.split_whitespace().next()?.trim();

                    // Skip if name is empty or looks like a header
                    if name.is_empty() || name == "Name" {
                        return None;
                    }

                    Some(name.to_string())
                })
                .collect()
        }
        _ => Vec::new(),
    }
}

pub fn scan_plugins(source_dir: &Path) -> Result<Vec<Plugin>> {
    let catalog_path = source_dir.join("plugins/plugins.yaml");

    if !catalog_path.exists() {
        return Ok(Vec::new());
    }

    let content = std::fs::read_to_string(&catalog_path)?;
    let catalog = parse_plugins_yaml(&content);

    // Get installed plugins
    let installed = get_installed_plugins();

    let mut plugins = Vec::new();
    for (marketplace, source, name, comment) in catalog {
        if let Some(warning) = validate_plugin(&name, &marketplace, &source) {
            // Skip invalid entries (silently; future work could surface in the UI)
            let _ = warning;
            continue;
        }

        let status = if installed.contains(&name) {
            PluginStatus::Installed
        } else {
            PluginStatus::NotInstalled
        };

        let def = PluginDef {
            name,
            marketplace,
            source,
            comment,
        };

        plugins.push(Plugin::new(def, status));
    }

    Ok(plugins)
}

fn get_installed_plugins() -> Vec<String> {
    use serde_json::Value;

    // Read from ~/.claude/settings.json
    let settings_path = dirs::home_dir()
        .map(|h| h.join(".claude/settings.json"))
        .filter(|p| p.exists());

    let settings_path = match settings_path {
        Some(p) => p,
        None => return Vec::new(),
    };

    let content = match std::fs::read_to_string(&settings_path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let settings: Value = match serde_json::from_str(&content) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    // Get enabledPlugins object
    let enabled_plugins = match settings.get("enabledPlugins") {
        Some(Value::Object(map)) => map,
        _ => return Vec::new(),
    };

    // Extract plugin names from "plugin@marketplace" format
    enabled_plugins
        .iter()
        .filter_map(|(key, value)| {
            // Only include if value is true
            if value.as_bool() == Some(true) {
                // Extract plugin name before '@'
                // e.g., "document-skills@anthropic-agent-skills" -> "document-skills"
                key.split('@').next().map(|s| s.to_string())
            } else {
                None
            }
        })
        .collect()
}
