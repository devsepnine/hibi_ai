use std::collections::HashSet;
use std::path::Path;
use anyhow::Result;
use walkdir::WalkDir;

use crate::app::TargetCli;
use crate::component::{Component, ComponentType, InstallStatus};

/// Source label applied to externally-discovered files (those present in
/// dest_dir but not produced by any configured source).
const EXTERNAL_LABEL: &str = "external";

/// Component types we surface external files for.
/// Excludes Hooks (binaries), Statusline (binaries), ConfigFile (special-case).
///
/// Subdirectory name is derived from `ComponentType::display_name()` to keep
/// a single source of truth (matches `scan_components` in components.rs and
/// the `map_to` dispatch in scanner/mod.rs).
fn external_types(target_cli: TargetCli) -> &'static [ComponentType] {
    match target_cli {
        TargetCli::Claude => &[
            ComponentType::Agents,
            ComponentType::Commands,
            ComponentType::Contexts,
            ComponentType::Rules,
            ComponentType::Skills,
            ComponentType::OutputStyles,
        ],
        TargetCli::Codex => &[ComponentType::Skills],
    }
}

/// Whether a component type should surface non-markdown companion files
/// when scanning for externals.
///
/// Skills are package-shaped (a `SKILL.md` + companion `.json` / `.sh` /
/// `.toml` / etc.) so all files under `skills/` are surfaced. Every other
/// component type is markdown-only by design.
fn includes_companion_files(component_type: &ComponentType) -> bool {
    matches!(component_type, ComponentType::Skills)
}

/// Scan dest_dir for files that are not produced by any source.
///
/// `existing_keys` is the set of `(component_type, name)` keys already
/// claimed by source-based scans. Anything in dest under a known component
/// type directory whose key is NOT in this set is reported as External.
///
/// File-format scope per component type is decided by
/// `includes_companion_files`: markdown-only for most types, all files for
/// Skills.
pub(super) fn scan_externals(
    dest_dir: &Path,
    target_cli: TargetCli,
    existing_keys: &HashSet<(ComponentType, String)>,
) -> Result<Vec<Component>> {
    let mut external = Vec::new();

    for comp_type in external_types(target_cli) {
        let type_dir = dest_dir.join(comp_type.display_name());
        if !type_dir.exists() {
            continue;
        }
        scan_type_directory(&type_dir, comp_type, existing_keys, &mut external)?;
    }

    Ok(external)
}

fn scan_type_directory(
    type_dir: &Path,
    component_type: &ComponentType,
    existing_keys: &HashSet<(ComponentType, String)>,
    out: &mut Vec<Component>,
) -> Result<()> {
    // Mirror scan_directory's pruning: skip workspace/ subtrees.
    for entry in WalkDir::new(type_dir)
        .min_depth(1)
        .into_iter()
        .filter_entry(|e| !(e.file_type().is_dir() && e.file_name() == "workspace"))
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        let Some(name) = external_component_name(
            path,
            type_dir,
            component_type,
            existing_keys,
        ) else {
            continue;
        };

        let dest_path = path.to_path_buf();
        // source_path = dest_path: External components are uninstall-only;
        // install_selected filters them out so this is never read by copy_file.
        let component = Component::new(
            component_type.clone(),
            name,
            dest_path.clone(),
            dest_path,
            InstallStatus::External,
        )
        .with_source_name(EXTERNAL_LABEL);
        out.push(component);
    }

    Ok(())
}

