use anyhow::Result;
use serde_json::Value;
use std::path::Path;

use crate::component::HookConfig;

/// Convert a Windows path to MSYS-style path for Git Bash compatibility.
/// `C:\Users\...` -> `/c/Users/...`, handles any drive letter.
#[allow(dead_code)]
fn to_msys_path(path: &Path) -> String {
    let s = path.to_string_lossy().replace('\\', "/");
    if s.len() >= 3 && s.as_bytes()[1] == b':' && s.as_bytes()[0].is_ascii_alphabetic() {
        let drive = (s.as_bytes()[0] as char).to_ascii_lowercase();
        format!("/{}{}", drive, &s[2..])
    } else {
        s
    }
}

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
    settings["statusLine"] = serde_json::json!({
        "type": "command",
        "command": statusline_command(dest_dir, script_name)
    });

    write_settings(dest_dir, &settings)
}

fn statusline_command(dest_dir: &Path, script_name: &str) -> String {
    // Windows: convert to MSYS-style path for Git Bash compatibility
    // macOS/Linux: use ~ shorthand derived from dest_dir
    if cfg!(windows) {
        let abs_path = dest_dir.join("statusline").join(script_name);
        let unix_path = to_msys_path(&abs_path);
        format!("cat | {}", unix_path)
    } else {
        let dir_name = dest_dir
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| ".claude".to_string());
        format!("~/{}/statusline/{}", dir_name, script_name)
    }
}

