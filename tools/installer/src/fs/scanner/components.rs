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

    // Skip skill-development artifacts.
    // Convention: skill-creator writes iteration/benchmark outputs to
    // `<skill>/workspace/`. See `src/skills/*/workspace/` and `.gitignore`.
    // Pruning via `filter_entry` avoids descending the subtree entirely.
    for entry in WalkDir::new(source_dir)
        .min_depth(1)
        .into_iter()
        .filter_entry(|e| !(e.file_type().is_dir() && e.file_name() == "workspace"))
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
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

/// Strip `\r` bytes so that CRLF and LF files compare as equal.
/// This avoids false "Modified" status when the same content is checked out
/// with different git `autocrlf` settings.
fn normalize_line_endings(content: &[u8]) -> Vec<u8> {
    content.iter().copied().filter(|&b| b != b'\r').collect()
}

fn determine_status(source: &Path, dest: &Path) -> Result<InstallStatus> {
    if !dest.exists() {
        return Ok(InstallStatus::New);
    }

    let source_content = std::fs::read(source)?;
    let dest_content = std::fs::read(dest)?;

    // Compare with normalized line endings to ignore CRLF/LF differences
    if normalize_line_endings(&source_content) == normalize_line_endings(&dest_content) {
        Ok(InstallStatus::Unchanged)
    } else {
        Ok(InstallStatus::Modified)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_test_dir(label: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let dir = std::env::temp_dir().join(format!("hibi_test_{label}_{nanos}"));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn scan_directory_skips_workspace_subtree() {
        let src = unique_test_dir("scan_ws_src");
        let dst = unique_test_dir("scan_ws_dst");

        // skill-a: one installable file + workspace artifacts that must be skipped
        std::fs::create_dir_all(src.join("skill-a/workspace/iteration-1")).unwrap();
        std::fs::write(src.join("skill-a/SKILL.md"), "ok").unwrap();
        std::fs::write(src.join("skill-a/workspace/iteration-1/log.txt"), "junk").unwrap();
        std::fs::write(src.join("skill-a/workspace/benchmark.json"), "{}").unwrap();

        let mut out = Vec::new();
        scan_directory(&src, &dst, ComponentType::Skills, &mut out).unwrap();

        assert_eq!(out.len(), 1, "only SKILL.md should be scanned");
        assert!(out[0].name.ends_with("SKILL.md"));
        assert!(!out.iter().any(|c| c.name.contains("workspace")));

        let _ = std::fs::remove_dir_all(&src);
        let _ = std::fs::remove_dir_all(&dst);
    }
}