/// Decide whether `path` should appear as an External component, returning
/// its component-relative `name` if so. Returns None for files filtered out
/// by extension, locale suffix, traversal guard, or already-claimed key.
fn external_component_name(
    path: &Path,
    type_dir: &Path,
    component_type: &ComponentType,
    existing_keys: &HashSet<(ComponentType, String)>,
) -> Option<String> {
    if !path.is_file() {
        return None;
    }

    // Markdown-only types skip non-.md files. Skills include every file
    // (json/toml/sh/etc) because skills ship as packages.
    if !includes_companion_files(component_type) {
        match path.extension().and_then(|e| e.to_str()) {
            Some("md") => {}
            _ => return None,
        }
    }

    // Skip Korean reference siblings (`*-ko.md`).
    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
        if stem.ends_with("-ko") {
            return None;
        }
    }

    let relative = path.strip_prefix(type_dir).ok()?;

    // Security: reject path traversal (mirrors scan_directory).
    if relative.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
        return None;
    }

    let name = relative.to_string_lossy().to_string();
    // existing_keys uses forward-slash normalized paths (matches the merge
    // step in scan_all_sources). Normalize before lookup.
    let lookup_key = (component_type.clone(), name.replace('\\', "/"));
    if existing_keys.contains(&lookup_key) {
        return None;
    }

    Some(name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_test_dir(label: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let dir = std::env::temp_dir().join(format!("hibi_ext_{label}_{nanos}"));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn detects_dest_only_md_files() {
        let dest = unique_test_dir("detects");
        std::fs::create_dir_all(dest.join("agents")).unwrap();
        std::fs::write(dest.join("agents/my-custom.md"), "# custom").unwrap();

        let existing = HashSet::new();
        let out = scan_externals(&dest, TargetCli::Claude, &existing).unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].name, "my-custom.md");
        assert_eq!(out[0].status, InstallStatus::External);
        assert_eq!(out[0].source_name, "external");
        assert!(!out[0].selected, "External must default unselected");

        let _ = std::fs::remove_dir_all(&dest);
    }

    #[test]
    fn skips_files_already_claimed_by_source() {
        let dest = unique_test_dir("skip_claimed");
        std::fs::create_dir_all(dest.join("commands")).unwrap();
        std::fs::write(dest.join("commands/managed.md"), "# managed").unwrap();
        std::fs::write(dest.join("commands/extra.md"), "# extra").unwrap();

        let mut existing = HashSet::new();
        existing.insert((ComponentType::Commands, "managed.md".to_string()));

        let out = scan_externals(&dest, TargetCli::Claude, &existing).unwrap();
        let names: Vec<_> = out.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names, vec!["extra.md"]);

        let _ = std::fs::remove_dir_all(&dest);
    }

    #[test]
    fn markdown_only_types_skip_ko_and_non_md() {
        let dest = unique_test_dir("md_only");
        std::fs::create_dir_all(dest.join("agents")).unwrap();
        std::fs::write(dest.join("agents/keep.md"), "en").unwrap();
        std::fs::write(dest.join("agents/keep-ko.md"), "ko").unwrap();
        std::fs::write(dest.join("agents/notes.txt"), "txt").unwrap();
        std::fs::write(dest.join("agents/data.json"), "{}").unwrap();

        let out = scan_externals(&dest, TargetCli::Claude, &HashSet::new()).unwrap();
        let names: Vec<_> = out.iter().map(|c| c.name.replace('\\', "/")).collect();
        assert_eq!(names, vec!["keep.md"]);

        let _ = std::fs::remove_dir_all(&dest);
    }

    #[test]
    fn skills_include_companion_files() {
        let dest = unique_test_dir("skill_companions");
        std::fs::create_dir_all(dest.join("skills/my-skill/references")).unwrap();
        std::fs::write(dest.join("skills/my-skill/SKILL.md"), "en").unwrap();
        std::fs::write(dest.join("skills/my-skill/SKILL-ko.md"), "ko").unwrap();
        std::fs::write(dest.join("skills/my-skill/script.sh"), "#!/bin/sh").unwrap();
        std::fs::write(dest.join("skills/my-skill/data.json"), "{}").unwrap();
        std::fs::write(dest.join("skills/my-skill/config.toml"), "key='v'").unwrap();
        std::fs::write(dest.join("skills/my-skill/references/foo.md"), "ref").unwrap();

        let out = scan_externals(&dest, TargetCli::Claude, &HashSet::new()).unwrap();
        let mut names: Vec<_> = out.iter().map(|c| c.name.replace('\\', "/")).collect();
        names.sort();

        let mut expected: Vec<String> = vec![
            "my-skill/SKILL.md".to_string(),
            "my-skill/script.sh".to_string(),
            "my-skill/data.json".to_string(),
            "my-skill/config.toml".to_string(),
            "my-skill/references/foo.md".to_string(),
        ];
        expected.sort();

        assert_eq!(names, expected);
        // -ko remains excluded even for Skills.
        assert!(out.iter().all(|c| !c.name.contains("-ko")));

        let _ = std::fs::remove_dir_all(&dest);
    }

    #[test]
    fn skips_workspace_subtree() {
        let dest = unique_test_dir("skip_ws");
        std::fs::create_dir_all(dest.join("skills/x/workspace/iter")).unwrap();
        std::fs::write(dest.join("skills/x/SKILL.md"), "ok").unwrap();
        std::fs::write(dest.join("skills/x/workspace/iter/log.md"), "junk").unwrap();

        let out = scan_externals(&dest, TargetCli::Claude, &HashSet::new()).unwrap();
        assert!(out.iter().all(|c| !c.name.contains("workspace")));
        assert_eq!(out.len(), 1);

        let _ = std::fs::remove_dir_all(&dest);
    }

    #[test]
    fn empty_dest_dir_returns_nothing() {
        let dest = unique_test_dir("empty");
        let out = scan_externals(&dest, TargetCli::Claude, &HashSet::new()).unwrap();
        assert!(out.is_empty());
        let _ = std::fs::remove_dir_all(&dest);
    }

    #[test]
    fn codex_only_scans_skills() {
        let dest = unique_test_dir("codex");
        std::fs::create_dir_all(dest.join("agents")).unwrap();
        std::fs::create_dir_all(dest.join("skills/s")).unwrap();
        std::fs::write(dest.join("agents/leak.md"), "should not appear").unwrap();
        std::fs::write(dest.join("skills/s/SKILL.md"), "ok").unwrap();

        let out = scan_externals(&dest, TargetCli::Codex, &HashSet::new()).unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].component_type, ComponentType::Skills);

        let _ = std::fs::remove_dir_all(&dest);
    }
}
