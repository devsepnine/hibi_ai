mod entries;
mod mcp;
mod plugin;
#[cfg(feature = "gui")]
pub(crate) mod preferences;
mod process;
mod renamed;
mod settings;
#[cfg(test)]
mod test_support;
mod unshipped;
pub(crate) mod visibility;

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

use crate::component::{Component, ComponentType};
use crate::fs::{create_cli_command, run_with_timeout};
use crate::target::TargetCli;
use settings::{
    register_hook_in_settings, register_output_style_in_settings, register_statusline_in_settings,
    unregister_hook_from_settings, unregister_output_style_if_matches,
    unregister_statusline_if_matches,
};

// Re-export public API
pub(crate) use entries::{preview_setting, refresh_setting_entries, scan_setting_entries};
pub use mcp::{McpInstallConfig, McpRegistration, install_mcp_server, remove_mcp_server};
pub use plugin::{install_plugin, remove_plugin};
pub use renamed::auto_cleanup_renamed_skills;
pub use settings::{set_output_style, set_statusline, unset_output_style, unset_statusline};
pub use unshipped::auto_cleanup_unshipped_files;

/// Timeout for the pre-flight `--version` probe. Long enough to absorb
/// a cold Node.js startup on a slow disk, short enough that a hung CLI
/// doesn't make the installer feel frozen.
const PREFLIGHT_TIMEOUT_SECS: u64 = 8;

/// Verify the target CLI is reachable before queuing per-item operations.
///
/// Plugin and MCP install/remove both shell out to `claude` (or `codex`).
/// Without this check, a missing CLI surfaces as N identical "not found"
/// errors — one per queued item — wrapped in install-flavored language
/// that obscures the real problem. Running `--version` once up front
/// lets us collapse that into a single, actionable status message and
/// abort before the user enters the Installing view.
///
/// # Blocking behaviour
///
/// This call is **synchronous** and may block the caller for up to
/// `PREFLIGHT_TIMEOUT_SECS` seconds. The TUI never calls it directly —
/// `start_preflight_thread` hoists it onto a worker thread so the tick
/// keeps rendering and accepting input while the probe runs. The CLI
/// `--sync` and any future non-TUI callers may invoke this synchronously
/// since they don't share a render loop. The happy path is sub-second
/// (`claude --version` returns immediately) and the missing-CLI path is
/// also immediate (`spawn` returns `ErrorKind::NotFound` without waiting);
/// the only slow path is a CLI that hangs on stdin or work it does at
/// startup — rare in practice, hence the 8-second budget.
pub(crate) fn preflight_cli_available(target_cli: TargetCli) -> Result<()> {
    let mut cmd = create_cli_command(target_cli);
    cmd.args(["--version"]);
    match run_with_timeout(&mut cmd, PREFLIGHT_TIMEOUT_SECS) {
        Ok(out) if out.status.success() => Ok(()),
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            let trimmed = stderr.trim();
            if trimmed.is_empty() {
                anyhow::bail!("CLI exited with status {}", out.status);
            } else {
                anyhow::bail!("CLI reported: {}", trimmed);
            }
        }
        Err(e) => Err(e),
    }
}

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
        if !path.is_dir() || super::is_system_entry(&entry.file_name(), true) {
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
        let config: crate::component::HookConfig = match serde_yaml_bw::from_str(&config_content) {
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

        if remove_registered_file(&dest_path, || {
            unregister_hook_from_settings(dest_dir, &config)
        })
        .is_ok()
        {
            cleaned.push(config.name.clone());
        }
    }

    cleaned
}

