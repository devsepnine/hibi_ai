use anyhow::{Context, Result};
use serde_json::Value;
use std::fs::{self, OpenOptions};
use std::io::{ErrorKind, Write};
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

pub(super) fn read_settings_file(path: &Path) -> Result<Value> {
    match fs::read_to_string(path) {
        Ok(content) => serde_json::from_str(&content)
            .with_context(|| format!("Invalid JSON in {}", path.display())),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(serde_json::json!({})),
        Err(error) => Err(error).with_context(|| format!("Cannot read {}", path.display())),
    }
}

fn edit_settings(dest_dir: &Path, edit: impl FnOnce(&mut Value) -> Result<bool>) -> Result<()> {
    edit_settings_file(&dest_dir.join("settings.json"), edit)
}

pub(super) fn edit_settings_file(
    path: &Path,
    edit: impl FnOnce(&mut Value) -> Result<bool>,
) -> Result<()> {
    let target = super::resolve_write_target(path)?;
    let parent = target
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    fs::create_dir_all(parent)?;
    let target = parent
        .canonicalize()?
        .join(target.file_name().context("Missing settings filename")?);
    let mut lock_name = target
        .file_name()
        .context("Missing settings filename")?
        .to_os_string();
    lock_name.push(".hibi.lock");
    let lock = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(target.with_file_name(lock_name))
        .context("Cannot open settings lock")?;
    lock.lock().context("Cannot lock settings")?;
    let mut settings = read_settings_file(&target)?;
    anyhow::ensure!(
        settings.is_object(),
        "Expected an object in {}",
        path.display()
    );
    if edit(&mut settings)? {
        write_settings_file(&target, &settings)?;
    }
    Ok(())
}

fn write_settings_file(path: &Path, settings: &Value) -> Result<()> {
    let existing =
        match fs::metadata(path) {
            Ok(metadata) => {
                anyhow::ensure!(
                    metadata.is_file() && !metadata.permissions().readonly(),
                    "Settings file is not writable: {}",
                    path.display()
                );
                // A directory rename can otherwise bypass the target file's effective access mode.
                drop(OpenOptions::new().write(true).open(path).with_context(|| {
                    format!("Settings file is not writable: {}", path.display())
                })?);
                Some(metadata)
            }
            Err(error) if error.kind() == ErrorKind::NotFound => None,
            Err(error) => {
                return Err(error).with_context(|| format!("Cannot inspect {}", path.display()));
            }
        };
    let parent = path
        .parent()
        .context("Settings file has no parent directory")?;
    let mut staged = tempfile::NamedTempFile::new_in(parent)?;
    staged.write_all(&serde_json::to_vec_pretty(settings)?)?;
    if let Some(existing) = existing {
        #[cfg(target_os = "linux")]
        preserve_ownership(&existing, staged.as_file())
            .with_context(|| format!("Cannot preserve ownership for {}", path.display()))?;
        staged.as_file().set_permissions(existing.permissions())?;
        #[cfg(target_os = "linux")]
        preserve_access_acl(path, staged.as_file())
            .with_context(|| format!("Cannot preserve access ACL for {}", path.display()))?;
    }
    staged.as_file().sync_all()?;
    staged
        .persist(path)
        .with_context(|| format!("Cannot replace {}", path.display()))?;
    Ok(())
}

