mod process;
mod settings;
mod mcp;
mod plugin;

use std::path::Path;
use anyhow::Result;

use crate::component::{Component, ComponentType};
use settings::{
    register_hook_in_settings, unregister_hook_from_settings,
    register_output_style_in_settings, register_statusline_in_settings,
    merge_settings_json,
};

// Re-export public API
pub use mcp::{install_mcp_server, remove_mcp_server, McpInstallConfig};
pub use plugin::{install_plugin, remove_plugin};
pub use settings::{
    set_output_style, unset_output_style,
    set_statusline, unset_statusline,
    remove_managed_settings_sections,
};

/// Automatically clean up deprecated hooks that are already installed.
/// Returns a list of hook names that were cleaned up.
pub fn auto_cleanup_deprecated_hooks(source_dir: &Path, dest_dir: &Path) -> Vec<String> {
    let hooks_dir = source_dir.join("hooks");
    if !hooks_dir.exists() {
        return Vec::new();
    }

    let mut cleaned = Vec::new();

    let entries = match std::fs::read_dir(&hooks_dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let hook_yaml = path.join("hook.yaml");
        if !hook_yaml.exists() {
            continue;
        }

        let config_content = match std::fs::read_to_string(&hook_yaml) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let config: crate::component::HookConfig = match serde_yaml::from_str(&config_content) {
            Ok(c) => c,
            Err(_) => continue,
        };

        if !config.is_deprecated() {
            continue;
        }

        let binary_name = config.binary_name();
        let dest_path = dest_dir.join("hooks").join(&binary_name);

        if !dest_path.exists() {
            continue;
        }

        // Unregister from settings.json (ignore errors)
        let _ = settings::unregister_hook_from_settings(dest_dir, &config);

        // Remove binary file
        if std::fs::remove_file(&dest_path).is_ok() {
            cleaned.push(config.name.clone());
        }
    }

    cleaned
}

pub fn install_component(component: &Component, _source_dir: &Path, dest_dir: &Path) -> Result<()> {
    match &component.component_type {
        ComponentType::Hooks => {
            if let Some(config) = &component.hook_config {
                if config.is_deprecated() {
                    anyhow::bail!("Hook '{}' is deprecated and cannot be installed", component.name);
                }
                // Copy hook binary and register in settings.json
                copy_file(component)?;
                register_hook_in_settings(dest_dir, config)?;
            } else {
                copy_file(component)?;
            }
        }
        ComponentType::OutputStyles => {
            // Copy output style file
            copy_file(component)?;
            // Auto-register in settings.json if no style is currently set
            register_output_style_in_settings(dest_dir, &component.name)?;
        }
        ComponentType::Statusline => {
            // Copy statusline file
            copy_file(component)?;
            // Auto-register in settings.json if no statusline is currently set
            register_statusline_in_settings(dest_dir, &component.name)?;
        }
        ComponentType::ConfigFile if component.name == "settings.json" => {
            // Merge settings.json instead of overwriting
            merge_settings_json(&component.source_path, &component.dest_path)?;
        }
        _ => {
            copy_file(component)?;
        }
    }
    Ok(())
}

pub fn remove_component(component: &Component, dest_dir: &Path) -> Result<()> {
    match &component.component_type {
        ComponentType::Hooks => {
            // Unregister hook from settings.json using hook_config
            if let Some(config) = &component.hook_config {
                unregister_hook_from_settings(dest_dir, config)?;
            }
            // Remove hook binary file
            if component.dest_path.exists() {
                std::fs::remove_file(&component.dest_path)?;
            }
        }
        ComponentType::ConfigFile if component.name == "settings.json" => {
            // Remove installer-managed sections from settings.json instead of deleting the file
            remove_managed_settings_sections(dest_dir)?;
        }
        _ => {
            // Remove file
            if component.dest_path.exists() {
                std::fs::remove_file(&component.dest_path)?;
            }
        }
    }
    Ok(())
}

fn copy_file(component: &Component) -> Result<()> {
    // Security: reject paths containing '..' to prevent path traversal
    if component.dest_path.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
        anyhow::bail!("Security: destination path contains '..' component: {:?}", component.dest_path);
    }

    // Create parent directory if needed
    if let Some(parent) = component.dest_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Copy file
    std::fs::copy(&component.source_path, &component.dest_path)?;

    // Set executable permission for shell scripts
    #[cfg(unix)]
    if component.component_type == ComponentType::Statusline
        || component.source_path.extension().map_or(false, |e| e == "sh")
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&component.dest_path)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&component.dest_path, perms)?;
    }

    Ok(())
}
