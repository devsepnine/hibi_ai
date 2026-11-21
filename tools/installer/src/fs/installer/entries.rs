use std::fmt::Write;
use std::path::Path;

use anyhow::{Context, Result, ensure};
use serde_json::{Map, Value};

use super::settings::{edit_settings_file, read_settings_file, repair_bundled_statusline};
use crate::component::{Component, InstallStatus, SettingEntry, statusline_binary_name};

pub(super) fn read_source_settings(path: &Path) -> Result<Value> {
    let data = std::fs::read(path)
        .with_context(|| format!("Cannot read source settings in {}", path.display()))?;
    let settings: Value = serde_json::from_slice(&data)
        .with_context(|| format!("Invalid JSON in {}", path.display()))?;
    ensure!(
        settings.is_object(),
        "Expected an object in {}",
        path.display()
    );
    Ok(settings)
}

pub(crate) fn scan_setting_entries(file: &Component) -> Result<Vec<Component>> {
    let mut source = read_source_settings(&file.source_path)?;
    if let Some(parent) = file.dest_path.parent() {
        let binary = statusline_binary_name();
        if parent.join("statusline").join(binary).is_file() {
            repair_bundled_statusline(&mut source, parent, binary);
        }
    }
    let current = read_settings_object(&file.dest_path);
    let mut leaves = Vec::new();
    collect_leaves(&source, &mut Vec::new(), &mut leaves);
    Ok(leaves
        .into_iter()
        .map(|(keys, value)| {
            let mut entry = file.clone();
            entry.name = format!("settings.json/{}", key_label(&keys));
            let mut setting = SettingEntry {
                keys,
                value,
                current: None,
            };
            entry.status = read_entry_state(&mut setting, &current);
            entry.with_setting_entry(setting)
        })
        .collect())
}

pub(crate) fn refresh_setting_entries(components: &mut [Component], dest_dir: &Path) -> bool {
    let mut entries = components
        .iter_mut()
        .filter(|entry| entry.setting_entry.is_some())
        .peekable();
    if entries.peek().is_none() {
        return false;
    }
    let settings = read_settings_object(&dest_dir.join("settings.json"));
    for entry in entries {
        if let Some(setting) = &mut entry.setting_entry {
            entry.status = read_entry_state(setting, &settings);
        }
    }
    true
}

fn read_settings_object(path: &Path) -> Result<Value> {
    let settings = read_settings_file(path)?;
    ensure!(
        settings.is_object(),
        "Expected an object in {}",
        path.display()
    );
    Ok(settings)
}

fn read_entry_state(setting: &mut SettingEntry, settings: &Result<Value>) -> InstallStatus {
    setting.current = None;
    if is_statusline(&setting.keys)
        && (setting.value["type"] != "command" || setting.value["command"].as_str().is_none())
    {
        return InstallStatus::Blocked(
            "Source statusLine requires type: command and a command string".into(),
        );
    }
    let current = match settings {
        Ok(settings) => value_at(settings, &setting.keys),
        Err(error) => return InstallStatus::Blocked(format!("{error:#}")),
    };
    match current {
        Ok(Some(value)) => {
            setting.current = Some(value.clone());
            if value == &setting.value {
                InstallStatus::Unchanged
            } else {
                InstallStatus::Modified
            }
        }
        Ok(None) => InstallStatus::New,
        Err(error) => InstallStatus::Blocked(error.to_string()),
    }
}

fn is_statusline(path: &[String]) -> bool {
    path == ["statusLine"]
}

fn collect_leaves(value: &Value, path: &mut Vec<String>, leaves: &mut Vec<(Vec<String>, Value)>) {
    if let Some(object) = value.as_object()
        && !is_statusline(path)
        && (path.is_empty() || !object.is_empty())
    {
        for (key, value) in object {
            path.push(key.clone());
            collect_leaves(value, path, leaves);
            path.pop();
        }
    } else {
        leaves.push((path.clone(), value.clone()));
    }
}