#[cfg(target_os = "linux")]
fn preserve_ownership(existing: &fs::Metadata, staged: &fs::File) -> std::io::Result<()> {
    use std::os::unix::fs::{self, MetadataExt};

    let current = staged.metadata()?;
    let owner = (existing.uid() != current.uid()).then_some(existing.uid());
    let group = (existing.gid() != current.gid()).then_some(existing.gid());
    if owner.is_some() || group.is_some() {
        fs::fchown(staged, owner, group)?;
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn preserve_access_acl(path: &Path, staged: &fs::File) -> std::io::Result<()> {
    use xattr::FileExt;
    const ACCESS_ACL: &str = "system.posix_acl_access";

    match xattr::get(path, ACCESS_ACL) {
        Ok(Some(acl)) => staged.set_xattr(ACCESS_ACL, &acl),
        Ok(None) => {
            // A temporary file may have inherited an ACL from its parent.
            if staged.get_xattr(ACCESS_ACL)?.is_some() {
                staged.remove_xattr(ACCESS_ACL)?;
            }
            Ok(())
        }
        Err(error) if error.kind() == ErrorKind::Unsupported => Ok(()),
        Err(error) => Err(error),
    }
}

pub fn set_output_style(dest_dir: &Path, style_name: &str) -> Result<()> {
    edit_settings(dest_dir, |settings| {
        settings["outputStyle"] = style_name.into();
        Ok(true)
    })
}

pub fn set_statusline(dest_dir: &Path, script_name: &str) -> Result<()> {
    edit_settings(dest_dir, |settings| {
        settings["statusLine"] = statusline_setting(dest_dir, script_name);
        Ok(true)
    })
}

fn statusline_setting(dest_dir: &Path, script_name: &str) -> Value {
    serde_json::json!({
        "type": "command",
        "command": statusline_command(dest_dir, script_name)
    })
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
    remove_setting_if(dest_dir, "outputStyle", |_| true)
}

pub(super) fn unregister_output_style_if_matches(dest_dir: &Path, style_name: &str) -> Result<()> {
    let target = style_name.strip_suffix(".md").unwrap_or(style_name);
    remove_setting_if(dest_dir, "outputStyle", |style| style == target)
}

pub fn unset_statusline(dest_dir: &Path) -> Result<()> {
    remove_setting_if(dest_dir, "statusLine", |_| true)
}

pub(super) fn unregister_statusline_if_matches(dest_dir: &Path, name: &str) -> Result<()> {
    let command = statusline_command(dest_dir, name);
    remove_setting_if(dest_dir, "statusLine", |status| {
        status
            .as_str()
            .is_some_and(|legacy| legacy == name || legacy == command)
            || (status["type"] == "command" && status["command"] == command)
    })
}

fn remove_setting_if(
    dest_dir: &Path,
    key: &str,
    matches: impl FnOnce(&Value) -> bool,
) -> Result<()> {
    if !dest_dir.join("settings.json").try_exists()? {
        return Ok(());
    }
    edit_settings(dest_dir, |settings| {
        let fields = settings
            .as_object_mut()
            .context("Expected a settings object")?;
        if fields.get(key).is_some_and(matches) {
            fields.remove(key);
            return Ok(true);
        }
        Ok(false)
    })
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
    let command = config.hook_command_path(dest_dir);
    edit_settings(dest_dir, |settings| {
        let fields = settings
            .as_object_mut()
            .context("Expected a settings object")?;
        let hooks = fields
            .entry("hooks")
            .or_insert_with(|| serde_json::json!({}))
            .as_object_mut()
            .context("Expected a hooks object")?;
        let commands = hooks
            .entry(&config.event)
            .or_insert_with(|| serde_json::json!([]))
            .as_array_mut()
            .context("Expected a hook event array")?;
        if hook_exists_in_array(commands, &command) {
            return Ok(false);
        }
        commands.push(build_hook_entry(config, &command));
        Ok(true)
    })
}

pub(crate) fn unregister_hook_from_settings(dest_dir: &Path, config: &HookConfig) -> Result<()> {
    if !dest_dir.join("settings.json").try_exists()? {
        return Ok(());
    }
    let command = config.hook_command_path(dest_dir);
    edit_settings(dest_dir, |settings| {
        let fields = settings
            .as_object_mut()
            .context("Expected a settings object")?;
        let Some(hooks) = fields.get_mut("hooks").and_then(Value::as_object_mut) else {
            return Ok(false);
        };
        let Some(event_hooks) = hooks.get_mut(&config.event).and_then(Value::as_array_mut) else {
            return Ok(false);
        };
        let mut changed = false;
        event_hooks.retain_mut(|group| {
            let Some(commands) = group.get_mut("hooks").and_then(Value::as_array_mut) else {
                return true;
            };
            let before = commands.len();
            commands.retain(|hook| hook["type"] != config.hook_type || hook["command"] != command);
            let removed = before != commands.len();
            changed |= removed;
            !removed || !commands.is_empty()
        });
        if changed && event_hooks.is_empty() {
            hooks.remove(&config.event);
            if hooks.is_empty() {
                fields.remove("hooks");
            }
        }
        Ok(changed)
    })
}

pub(super) fn register_output_style_in_settings(dest_dir: &Path, style_name: &str) -> Result<()> {
    edit_settings(dest_dir, |settings| {
        if settings.get("outputStyle").is_some() {
            return Ok(false);
        }
        settings["outputStyle"] = style_name.strip_suffix(".md").unwrap_or(style_name).into();
        Ok(true)
    })
}

/// Preserve custom commands while repairing a bundled command from another platform.
pub(super) fn register_statusline_in_settings(
    dest_dir: &Path,
    statusline_name: &str,
) -> Result<()> {
    edit_settings(dest_dir, |settings| {
        // Earlier installers wrote a bare string instead of a command object.
        if !settings.get("statusLine").is_some_and(Value::is_object) {
            settings["statusLine"] = statusline_setting(dest_dir, statusline_name);
            return Ok(true);
        }
        Ok(repair_bundled_statusline(
            settings,
            dest_dir,
            statusline_name,
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[cfg(target_os = "linux")]
    fn named_user_acl() -> Vec<u8> {
        // Linux ACL xattr version 2 with a named user and no owning-group access.
        let mut fixture = 2_u32.to_le_bytes().to_vec();
        for (tag, permissions, user) in [
            (0x01_u16, 0x06_u16, u32::MAX),
            (0x02, 0x04, 65534),
            (0x04, 0x00, u32::MAX),
            (0x10, 0x04, u32::MAX),
            (0x20, 0x00, u32::MAX),
        ] {
            fixture.extend(tag.to_le_bytes());
            fixture.extend(permissions.to_le_bytes());
            fixture.extend(user.to_le_bytes());
        }
        fixture
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn settings_replacement_keeps_the_access_acl() -> Result<()> {
        let root = tempfile::tempdir()?;
        let path = root.path().join("settings.json");
        fs::write(&path, b"{}")?;
        const ACL: &str = "system.posix_acl_access";
        let fixture = named_user_acl();
        match xattr::set(&path, ACL, &fixture) {
            Err(error) if error.kind() == ErrorKind::Unsupported => return Ok(()),
            result => result?,
        }
        set_output_style(root.path(), "review")?;
        assert_eq!(xattr::get(&path, ACL)?, Some(fixture));
        assert_eq!(read_settings_file(&path)?["outputStyle"], "review");
        Ok(())
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn settings_replacement_does_not_inherit_a_new_access_acl() -> Result<()> {
        let root = tempfile::tempdir()?;
        let path = root.path().join("settings.json");
        fs::write(&path, b"{}")?;
        match xattr::set(root.path(), "system.posix_acl_default", &named_user_acl()) {
            Err(error) if error.kind() == ErrorKind::Unsupported => return Ok(()),
            result => result?,
        }

        set_output_style(root.path(), "review")?;
        assert_eq!(xattr::get(&path, "system.posix_acl_access")?, None);
        assert_eq!(read_settings_file(&path)?["outputStyle"], "review");
        Ok(())
    }

    #[test]
    fn concurrent_edits_hold_the_lock_until_both_changes_are_saved() -> Result<()> {
        let root = tempfile::tempdir()?;
        let path = root.path().join("settings.json");
        std::thread::scope(|threads| -> Result<()> {
            let (locked, locked_rx) = std::sync::mpsc::channel();
            let (release, release_rx) = std::sync::mpsc::channel();
            let first_path = &path;
            let first = threads.spawn(move || {
                edit_settings_file(first_path, |settings| {
                    locked.send(())?;
                    release_rx.recv()?;
                    settings["outputStyle"] = "review".into();
                    Ok(true)
                })
            });
            locked_rx.recv_timeout(std::time::Duration::from_secs(5))?;
            let probe = OpenOptions::new()
                .read(true)
                .write(true)
                .open(root.path().join("settings.json.hibi.lock"))?;
            let held = probe.try_lock().is_err();
            drop(probe);
            let second = threads.spawn(|| set_statusline(root.path(), "review"));
            release.send(())?;
            first.join().unwrap()?;
            second.join().unwrap()?;
            assert!(held);
            Ok(())
        })?;
        let settings = read_settings_file(&path)?;
        assert_eq!(settings["outputStyle"], "review");
        assert_eq!(settings["statusLine"]["type"], "command");
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn settings_write_creates_a_missing_relative_symlink_target() -> Result<()> {
        let root = tempfile::tempdir()?;
        let path = root.path().join("settings.json");
        std::os::unix::fs::symlink("dotfiles.json", &path)?;
        set_output_style(root.path(), "review")?;
        assert_eq!(fs::read_link(path)?, Path::new("dotfiles.json"));
        assert_eq!(
            read_settings_file(&root.path().join("dotfiles.json"))?["outputStyle"],
            "review"
        );
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn atomic_replace_respects_the_settings_files_effective_permissions() -> Result<()> {
        use std::os::unix::fs::PermissionsExt;
        let root = tempfile::tempdir()?;
        let path = root.path().join("settings.json");
        fs::write(&path, b"{}")?;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o466))?;
        if OpenOptions::new().write(true).open(&path).is_err() {
            assert!(set_output_style(root.path(), "review").is_err());
            assert_eq!(fs::read(&path)?, b"{}");
        }
        Ok(())
    }

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
            read_settings_file(&dest.join("settings.json")).unwrap(),
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
            fs::write(
                dest.join("settings.json"),
                serde_json::to_vec(&settings).unwrap(),
            )
            .unwrap();
            register_statusline_in_settings(&dest, "statusline_linux").unwrap();
            assert_eq!(
                read_settings_file(&dest.join("settings.json")).unwrap(),
                settings
            );
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
        let detail = format!("{error:#}");
        assert!(
            detail.contains("Invalid JSON") && detail.contains("key must be a string"),
            "{detail}"
        );
        assert_eq!(
            std::fs::read_to_string(dest.join("settings.json")).unwrap(),
            "{invalid"
        );
        std::fs::remove_dir_all(dest).unwrap();
    }
}
