use std::path::Path;
use anyhow::Result;

use crate::plugin::{parse_plugins_yaml, Plugin, PluginDef, PluginStatus};
use super::validation::validate_plugin;

/// Scan plugin catalog and mark each as installed or not.
pub(super) fn scan_plugins(source_dir: &Path) -> Result<Vec<Plugin>> {
    let catalog_path = source_dir.join("plugins/plugins.yaml");
    if !catalog_path.exists() {
        return Ok(Vec::new());
    }

    let content = std::fs::read_to_string(&catalog_path)?;
    let catalog = parse_plugins_yaml(&content);
    let installed = get_installed_plugins();

    let mut plugins = Vec::new();
    for (marketplace, source, name, comment) in catalog {
        if validate_plugin(&name, &marketplace, &source).is_some() {
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

/// Read installed plugins from ~/.claude/settings.json.
fn get_installed_plugins() -> Vec<String> {
    use serde_json::Value;

    let settings_path = dirs::home_dir()
        .map(|h| h.join(".claude").join("settings.json"))
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

    let enabled_plugins = match settings.get("enabledPlugins") {
        Some(Value::Object(map)) => map,
        _ => return Vec::new(),
    };

    // Extract plugin names from "plugin@marketplace" format
    enabled_plugins
        .iter()
        .filter_map(|(key, value)| {
            if value.as_bool() == Some(true) {
                key.split('@').next().map(|s| s.to_string())
            } else {
                None
            }
        })
        .collect()
}