pub(super) fn repair_bundled_statusline(
    settings: &mut Value,
    dest_dir: &Path,
    script_name: &str,
) -> bool {
    const BUNDLED_NAMES: [&str; 3] = ["statusline_macos", "statusline_linux", "statusline.exe"];
    if !BUNDLED_NAMES.contains(&script_name) || settings["statusLine"]["type"] != "command" {
        return false;
    }
    let Some(command) = settings["statusLine"]["command"].as_str() else {
        return false;
    };
    let expected = statusline_command(dest_dir, script_name);
    if command == expected
        || !BUNDLED_NAMES
            .iter()
            .any(|name| command == statusline_command(dest_dir, name))
    {
        return false;
    }
    // Match complete installer-generated commands so custom scripts and arguments survive.
    settings["statusLine"]["command"] = Value::String(expected);
    true
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

/// Remove `outputStyle` from settings.json ONLY if it currently points at
/// `style_name`. Used during remove flow so that deleting an unrelated style
/// file doesn't clear the user's chosen default.
///
/// `style_name` may carry a `.md` suffix (component name) or not (settings
/// value); both forms compare equal after stripping.
pub(super) fn unregister_output_style_if_matches(dest_dir: &Path, style_name: &str) -> Result<()> {
    let settings_path = dest_dir.join("settings.json");
    if !settings_path.exists() {
        return Ok(());
    }

    let mut settings = read_settings(dest_dir)?;
    let target = style_name.strip_suffix(".md").unwrap_or(style_name);
    let current = settings.get("outputStyle").and_then(|v| v.as_str());

    if current == Some(target) {
        if let Value::Object(ref mut map) = settings {
            map.remove("outputStyle");
        }
        write_settings(dest_dir, &settings)?;
    }
    Ok(())
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

pub(crate) fn unregister_hook_from_settings(dest_dir: &Path, config: &HookConfig) -> Result<()> {
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
        !item
            .get("hooks")
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

/// Preserve custom commands while repairing a bundled command from another platform.
pub(super) fn register_statusline_in_settings(
    dest_dir: &Path,
    statusline_name: &str,
) -> Result<()> {
    let mut settings = read_settings(dest_dir)?;

    // Earlier installers wrote a bare string here, and `read_current_settings`
    // only reads an object's `command`, so a string counts as unset and is replaced.
    if !settings.get("statusLine").is_some_and(Value::is_object) {
        set_statusline(dest_dir, statusline_name)?;
    } else if repair_bundled_statusline(&mut settings, dest_dir, statusline_name) {
        write_settings(dest_dir, &settings)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_dest(label: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("hibi_settings_{label}_{nanos}"));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn read_output_style(dest: &Path) -> Option<String> {
        let path = dest.join("settings.json");
        let raw = std::fs::read_to_string(&path).ok()?;
        let v: Value = serde_json::from_str(&raw).ok()?;
        v.get("outputStyle")
            .and_then(|x| x.as_str())
            .map(String::from)
    }

    #[test]
    fn unregister_clears_when_current_matches() {
        let dest = unique_dest("clears");
        std::fs::write(
            dest.join("settings.json"),
            r#"{"outputStyle":"hibi_default"}"#,
        )
        .unwrap();

        // Component name carries .md; settings value does not -- both forms match.
        unregister_output_style_if_matches(&dest, "hibi_default.md").unwrap();

        assert!(
            read_output_style(&dest).is_none(),
            "matching style should be cleared"
        );

        let _ = std::fs::remove_dir_all(&dest);
    }

    #[test]
    fn unregister_preserves_when_current_differs() {
        let dest = unique_dest("preserves");
        std::fs::write(dest.join("settings.json"), r#"{"outputStyle":"important"}"#).unwrap();

        // Removing an unrelated style must NOT clear the user's chosen default.
        unregister_output_style_if_matches(&dest, "other.md").unwrap();

        assert_eq!(read_output_style(&dest).as_deref(), Some("important"));

        let _ = std::fs::remove_dir_all(&dest);
    }

    #[test]
    fn unregister_is_noop_without_settings_file() {
        let dest = unique_dest("nofile");
        // No settings.json.
        unregister_output_style_if_matches(&dest, "anything.md").unwrap();
        assert!(
            !dest.join("settings.json").exists(),
            "must not create settings.json"
        );

        let _ = std::fs::remove_dir_all(&dest);
    }

    #[test]
    fn unregister_is_noop_when_output_style_absent() {
        let dest = unique_dest("absent");
        std::fs::write(dest.join("settings.json"), r#"{"theme":"dark"}"#).unwrap();

        unregister_output_style_if_matches(&dest, "anything.md").unwrap();

        // Other keys must remain intact.
        let raw = std::fs::read_to_string(dest.join("settings.json")).unwrap();
        let v: Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(v.get("theme").and_then(|x| x.as_str()), Some("dark"));
        assert!(v.get("outputStyle").is_none());

        let _ = std::fs::remove_dir_all(&dest);
    }

    /// Claude Code and this installer's own Statusline tab both read
    /// `statusLine.command`, so an auto-registered statusline must be the same
    /// `{type, command}` object the `s` key writes, not a bare file name.
    #[test]
    fn auto_registered_statusline_is_a_command_object() {
        let dest = unique_dest("statusline");
        std::fs::write(dest.join("settings.json"), "{}").unwrap();

        register_statusline_in_settings(&dest, "statusline.exe").unwrap();

        let raw = std::fs::read_to_string(dest.join("settings.json")).unwrap();
        let v: Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(v["statusLine"]["type"], "command", "{v}");
        let command = v["statusLine"]["command"].as_str().unwrap_or_default();
        assert!(command.ends_with("statusline/statusline.exe"), "{v}");

        let _ = std::fs::remove_dir_all(&dest);
    }

    #[test]
    fn auto_registration_replaces_a_bare_string_left_by_older_installs() {
        let dest = unique_dest("statusline_repair");
        std::fs::write(
            dest.join("settings.json"),
            r#"{"statusLine":"statusline.exe"}"#,
        )
        .unwrap();

        register_statusline_in_settings(&dest, "statusline.exe").unwrap();

        let raw = std::fs::read_to_string(dest.join("settings.json")).unwrap();
        let v: Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(v["statusLine"]["type"], "command", "{v}");
        let command = v["statusLine"]["command"].as_str().unwrap_or_default();
        assert!(command.ends_with("statusline/statusline.exe"), "{v}");

        let _ = std::fs::remove_dir_all(&dest);
    }

    #[test]
    fn auto_registration_keeps_a_statusline_the_user_already_set() {
        let dest = unique_dest("statusline_keep");
        let mine = r#"{"statusLine":{"type":"command","command":"~/bin/my-line"}}"#;
        std::fs::write(dest.join("settings.json"), mine).unwrap();

        register_statusline_in_settings(&dest, "statusline.exe").unwrap();

        let raw = std::fs::read_to_string(dest.join("settings.json")).unwrap();
        let v: Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(v["statusLine"]["command"], "~/bin/my-line", "{v}");

        let _ = std::fs::remove_dir_all(&dest);
    }

    #[test]
    #[cfg(unix)]
    fn linux_registration_repairs_the_bundled_macos_command() {
        let root = unique_dest("linux_statusline");
        let dest = root.join(".claude");
        std::fs::create_dir_all(&dest).unwrap();
        std::fs::write(dest.join("settings.json"), r#"{"statusLine":{"type":"command","command":"~/.claude/statusline/statusline_macos","padding":2},"theme":"dark"}"#).unwrap();

        register_statusline_in_settings(&dest, "statusline_linux").unwrap();

        assert_eq!(
            read_settings(&dest).unwrap(),
            serde_json::json!({
                "statusLine": {"type": "command", "command": "~/.claude/statusline/statusline_linux", "padding": 2},
                "theme": "dark"
            })
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn registration_preserves_custom_commands_with_bundled_names() {
        let dest = unique_dest("custom_statusline");
        for command in [
            "~/bin/statusline_macos",
            "~/.claude/statusline/statusline_macos --custom",
            "echo statusline_macos",
        ] {
            let settings =
                serde_json::json!({"statusLine": {"type": "command", "command": command}});
            write_settings(&dest, &settings).unwrap();
            register_statusline_in_settings(&dest, "statusline_linux").unwrap();
            assert_eq!(read_settings(&dest).unwrap(), settings);
        }
        std::fs::remove_dir_all(dest).unwrap();
    }

    #[test]
    fn bundled_repair_leaves_unrelated_or_current_settings_intact() {
        let dest = Path::new(".claude");
        for (settings, script) in [
            (
                serde_json::json!({"statusLine": {"type": "command", "command": "~/.claude/statusline/statusline_macos"}}),
                "custom.sh",
            ),
            (
                serde_json::json!({"statusLine": {"type": "custom", "command": "~/.claude/statusline/statusline_macos"}}),
                "statusline_linux",
            ),
            (
                serde_json::json!({"statusLine": {"type": "command"}}),
                "statusline_linux",
            ),
            (
                serde_json::json!({"statusLine": {"type": "command", "command": statusline_command(dest, "statusline_linux")}}),
                "statusline_linux",
            ),
        ] {
            let mut result = settings.clone();
            assert!(!repair_bundled_statusline(&mut result, dest, script));
            assert_eq!(result, settings);
        }
        let mut foreign = serde_json::json!({"statusLine": {"type": "command", "command": statusline_command(dest, "statusline_macos")}});
        assert!(repair_bundled_statusline(
            &mut foreign,
            dest,
            "statusline_linux"
        ));
        assert_eq!(
            foreign["statusLine"]["command"],
            statusline_command(dest, "statusline_linux")
        );
    }

    #[test]
    fn registration_rejects_invalid_json_without_overwriting_it() {
        let dest = unique_dest("invalid_statusline_json");
        std::fs::write(dest.join("settings.json"), "{invalid").unwrap();
        let error = register_statusline_in_settings(&dest, "statusline_linux").unwrap_err();
        assert!(
            error.to_string().contains("key must be a string"),
            "{error}"
        );
        assert_eq!(
            std::fs::read_to_string(dest.join("settings.json")).unwrap(),
            "{invalid"
        );
        std::fs::remove_dir_all(dest).unwrap();
    }
}
