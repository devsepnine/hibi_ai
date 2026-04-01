use std::path::Path;
use anyhow::Result;
use serde_json::Value;

/// Merge source settings.json into dest, with deep merge and hook append logic.
pub(super) fn merge_settings_json(source: &Path, dest: &Path) -> Result<()> {
    let source_content = std::fs::read_to_string(source)?;
    let source_json: Value = serde_json::from_str(&source_content)?;

    let merged = if dest.exists() {
        let dest_content = std::fs::read_to_string(dest)?;
        let mut dest_json: Value = serde_json::from_str(&dest_content)?;
        merge_json_values(&mut dest_json, &source_json);
        dest_json
    } else {
        source_json
    };

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