fn key_label(keys: &[String]) -> String {
    let mut label = String::new();
    for (index, key) in keys.iter().enumerate() {
        if index > 0 {
            label.push('.');
        }
        if key.is_empty() {
            label.push('%');
        }
        // Escape punctuation the component tree would interpret as path separators.
        for character in key.chars() {
            if matches!(character, '.' | '/' | '\\' | '%') || character.is_control() {
                for byte in character.encode_utf8(&mut [0; 4]).bytes() {
                    let _ = write!(label, "%{byte:02X}");
                }
            } else {
                label.push(character);
            }
        }
    }
    label
}

fn value_at<'a>(mut settings: &'a Value, keys: &[String]) -> Result<Option<&'a Value>> {
    for key in keys {
        let object = settings
            .as_object()
            .context("An enclosing setting must be an object")?;
        let Some(value) = object.get(key) else {
            return Ok(None);
        };
        settings = value;
    }
    Ok(Some(settings))
}

pub(super) fn install_setting(component: &Component, setting: &SettingEntry) -> Result<()> {
    let (key, parents) = setting.keys.split_last().context("Missing setting key")?;
    edit_settings_file(&component.dest_path, |settings| {
        let current = value_at(settings, &setting.keys)?;
        if current == Some(&setting.value) {
            return Ok(false);
        }
        ensure!(
            current == setting.current.as_ref(),
            "Setting {} changed since it was listed. Review it and try again",
            component.name
        );
        ensure!(
            is_statusline(&setting.keys)
                || !current
                    .and_then(Value::as_object)
                    .is_some_and(|object| !object.is_empty()),
            "Cannot replace the object at {} with a single setting",
            component.name
        );
        let mut parent = settings
            .as_object_mut()
            .context("Expected settings object")?;
        for key in parents {
            parent = parent
                .entry(key)
                .or_insert_with(|| Value::Object(Map::new()))
                .as_object_mut()
                .with_context(|| {
                    format!(
                        "Cannot set {} because an enclosing key is not an object",
                        component.name
                    )
                })?;
        }
        parent.insert(key.clone(), setting.value.clone());
        Ok(true)
    })
}

pub(super) fn remove_setting(component: &Component, setting: &SettingEntry) -> Result<()> {
    let (key, parents) = setting.keys.split_last().context("Missing setting key")?;
    if !component.dest_path.try_exists()? {
        return Ok(());
    }
    edit_settings_file(&component.dest_path, |settings| {
        let Some(current) = value_at(settings, &setting.keys)? else {
            return Ok(false);
        };
        ensure!(
            *current == setting.value,
            "Setting {} changed or differs from its source; it was kept",
            component.name
        );
        let parent = parents
            .iter()
            .fold(settings, |node, key| &mut node[key.as_str()]);
        let parent = parent
            .as_object_mut()
            .context("Expected a setting object")?;
        parent.remove(key);
        Ok(true)
    })
}

