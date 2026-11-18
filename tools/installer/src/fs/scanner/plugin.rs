use anyhow::Result;
use serde_json::Value;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use super::validation::validate_plugin;
use crate::plugin::{Plugin, PluginDef, PluginStatus, parse_plugins_yaml};

/// Scan plugin catalog and mark each as installed or not.
pub(super) fn scan_plugins(
    source_dir: &Path,
    installed: Option<&HashSet<String>>,
) -> Result<Vec<Plugin>> {
    let catalog_path = source_dir.join("plugins/plugins.yaml");
    if !catalog_path.exists() {
        return Ok(Vec::new());
    }

    let content = std::fs::read_to_string(&catalog_path)?;
    let catalog = parse_plugins_yaml(&content);

    let mut plugins = Vec::new();
    for (marketplace, source, name, comment) in catalog {
        if validate_plugin(&name, &marketplace, &source).is_some() {
            continue;
        }

        let def = PluginDef {
            name,
            marketplace,
            source,
            comment,
        };

        let status = match installed {
            Some(ids) if ids.contains(&def.id()) => PluginStatus::Installed,
            Some(_) => PluginStatus::NotInstalled,
            None => PluginStatus::Unknown,
        };
        plugins.push(Plugin::new(def, status));
    }

    Ok(plugins)
}

pub(super) fn get_installed_plugins() -> Option<HashSet<String>> {
    let dir = std::env::var_os("CLAUDE_CONFIG_DIR")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .or_else(|| crate::paths::home_dir().map(|home| home.join(".claude")))?;
    installed_plugins_in(&dir).ok()
}

fn read_optional_json(path: &Path) -> Result<Option<Value>> {
    match std::fs::read(path) {
        Ok(bytes) => Ok(Some(serde_json::from_slice(&bytes)?)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}

fn installed_plugins_in(dir: &Path) -> Result<HashSet<String>> {
    if let Some(registry) = read_optional_json(&dir.join("plugins/installed_plugins.json"))? {
        match registry["version"].as_u64() {
            Some(2) => {
                let entries = registry["plugins"]
                    .as_object()
                    .ok_or_else(|| anyhow::anyhow!("Missing plugin entries"))?;
                let mut ids = HashSet::new();
                for (id, scopes) in entries {
                    let scopes = scopes
                        .as_array()
                        .ok_or_else(|| anyhow::anyhow!("Invalid plugin scopes"))?;
                    if scopes.iter().any(|entry| entry["scope"] == "user") {
                        ids.insert(id.clone());
                    }
                }
                return Ok(ids);
            }
            Some(1) => {}
            _ => anyhow::bail!("Unknown Claude plugin registry version"),
        }
    }

    // Older clients keep enabled plugin IDs in user settings.
    let settings = read_optional_json(&dir.join("settings.json"))?;
    Ok(settings
        .as_ref()
        .and_then(|s| s["enabledPlugins"].as_object())
        .into_iter()
        .flatten()
        .filter(|(_, enabled)| enabled.as_bool() == Some(true))
        .map(|(id, _)| id.clone())
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_install_status_tracks_marketplace_and_scope_even_when_disabled() {
        let home = tempfile::tempdir().unwrap();
        let dir = home.path();
        std::fs::write(
            dir.join("settings.json"),
            r#"{"enabledPlugins":{"tool@one":false,"tool@two":true}}"#,
        )
        .unwrap();
        assert_eq!(
            installed_plugins_in(dir).unwrap(),
            HashSet::from(["tool@two".into()])
        );
        std::fs::create_dir(dir.join("plugins")).unwrap();
        std::fs::write(dir.join("plugins/installed_plugins.json"), r#"{"version":2,"plugins":{"tool@one":[{"scope":"user"}],"tool@two":[{"scope":"local"}]}}"#).unwrap();
        assert_eq!(
            installed_plugins_in(dir).unwrap(),
            HashSet::from(["tool@one".into()])
        );
    }
}