pub fn install_component(component: &Component, _source_dir: &Path, dest_dir: &Path) -> Result<()> {
    if let Some(reason) = component.status.blocking_reason() {
        anyhow::bail!("{reason}");
    }
    if let Some(setting) = &component.setting_entry {
        return entries::install_setting(component, setting);
    }
    if component.is_settings_file() {
        entries::read_source_settings(&component.source_path)?;
        anyhow::bail!("Select individual settings to install");
    }
    match &component.component_type {
        ComponentType::Hooks => {
            if let Some(config) = &component.hook_config {
                if config.is_deprecated() {
                    anyhow::bail!(
                        "Hook '{}' is deprecated and cannot be installed",
                        component.name
                    );
                }
                install_registered_file(component, || register_hook_in_settings(dest_dir, config))?;
            } else {
                copy_file(component)?;
            }
        }
        ComponentType::OutputStyles => install_registered_file(component, || {
            register_output_style_in_settings(dest_dir, &component.name)
        })?,
        ComponentType::Statusline => install_registered_file(component, || {
            register_statusline_in_settings(dest_dir, &component.name)
        })?,
        _ => {
            copy_file(component)?;
        }
    }
    Ok(())
}

pub fn remove_component(component: &Component, dest_dir: &Path) -> Result<()> {
    if let Some(reason) = component.status.blocking_reason() {
        anyhow::bail!("{reason}");
    }
    if let Some(setting) = &component.setting_entry {
        return entries::remove_setting(component, setting);
    }
    anyhow::ensure!(
        component.is_remove_eligible(),
        "Select settings to remove individually"
    );
    match &component.component_type {
        ComponentType::Hooks => remove_registered_file(&component.dest_path, || {
            if let Some(config) = &component.hook_config {
                unregister_hook_from_settings(dest_dir, config)?;
            }
            Ok(())
        })?,
        ComponentType::OutputStyles => remove_registered_file(&component.dest_path, || {
            unregister_output_style_if_matches(dest_dir, &component.name)
        })?,
        ComponentType::Statusline => remove_registered_file(&component.dest_path, || {
            unregister_statusline_if_matches(dest_dir, &component.name)
        })?,
        _ => {
            // Remove file
            if component.dest_path.exists() {
                std::fs::remove_file(&component.dest_path)?;
            }
        }
    }
    Ok(())
}

fn lock_registered_files(path: &Path) -> Result<std::fs::File> {
    let parent = component_parent(path)?;
    std::fs::create_dir_all(parent)?;
    // Install and removal need to serialize the file change together with registration.
    let lock = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(parent.join(".hibi-component.lock"))
        .with_context(|| format!("Cannot open component lock in {}", parent.display()))?;
    lock.lock().context("Cannot lock registered components")?;
    Ok(lock)
}

fn resolve_write_target(path: &Path) -> Result<PathBuf> {
    let mut target = path.to_owned();
    // A dotfile link can predate its target.
    for _ in 0..40 {
        match std::fs::symlink_metadata(&target) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                let link = std::fs::read_link(&target)?;
                target = target
                    .parent()
                    .context("File link has no parent")?
                    .join(link);
            }
            Ok(_) => return Ok(target),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(target),
            Err(error) => {
                return Err(error).with_context(|| format!("Cannot inspect {}", target.display()));
            }
        }
    }
    anyhow::bail!("Too many symbolic links at {}", path.display());
}

