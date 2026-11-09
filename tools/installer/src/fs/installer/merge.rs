use anyhow::Result;
use serde_json::Value;
use std::path::Path;

use super::settings::repair_bundled_statusline;
use crate::component::statusline_binary_name;

/// Merge source settings.json into dest, with deep merge and hook append logic.
pub(super) fn merge_settings_json(source: &Path, dest: &Path) -> Result<()> {
    let source_content = std::fs::read_to_string(source)?;
    let source_json: Value = serde_json::from_str(&source_content)?;

    let mut merged = if dest.exists() {
        let dest_content = std::fs::read_to_string(dest)?;
        let mut dest_json: Value = serde_json::from_str(&dest_content)?;
        merge_json_values(&mut dest_json, &source_json);
        dest_json
    } else {
        source_json
    };

    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
        let binary_name = statusline_binary_name();
        if parent.join("statusline").join(binary_name).is_file() {
            repair_bundled_statusline(&mut merged, parent, binary_name);
        }
    }

    let output = serde_json::to_string_pretty(&merged)?;
    std::fs::write(dest, output)?;

    Ok(())
}

fn merge_json_values(dest: &mut Value, source: &Value) {
    match (dest, source) {
        (Value::Object(dest_map), Value::Object(source_map)) => {
            for (key, source_value) in source_map {
                match dest_map.get_mut(key) {
                    Some(dest_value) => {
                        if key == "hooks" {
                            merge_hooks(dest_value, source_value);
                        } else {
                            merge_json_values(dest_value, source_value);
                        }
                    }
                    None => {
                        dest_map.insert(key.clone(), source_value.clone());
                    }
                }
            }
        }
        (dest, source) => {
            *dest = source.clone();
        }
    }
}

fn merge_hooks(dest: &mut Value, source: &Value) {
    if let (Value::Object(dest_hooks), Value::Object(source_hooks)) = (dest, source) {
        for (hook_type, source_hook_array) in source_hooks {
            match dest_hooks.get_mut(hook_type) {
                Some(Value::Array(dest_array)) => {
                    if let Value::Array(source_array) = source_hook_array {
                        for source_item in source_array {
                            if !dest_array.contains(source_item) {
                                dest_array.push(source_item.clone());
                            }
                        }
                    }
                }
                None => {
                    dest_hooks.insert(hook_type.clone(), source_hook_array.clone());
                }
                _ => {}
            }
        }
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use crate::component::{Component, ComponentType, InstallStatus};
    use crate::fs::installer::{
        install_component,
        test_support::{put_text, unique_dir},
    };

    #[test]
    fn fresh_statusline_and_config_install_use_native_command_in_either_order() {
        let (native, foreign) = if cfg!(target_os = "macos") {
            ("statusline_macos", "statusline_linux")
        } else {
            ("statusline_linux", "statusline_macos")
        };
        for statusline_first in [true, false] {
            let root = unique_dir("statusline_install_order");
            let source = root.join("source");
            let dest = root.join(".claude");
            put_text(&source, &format!("statusline/{native}"), "native binary");
            let config = serde_json::json!({
                "statusLine": {"type": "command", "command": format!("~/.claude/statusline/{foreign}"), "padding": 2},
                "alwaysThinkingEnabled": true
            });
            put_text(&source, "settings.json", &config.to_string());
            let statusline = Component::new(
                ComponentType::Statusline,
                native.into(),
                source.join("statusline").join(native),
                dest.join("statusline").join(native),
                InstallStatus::New,
            );
            let settings = Component::new(
                ComponentType::ConfigFile,
                "settings.json".into(),
                source.join("settings.json"),
                dest.join("settings.json"),
                InstallStatus::New,
            );
            let order = if statusline_first {
                [&statusline, &settings]
            } else {
                [&settings, &statusline]
            };
            for component in order {
                install_component(component, &source, &dest).unwrap();
            }
            let installed: Value =
                serde_json::from_str(&std::fs::read_to_string(dest.join("settings.json")).unwrap())
                    .unwrap();
            assert_eq!(
                installed["statusLine"]["command"],
                format!("~/.claude/statusline/{native}"),
                "statusline_first={statusline_first}"
            );
            assert_eq!(installed["statusLine"]["padding"], 2);
            assert_eq!(installed["alwaysThinkingEnabled"], true);
            assert_eq!(
                std::fs::read_to_string(dest.join("statusline").join(native)).unwrap(),
                "native binary"
            );
            std::fs::remove_dir_all(root).unwrap();
        }
    }
}
