use std::path::Path;
use anyhow::Result;
use walkdir::WalkDir;

use crate::app::TargetCli;
use crate::component::{Component, ComponentType, HookConfig, InstallStatus};

/// Scan all files in a directory as a single component type (for `map_to` sources).
pub(super) fn scan_flat(
    source_dir: &Path,
    dest_dir: &Path,
    component_type: ComponentType,
) -> Result<Vec<Component>> {
    let mut components = Vec::new();
    scan_directory(source_dir, dest_dir, component_type, &mut components)?;
    Ok(components)
}

/// Scan all component types from a source directory.
pub(super) fn scan_components(
    source_dir: &Path,
    dest_dir: &Path,
    target_cli: TargetCli,
) -> Result<Vec<Component>> {
    let mut components = Vec::new();

    match target_cli {
        TargetCli::Claude => {
            scan_directory(&source_dir.join("agents"), &dest_dir.join("agents"), ComponentType::Agents, &mut components)?;
            scan_directory(&source_dir.join("commands"), &dest_dir.join("commands"), ComponentType::Commands, &mut components)?;
            scan_directory(&source_dir.join("contexts"), &dest_dir.join("contexts"), ComponentType::Contexts, &mut components)?;
            scan_directory(&source_dir.join("rules"), &dest_dir.join("rules"), ComponentType::Rules, &mut components)?;
            scan_directory(&source_dir.join("skills"), &dest_dir.join("skills"), ComponentType::Skills, &mut components)?;
            scan_directory(&source_dir.join("output-styles"), &dest_dir.join("output-styles"), ComponentType::OutputStyles, &mut components)?;
            scan_statusline(source_dir, dest_dir, &mut components)?;
            scan_hooks(source_dir, dest_dir, &mut components)?;
            add_config_files(source_dir, dest_dir, target_cli, &mut components)?;
        }
        TargetCli::Codex => {
            scan_directory(&source_dir.join("skills"), &dest_dir.join("skills"), ComponentType::Skills, &mut components)?;
            add_config_files(source_dir, dest_dir, target_cli, &mut components)?;
        }
    }

    Ok(components)
}

fn scan_directory(
    source_dir: &Path,
    dest_dir: &Path,
    component_type: ComponentType,
    components: &mut Vec<Component>,
) -> Result<()> {
    if !source_dir.exists() {
        return Ok(());
    }

    for entry in WalkDir::new(source_dir)
        .min_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        // Skip files ending with -ko.md
        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
            if file_name.ends_with("-ko.md") {
                continue;
            }
        }

        let relative = path.strip_prefix(source_dir)?;

        // Security: reject path traversal attempts
        if relative.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
            continue;
        }

        let dest_path = dest_dir.join(relative);
        let name = relative.to_string_lossy().to_string();
        let status = determine_status(path, &dest_path)?;

        components.push(Component::new(
            component_type.clone(),
            name,
            path.to_path_buf(),
            dest_path,
            status,
        ));
    }

    Ok(())
}

fn scan_statusline(
    source_dir: &Path,
    dest_dir: &Path,
    components: &mut Vec<Component>,
) -> Result<()> {
    let statusline_dir = source_dir.join("statusline");
    if !statusline_dir.exists() {
        return Ok(());
    }

    let binary_name = if cfg!(windows) {
        "statusline.exe"
    } else if cfg!(target_os = "macos") {
        "statusline_macos"
    } else {
        "statusline_linux"
    };

    let binary_path = statusline_dir.join(binary_name);
    if !binary_path.exists() {
        return Ok(());
    }

    let dest_path = dest_dir.join("statusline").join(binary_name);
    let status = determine_status(&binary_path, &dest_path)?;

    components.push(Component::new(
        ComponentType::Statusline,
        binary_name.to_string(),
        binary_path,
        dest_path,
        status,
    ));

    Ok(())
}

fn scan_hooks(
    source_dir: &Path,
    dest_dir: &Path,
    components: &mut Vec<Component>,
) -> Result<()> {
    let hooks_dir = source_dir.join("hooks");
    if !hooks_dir.exists() {
        return Ok(());
    }

    for entry in std::fs::read_dir(&hooks_dir)? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let hook_yaml = path.join("hook.yaml");
        if !hook_yaml.exists() {
            continue;
        }

        let config_content = std::fs::read_to_string(&hook_yaml)?;
        let config: HookConfig = serde_yaml::from_str(&config_content)?;

        let hook_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        scan_hook_entry(hook_name, &path, dest_dir, config, components)?;
    }

    Ok(())
}

/// Process a single hook directory entry (deprecated or normal).
fn scan_hook_entry(
    hook_name: &str,
    hook_dir: &Path,
    dest_dir: &Path,
    config: HookConfig,
    components: &mut Vec<Component>,
) -> Result<()> {
    let binary_name = config.binary_name();

    if config.is_deprecated() {
        let dest_path = dest_dir.join("hooks").join(&binary_name);
        if !dest_path.exists() {
            return Ok(()); // Not installed, cannot uninstall
        }
        let component = Component::new(
            ComponentType::Hooks,
            hook_name.to_string(),
            dest_path.clone(), // source_path = dest_path (uninstall only)
            dest_path,
            InstallStatus::Unchanged,
        ).with_hook_config(config);
        components.push(component);
    } else {
        let binary_path = hook_dir.join(&binary_name);
        if !binary_path.exists() {
            return Ok(());
        }

        let dest_path = dest_dir.join("hooks").join(&binary_name);
        let status = determine_status(&binary_path, &dest_path)?;

        let component = Component::new(
            ComponentType::Hooks,
            hook_name.to_string(),
            binary_path,
            dest_path,
            status,
        ).with_hook_config(config);
        components.push(component);
    }

    Ok(())
}

fn add_config_files(
    source_dir: &Path,
    dest_dir: &Path,
    target_cli: TargetCli,
    components: &mut Vec<Component>,
) -> Result<()> {
    let config_files = match target_cli {
        TargetCli::Claude => vec!["settings.json", "CLAUDE.md"],
        TargetCli::Codex => vec!["AGENTS.md"],
    };

    for file in config_files {
        let source_path = source_dir.join(file);
        if source_path.exists() {
            let dest_path = dest_dir.join(file);
            let status = if file == "settings.json" && dest_path.exists() {
                InstallStatus::Managed
            } else {
                determine_status(&source_path, &dest_path)?
            };

            components.push(Component::new(
                ComponentType::ConfigFile,
                file.to_string(),
                source_path,
                dest_path,
                status,
            ));
        }
    }

    Ok(())
}

fn determine_status(source: &Path, dest: &Path) -> Result<InstallStatus> {
    if !dest.exists() {
        return Ok(InstallStatus::New);
    }

    // Quick check: compare file size first (avoid reading large files)
    let source_meta = std::fs::metadata(source)?;
    let dest_meta = std::fs::metadata(dest)?;

    if source_meta.len() != dest_meta.len() {
        return Ok(InstallStatus::Modified);
    }

    // Sizes match: compare contents for accuracy
    let source_content = std::fs::read(source)?;
    let dest_content = std::fs::read(dest)?;

    if source_content == dest_content {
        Ok(InstallStatus::Unchanged)
    } else {
        Ok(InstallStatus::Modified)
    }
}