pub(crate) fn preview_setting(component: &Component, setting: &SettingEntry) -> String {
    let mut output = format!(
        "--- {}\n+++ {}\n@@ {} @@\n",
        component.dest_path.display(),
        component.source_path.display(),
        component.name
    );
    if setting.current.as_ref() == Some(&setting.value) {
        for line in format!("{:#}", setting.value).lines() {
            let _ = writeln!(output, " {line}");
        }
    } else {
        if let Some(current) = &setting.current {
            for line in format!("{current:#}").lines() {
                let _ = writeln!(output, "-{line}");
            }
        }
        for line in format!("{:#}", setting.value).lines() {
            let _ = writeln!(output, "+{line}");
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::ComponentType;

    fn config(fixture: &tempfile::TempDir, source: &str) -> Result<Component> {
        let source_path = fixture.path().join("source.json");
        std::fs::write(&source_path, source)?;
        Ok(Component::new(
            ComponentType::ConfigFile,
            "settings.json".into(),
            source_path,
            fixture.path().join("settings.json"),
            InstallStatus::New,
        ))
    }

    #[test]
    fn install_rejects_incompatible_objects_and_malformed_documents() -> Result<()> {
        let fixture = tempfile::tempdir()?;
        let file = config(&fixture, r#"{"env":{"TIMEOUT":10},"theme":"dark"}"#)?;
        let original = r#"{"env":"custom","theme":{"user":"light"}}"#;
        std::fs::write(&file.dest_path, original)?;
        for row in scan_setting_entries(&file)? {
            assert!(super::super::install_component(&row, fixture.path(), fixture.path()).is_err());
        }
        assert_eq!(std::fs::read_to_string(&file.dest_path)?, original);

        std::fs::write(&file.dest_path, "{broken")?;
        let row = &scan_setting_entries(&file)?[0];
        assert_eq!(row.status.display(), "blocked");
        assert!(!row.is_install_eligible());
        assert!(!row.is_remove_eligible());
        let error =
            super::super::install_component(row, fixture.path(), fixture.path()).unwrap_err();
        assert!(format!("{error:#}").contains("Invalid JSON"));
        assert_eq!(std::fs::read_to_string(&file.dest_path)?, "{broken");
        Ok(())
    }

    #[test]
    fn arrays_are_single_entries_and_literal_keys_are_distinct() -> Result<()> {
        let fixture = tempfile::tempdir()?;
        let file = config(&fixture, r#"{"env":{"A":[1,2]},"env.A":["literal"]}"#)?;
        let rows = scan_setting_entries(&file)?;
        assert_eq!(rows[0].name, "settings.json/env.A");
        assert_eq!(rows[1].name, "settings.json/env%2EA");
        assert!(rows[0].is_install_eligible());
        assert!(!rows[0].is_remove_eligible());
        super::super::remove_component(&rows[0], fixture.path())?;
        assert!(!file.dest_path.exists());
        super::super::install_component(&rows[0], fixture.path(), fixture.path())?;
        assert_eq!(
            read_settings_file(&file.dest_path)?,
            serde_json::json!({"env":{"A":[1,2]}})
        );
        Ok(())
    }

    #[test]
    fn incomplete_source_statusline_is_blocked() -> Result<()> {
        let fixture = tempfile::tempdir()?;
        let file = config(&fixture, r#"{"statusLine":{"command":"./statusline"}}"#)?;
        let rows = scan_setting_entries(&file)?;
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].name, "settings.json/statusLine");
        assert!(!rows[0].is_install_eligible());
        assert!(
            rows[0]
                .status
                .blocking_reason()
                .unwrap()
                .contains("statusLine requires")
        );
        assert!(super::super::install_component(&rows[0], fixture.path(), fixture.path()).is_err());
        assert!(!file.dest_path.exists());
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn settings_entry_uses_installed_native_statusline_command() -> Result<()> {
        let fixture = tempfile::tempdir()?;
        let native = statusline_binary_name();
        let foreign = if native == "statusline_macos" {
            "statusline_linux"
        } else {
            "statusline_macos"
        };
        let home_config = fixture.path().file_name().unwrap().to_string_lossy();
        let old = format!("~/{home_config}/statusline/{foreign}");
        let expected = format!("~/{home_config}/statusline/{native}");
        let file = config(
            &fixture,
            &serde_json::json!({"statusLine":{"type":"command","command":old}}).to_string(),
        )?;
        std::fs::create_dir(fixture.path().join("statusline"))?;
        std::fs::write(fixture.path().join("statusline").join(native), "binary")?;
        let rows = scan_setting_entries(&file)?;
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].name, "settings.json/statusLine");
        std::fs::write(&file.dest_path, br#"{"model":"custom"}"#)?;
        super::super::install_component(&rows[0], fixture.path(), fixture.path())?;
        assert_eq!(
            read_settings_file(&file.dest_path)?,
            serde_json::json!({"model":"custom","statusLine":{"type":"command","command":expected}})
        );
        let rows = scan_setting_entries(&file)?;
        super::super::remove_component(&rows[0], fixture.path())?;
        assert_eq!(
            read_settings_file(&file.dest_path)?,
            serde_json::json!({"model":"custom"})
        );
        Ok(())
    }
}
