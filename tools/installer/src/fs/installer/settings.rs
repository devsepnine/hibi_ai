use std::path::Path;
use anyhow::Result;
use serde_json::Value;

use crate::component::HookConfig;

/// Read settings.json from dest_dir, returning empty object if file doesn't exist.
fn read_settings(dest_dir: &Path) -> Result<Value> {
    let settings_path = dest_dir.join("settings.json");
    if settings_path.exists() {
        let content = std::fs::read_to_string(&settings_path)?;
        Ok(serde_json::from_str(&content)?)
    } else {
        Ok(serde_json::json!({}))
    }
}

/// Write settings.json to dest_dir, creating parent directories if needed.
fn write_settings(dest_dir: &Path, settings: &Value) -> Result<()> {
    let settings_path = dest_dir.join("settings.json");
    if let Some(parent) = settings_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let output = serde_json::to_string_pretty(settings)?;
    std::fs::write(&settings_path, output)?;
    Ok(())
}

pub fn set_output_style(dest_dir: &Path, style_name: &str) -> Result<()> {
    let mut settings = read_settings(dest_dir)?;
    settings["outputStyle"] = serde_json::json!(style_name);
    write_settings(dest_dir, &settings)
}

pub fn set_statusline(dest_dir: &Path, script_name: &str) -> Result<()> {
    let mut settings = read_settings(dest_dir)?;

    // Windows doesn't support ~ expansion, use absolute path
    let statusline_path = if cfg!(windows) {
        dest_dir.join("statusline").join(script_name)
            .to_string_lossy()
            .to_string()
    } else {
        format!("~/.claude/statusline/{}", script_name)
    };
    settings["statusLine"] = serde_json::json!({
        "type": "command",
        "command": statusline_path,
        "padding": 0
    });

    write_settings(dest_dir, &settings)
}

pub fn unset_output_style(dest_dir: &Path) -> Result<()> {
    let settings_path = dest_dir.join("settings.json");
    if !settings_path.exists() {
        return Ok(());
    }

    let mut settings = read_settings(dest_dir)?;
    if let Value::Object(ref mut map) = settings {
        map.remove("outputStyle");
    }
    write_settings(dest_dir, &settings)
}

pub fn unset_statusline(dest_dir: &Path) -> Result<()> {
    let settings_path = dest_dir.join("settings.json");
    if !settings_path.exists() {
        return Ok(());
    }

    let mut settings = read_settings(dest_dir)?;
    if let Value::Object(ref mut map) = settings {
        map.remove("statusLine");
    }
    write_settings(dest_dir, &settings)
}