fn install_registered_file(
    component: &Component,
    register: impl FnOnce() -> Result<()>,
) -> Result<()> {
    let _lock = lock_registered_files(&component.dest_path)?;
    let target = resolve_write_target(&component.dest_path)?;
    let parent = target
        .parent()
        .filter(|dir| !dir.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let staged = tempfile::tempdir_in(parent)?;
    let replacement = staged.path().join("replacement");
    let backup = staged.path().join("previous");
    copy_component_to(component, &replacement)
        .with_context(|| format!("Cannot prepare {}", target.display()))?;

    let had_previous = match std::fs::metadata(&target) {
        Ok(metadata) => {
            anyhow::ensure!(
                metadata.is_file(),
                "Expected a file at {}",
                target.display()
            );
            drop(
                std::fs::OpenOptions::new()
                    .write(true)
                    .open(&target)
                    .with_context(|| format!("Cannot write {}", target.display()))?,
            );
            std::fs::rename(&target, &backup)
                .with_context(|| format!("Cannot back up {}", target.display()))?;
            true
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
        Err(error) => {
            return Err(error).with_context(|| format!("Cannot inspect {}", target.display()));
        }
    };
    let published = std::fs::rename(&replacement, &target)
        .with_context(|| format!("Cannot install {}", target.display()));
    let did_publish = published.is_ok();
    if let Err(error) = published.and_then(|()| register()) {
        let restore = (|| -> std::io::Result<()> {
            if did_publish {
                std::fs::rename(&target, &replacement)?;
            }
            if had_previous {
                std::fs::rename(&backup, &target)?;
            }
            Ok(())
        })();
        if let Err(restore) = restore {
            let staging = staged.keep();
            return Err(error).context(format!(
                "Cannot restore {}: {restore}. Staged files are at {}",
                target.display(),
                staging.display()
            ));
        }
        return Err(error).context("Component installation rolled back");
    }
    Ok(())
}

fn remove_registered_file(path: &Path, unregister: impl FnOnce() -> Result<()>) -> Result<()> {
    let _lock = lock_registered_files(path)?;
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return unregister(),
        Err(error) => {
            return Err(error).with_context(|| format!("Cannot inspect {}", path.display()));
        }
    };
    anyhow::ensure!(
        metadata.is_file() || metadata.file_type().is_symlink(),
        "Expected a file at {}",
        path.display()
    );
    let staged = tempfile::tempdir_in(component_parent(path)?)?;
    let backup = staged.path().join("component");
    std::fs::rename(path, &backup)
        .with_context(|| format!("Cannot stage removal of {}", path.display()))?;
    if let Err(error) = unregister() {
        if let Err(restore) = std::fs::rename(&backup, path) {
            let saved = staged.keep().join("component");
            return Err(error).context(format!(
                "Cannot restore {}: {restore}. Recover it from {}",
                path.display(),
                saved.display()
            ));
        }
        return Err(error).context("Could not update settings; component restored");
    }
    staged
        .close()
        .context("Could not delete staged component file")
}

fn component_parent(path: &Path) -> Result<&Path> {
    anyhow::ensure!(
        !path
            .components()
            .any(|part| matches!(part, std::path::Component::ParentDir)),
        "Security: destination path contains '..' component: {path:?}"
    );
    let parent = path.parent().context("Component file has no parent")?;
    Ok(if parent.as_os_str().is_empty() {
        Path::new(".")
    } else {
        parent
    })
}

fn copy_file(component: &Component) -> Result<()> {
    std::fs::create_dir_all(component_parent(&component.dest_path)?)?;
    copy_component_to(component, &component.dest_path)
}

fn copy_component_to(component: &Component, destination: &Path) -> Result<()> {
    // Copy file
    std::fs::copy(&component.source_path, destination)?;

    // Set executable permission for shell scripts (Unix only).
    // On Windows, .sh scripts are executed via Git Bash; .exe hooks are already executable.
    #[cfg(unix)]
    if component.component_type == ComponentType::Statusline
        || component
            .source_path
            .extension()
            .map_or(false, |e| e == "sh")
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(destination)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(destination, perms)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::{HookConfig, InstallStatus};
    use serde_json::{Value, json};

    #[test]
    fn failed_statusline_registration_preserves_existing_file_and_absence() -> Result<()> {
        let home = tempfile::tempdir()?;
        let settings = home.path().join("settings.json");
        let name = crate::component::statusline_binary_name();
        let binary = home.path().join("statusline").join(name);
        let source = home.path().join("source");
        std::fs::write(&source, b"new statusline")?;
        std::fs::write(&settings, b"{invalid")?;
        let component = Component::new(
            ComponentType::Statusline,
            name.into(),
            source,
            binary.clone(),
            InstallStatus::New,
        );

        let error = install_component(&component, home.path(), home.path()).unwrap_err();
        assert!(format!("{error:#}").contains("Invalid JSON"));
        assert!(!binary.exists());
        std::fs::write(&binary, b"existing statusline")?;
        let error = install_component(&component, home.path(), home.path()).unwrap_err();
        assert!(format!("{error:#}").contains("Invalid JSON"));
        assert_eq!(std::fs::read(&binary)?, b"existing statusline");
        assert_eq!(std::fs::read(&settings)?, b"{invalid");

        std::fs::write(&settings, b"{}")?;
        install_component(&component, home.path(), home.path())?;
        assert_eq!(std::fs::read(&binary)?, b"new statusline");
        let updated: Value = serde_json::from_slice(&std::fs::read(settings)?)?;
        assert_eq!(updated["statusLine"]["type"], "command");
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn statusline_installation_and_rollback_keep_symlinks() -> Result<()> {
        let home = tempfile::tempdir()?;
        let settings = home.path().join("settings.json");
        let name = crate::component::statusline_binary_name();
        let directory = home.path().join("statusline");
        let destination = directory.join(name);
        let target = home.path().join("dotfiles-statusline");
        let source = home.path().join("source");
        std::fs::create_dir(&directory)?;
        std::os::unix::fs::symlink("../dotfiles-statusline", &destination)?;
        std::fs::write(&target, b"existing statusline")?;
        std::fs::write(&source, b"new statusline")?;
        std::fs::write(&settings, b"{invalid")?;
        let component = Component::new(
            ComponentType::Statusline,
            name.into(),
            source,
            destination.clone(),
            InstallStatus::New,
        );

        assert!(install_component(&component, home.path(), home.path()).is_err());
        assert_eq!(
            std::fs::read_link(&destination)?,
            Path::new("../dotfiles-statusline")
        );
        assert_eq!(std::fs::read(&target)?, b"existing statusline");

        std::fs::write(&settings, b"{}")?;
        install_component(&component, home.path(), home.path())?;
        assert_eq!(
            std::fs::read_link(&destination)?,
            Path::new("../dotfiles-statusline")
        );
        assert_eq!(std::fs::read(&target)?, b"new statusline");
        Ok(())
    }

    #[test]
    fn removing_the_settings_file_preserves_the_original_bytes() -> Result<()> {
        let home = tempfile::tempdir()?;
        let path = home.path().join("settings.json");
        let original =
            b"{\"hooks\": {}, \"outputStyle\": \"mine\", \"statusLine\": {\"command\": \"mine\"}}";
        std::fs::write(&path, original)?;
        let component = Component::new(
            ComponentType::ConfigFile,
            "settings.json".into(),
            path.clone(),
            path.clone(),
            InstallStatus::Managed,
        );
        let error = remove_component(&component, home.path()).unwrap_err();
        assert!(error.to_string().contains("remove individually"));
        assert_eq!(std::fs::read(&path)?, original);
        Ok(())
    }

    #[test]
    fn removing_a_hook_keeps_other_commands_in_its_matcher() -> Result<()> {
        let home = tempfile::tempdir()?;
        let path = home.path().join("settings.json");
        let hook: HookConfig =
            serde_yaml_bw::from_str("name: sample\nevent: SessionStart\ntype: command\n")?;
        let custom = json!({"type": "command", "command": "./sample-custom.sh"});
        let empty = json!({"matcher": "other", "hooks": []});
        let settings = json!({"hooks": {"SessionStart": [
            {"matcher": "startup", "hooks": [custom, {"type": "command", "command": hook.hook_command_path(home.path())}]},
            empty
        ]}});
        std::fs::write(&path, serde_json::to_vec(&settings)?)?;
        let binary = home.path().join(hook.binary_name());
        std::fs::write(&binary, b"hook")?;
        let mut component = Component::new(
            ComponentType::Hooks,
            hook.name.clone(),
            binary.clone(),
            binary.clone(),
            InstallStatus::Unchanged,
        );
        component.hook_config = Some(hook);
        remove_component(&component, home.path())?;
        let result: Value = serde_json::from_slice(&std::fs::read(path)?)?;
        assert_eq!(
            result["hooks"]["SessionStart"],
            json!([{"matcher": "startup", "hooks": [custom]}, empty])
        );
        assert!(!binary.exists());
        Ok(())
    }

    #[test]
    fn statusline_removal_cleans_exact_references_and_restores_the_binary_on_error() -> Result<()> {
        let home = tempfile::tempdir()?;
        let path = home.path().join("settings.json");
        let name = crate::component::statusline_binary_name();
        let binary = home.path().join("statusline").join(name);
        let source = home.path().join("source");
        std::fs::write(&source, b"statusline")?;
        let component = Component::new(
            ComponentType::Statusline,
            name.into(),
            source,
            binary.clone(),
            InstallStatus::New,
        );
        install_component(&component, home.path(), home.path())?;
        let installed = std::fs::read(&path)?;
        std::fs::write(&path, b"{invalid")?;
        let error = remove_component(&component, home.path()).unwrap_err();
        assert!(error.to_string().contains("component restored"));
        assert_eq!(std::fs::read(&binary)?, b"statusline");
        assert_eq!(std::fs::read(&path)?, b"{invalid");
        std::fs::write(&path, installed)?;
        remove_component(&component, home.path())?;
        assert!(!binary.exists());
        let settings: Value = serde_json::from_slice(&std::fs::read(&path)?)?;
        assert_eq!(settings, json!({}));
        std::fs::write(&path, serde_json::to_vec(&json!({"statusLine": name}))?)?;
        remove_component(&component, home.path())?;
        let settings: Value = serde_json::from_slice(&std::fs::read(&path)?)?;
        assert_eq!(settings, json!({}));
        std::fs::write(
            &path,
            br#"{"statusLine":{"type":"command","command":"./custom-statusline"}}"#,
        )?;
        remove_component(&component, home.path())?;
        let settings: Value = serde_json::from_slice(&std::fs::read(&path)?)?;
        assert_eq!(settings["statusLine"]["command"], "./custom-statusline");
        Ok(())
    }

    #[test]
    fn reinstall_waits_until_the_previous_file_and_registration_are_removed() -> Result<()> {
        let home = tempfile::tempdir()?;
        let name = crate::component::statusline_binary_name();
        let source = home.path().join("source");
        std::fs::write(&source, b"statusline")?;
        let binary = home.path().join("statusline").join(name);
        let component = Component::new(
            ComponentType::Statusline,
            name.into(),
            source,
            binary.clone(),
            InstallStatus::New,
        );
        install_component(&component, home.path(), home.path())?;
        std::thread::scope(|threads| -> Result<()> {
            let (stopped, staging) = std::sync::mpsc::channel();
            let (resume, resumed) = std::sync::mpsc::channel();
            let dir = home.path();
            let path = &binary;
            let remove = threads.spawn(move || {
                remove_registered_file(path, || {
                    stopped.send(())?;
                    resumed.recv()?;
                    unregister_statusline_if_matches(dir, name)
                })
            });
            staging.recv_timeout(std::time::Duration::from_secs(5))?;
            let probe = std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(dir.join("statusline/.hibi-component.lock"))?;
            let held = probe.try_lock().is_err();
            drop(probe);
            let reinstall = threads.spawn(|| install_component(&component, dir, dir));
            resume.send(())?;
            remove.join().unwrap()?;
            reinstall.join().unwrap()?;
            assert!(held);
            Ok(())
        })?;
        assert_eq!(std::fs::read(&binary)?, b"statusline");
        let settings: Value =
            serde_json::from_slice(&std::fs::read(home.path().join("settings.json"))?)?;
        assert_eq!(settings["statusLine"]["type"], "command");
        Ok(())
    }
}