pub(super) fn merge_settings_json(source: &Path, dest: &Path) -> Result<()> {
    let source_content = std::fs::read_to_string(source)?;
    let source_json: Value = serde_json::from_str(&source_content)?;

    let merged = if dest.exists() {
        let dest_content = std::fs::read_to_string(dest)?;
        let mut dest_json: Value = serde_json::from_str(&dest_content)?;

        // Deep merge source into dest
        merge_json_values(&mut dest_json, &source_json);
        dest_json
    } else {
        source_json
    };

    // Create parent directory if needed
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
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
                        // Special handling for hooks array - append instead of replace
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
                        // Append source hooks that don't already exist
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

/// Check if a hook with the given command path already exists in the event array.
fn hook_exists_in_array(arr: &[Value], hook_command: &str) -> bool {
    arr.iter().any(|item| {
        item.get("hooks")
            .and_then(|h| h.as_array())
            .map(|hooks_arr| {
                hooks_arr.iter().any(|hook| {
                    hook.get("command")
                        .and_then(|c| c.as_str())
                        .map(|cmd| cmd == hook_command)
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false)
    })
}

/// Build a hook entry JSON value from HookConfig.
fn build_hook_entry(config: &HookConfig, hook_command: &str) -> Value {
    let mut hook_entry = serde_json::json!({
        "type": config.hook_type,
        "command": hook_command
    });
    if let Some(timeout) = config.timeout {
        hook_entry["timeout"] = serde_json::json!(timeout);
    }
    serde_json::json!({
        "hooks": [hook_entry]
    })
}

pub(super) fn register_hook_in_settings(dest_dir: &Path, config: &HookConfig) -> Result<()> {
    let hook_command = config.hook_command_path(dest_dir);
    let mut settings = read_settings(dest_dir)?;

    // Ensure hooks object and event array exist
    if settings.get("hooks").is_none() {
        settings["hooks"] = serde_json::json!({});
    }
    let event_name = &config.event;
    let hooks = settings.get_mut("hooks").unwrap();
    if hooks.get(event_name).is_none() {
        hooks[event_name] = serde_json::json!([]);
    }

    let event_hooks = hooks.get_mut(event_name).unwrap();
    if let Value::Array(arr) = event_hooks {
        if !hook_exists_in_array(arr, &hook_command) {
            arr.push(build_hook_entry(config, &hook_command));
        }
    }

    write_settings(dest_dir, &settings)
}

pub(super) fn unregister_hook_from_settings(dest_dir: &Path, config: &HookConfig) -> Result<()> {
    let settings_path = dest_dir.join("settings.json");
    if !settings_path.exists() {
        return Ok(());
    }

    let mut settings = read_settings(dest_dir)?;

    // Get hooks object
    let hooks = match settings.get_mut("hooks") {
        Some(Value::Object(h)) => h,
        _ => return Ok(()),
    };

    // Get event array
    let event_name = &config.event;
    let event_hooks = match hooks.get_mut(event_name) {
        Some(Value::Array(arr)) => arr,
        _ => return Ok(()),
    };

    // Remove hook entries that match config.name
    event_hooks.retain(|item| {
        !item.get("hooks")
            .and_then(|h| h.as_array())
            .map(|hooks_arr| {
                hooks_arr.iter().any(|hook| {
                    hook.get("command")
                        .and_then(|c| c.as_str())
                        .map(|cmd| cmd.contains(&config.name))
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false)
    });

    // Clean up empty structures
    if event_hooks.is_empty() {
        hooks.remove(event_name);
    }
    if hooks.is_empty() {
        if let Value::Object(ref mut map) = settings {
            map.remove("hooks");
        }
    }

    write_settings(dest_dir, &settings)
}

/// Removes installer-managed sections from settings.json
/// This includes: hooks, outputStyle (if it's a known installed style), statusLine (if it's a known installed statusline)
/// Preserves user settings like env, model, enabledPlugins, etc.
pub fn remove_managed_settings_sections(dest_dir: &Path) -> Result<()> {
    let settings_path = dest_dir.join("settings.json");
    if !settings_path.exists() {
        return Ok(());
    }

    let mut settings = read_settings(dest_dir)?;
    if let Value::Object(ref mut map) = settings {
        map.remove("hooks");
        map.remove("outputStyle");
        map.remove("statusLine");
    }
    write_settings(dest_dir, &settings)
}

/// Auto-register an output style in settings.json if no style is currently set
pub(super) fn register_output_style_in_settings(dest_dir: &Path, style_name: &str) -> Result<()> {
    let mut settings = read_settings(dest_dir)?;

    // Only set if outputStyle is not already configured
    if settings.get("outputStyle").is_none() {
        let style_name = style_name.strip_suffix(".md").unwrap_or(style_name);
        settings["outputStyle"] = serde_json::json!(style_name);
        write_settings(dest_dir, &settings)?;
    }

    Ok(())
}

/// Auto-register a statusline in settings.json if no statusline is currently set
pub(super) fn register_statusline_in_settings(dest_dir: &Path, statusline_name: &str) -> Result<()> {
    let mut settings = read_settings(dest_dir)?;

    // Only set if statusLine is not already configured
    if settings.get("statusLine").is_none() {
        settings["statusLine"] = serde_json::json!(statusline_name);
        write_settings(dest_dir, &settings)?;
    }

    Ok(())
}
