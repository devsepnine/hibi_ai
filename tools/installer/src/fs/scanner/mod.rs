mod components;
mod external;
mod mcp;
mod plugin;
mod validation;

use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::path::Path;

use crate::component::{Component, ComponentType};
use crate::mcp::McpServer;
use crate::plugin::Plugin;
use crate::source::ResolvedSource;
use crate::target::TargetCli;

/// Merge items from multiple sources using a last-wins strategy.
///
/// For each source, `scan_fn` produces a list of items. Items with the same
/// key (produced by `key_fn`) are replaced by later sources. Each item gets
/// its `source_name` set via `set_source`.
fn merge_scanned<T, K>(
    sources: &[ResolvedSource],
    scan_fn: impl Fn(&ResolvedSource) -> Result<Vec<T>>,
    key_fn: impl Fn(&T) -> K,
    mut set_source: impl FnMut(&mut T, &str),
) -> Result<Vec<T>>
where
    K: Eq + Hash,
{
    let mut seen: HashMap<K, usize> = HashMap::new();
    let mut merged: Vec<T> = Vec::new();

    for source in sources {
        let items = scan_fn(source)?;
        for mut item in items {
            let key = key_fn(&item);
            set_source(&mut item, &source.label);

            if let Some(&idx) = seen.get(&key) {
                merged[idx] = item;
            } else {
                seen.insert(key, merged.len());
                merged.push(item);
            }
        }
    }

    Ok(merged)
}

/// A `*-ko` sibling is the Korean mirror kept for developer readability and is
/// never installed; the English original is what the model reads.
fn is_korean_mirror(path: &Path) -> bool {
    path.file_stem()
        .and_then(|s| s.to_str())
        .is_some_and(|stem| stem.ends_with("-ko"))
}

/// Parse a `map_to` string into a ComponentType.
fn parse_map_to(map_to: &str) -> Option<ComponentType> {
    match map_to.to_lowercase().as_str() {
        "agents" => Some(ComponentType::Agents),
        "commands" => Some(ComponentType::Commands),
        "contexts" => Some(ComponentType::Contexts),
        "rules" => Some(ComponentType::Rules),
        "skills" => Some(ComponentType::Skills),
        "hooks" => Some(ComponentType::Hooks),
        "output-styles" | "styles" => Some(ComponentType::OutputStyles),
        _ => None,
    }
}

/// Scan components from all sources. Later sources override earlier ones.
/// Sources with `map_to` get flat-scanned as a single component type.
///
/// After merging source-based components, this also scans `dest_dir` for
/// files that no source produces and reports them with
/// `InstallStatus::External` so users can see (and optionally remove)
/// orphaned or user-added files. Format scope per type is decided by the
/// scanner (markdown-only for most, all files for Skills).
pub fn scan_all_sources(
    sources: &[ResolvedSource],
    dest_dir: &Path,
    target_cli: TargetCli,
) -> Result<Vec<Component>> {
    let mut merged = merge_scanned(
        sources,
        |source| {
            if let Some(map_to) = source.map_to.as_deref() {
                if let Some(comp_type) = parse_map_to(map_to) {
                    let type_dir = comp_type.display_name().to_lowercase();
                    let dest = dest_dir.join(&type_dir);
                    components::scan_flat(&source.path, &dest, comp_type)
                } else {
                    Ok(Vec::new()) // Unknown map_to value, skip silently
                }
            } else {
                components::scan_components(&source.path, dest_dir, target_cli)
            }
        },
        |c| {
            format!(
                "{}/{}",
                c.component_type.display_name(),
                c.name.replace('\\', "/")
            )
        },
        |c, label| c.source_name = label.to_string(),
    )?;

    if let Some(index) = merged.iter().position(Component::is_settings_file) {
        let file = &mut merged[index];
        match super::installer::scan_setting_entries(file) {
            Ok(entries) => {
                merged.splice(index..=index, entries);
            }
            Err(error) => {
                file.status = crate::component::InstallStatus::Blocked(format!("{error:#}"));
            }
        }
    }

    // Reuse the same key shape as scan_directory uses for component name
    // (relative path with forward slashes).
    let existing_keys: HashSet<(ComponentType, String)> = merged
        .iter()
        .map(|c| (c.component_type.clone(), c.name.replace('\\', "/")))
        .collect();

    let externals = external::scan_externals(dest_dir, target_cli, &existing_keys)?;
    merged.extend(externals);

    Ok(merged)
}

/// Scan MCP servers from all sources. Later sources override earlier ones.
/// CLI command for installed servers runs only once (not per-source).
pub fn scan_all_mcp_sources(
    sources: &[ResolvedSource],
    target_cli: TargetCli,
    project: Option<&Path>,
) -> Result<(Vec<McpServer>, Option<String>)> {
    let (installed, warning) = match target_cli {
        TargetCli::Claude => mcp::get_installed_claude_servers(project),
        TargetCli::Codex => match crate::fs::codex_mcp_names() {
            Ok(names) => (names, None),
            Err(error) => (Vec::new(), Some(format!("Codex MCP scan failed: {error}"))),
        },
    };

    let servers = merge_scanned(
        sources,
        |source| {
            mcp::scan_with_installed(
                &source.path,
                warning.is_none().then_some(installed.as_slice()),
            )
        },
        |s| s.def.name.clone(),
        |s, label| s.source_name = label.to_string(),
    )?;

    Ok((servers, warning))
}

/// Scan plugins from all sources. Later sources override earlier ones.
pub fn scan_all_plugin_sources(sources: &[ResolvedSource]) -> Result<Vec<Plugin>> {
    let installed = plugin::get_installed_plugins();
    merge_scanned(
        sources,
        |source| plugin::scan_plugins(&source.path, installed.as_ref()),
        |p| p.def.id(),
        |p, label| p.source_name = label.to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::InstallStatus;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn settings_rows_install_and_remove_only_selected_keys() -> Result<()> {
        let fixture = tempfile::tempdir()?;
        let source = fixture.path().join("source");
        let dest = fixture.path().join(".claude");
        std::fs::create_dir(&source)?;
        std::fs::create_dir(&dest)?;
        std::fs::write(
            source.join("settings.json"),
            br#"{"env":{"MCP_TIMEOUT":10000,"KEEP":true},"autoUpdates":false}"#,
        )?;
        let target = dest.join("settings.json");
        std::fs::write(source.join("CLAUDE.md"), b"# Bundled instructions")?;
        std::fs::write(dest.join("CLAUDE.md"), b"# User instructions")?;
        std::fs::write(
            &target,
            br#"{"model":"custom","env":{"USER_VAR":"preserve","MCP_TIMEOUT":5000}}"#,
        )?;
        let sources = [ResolvedSource::bundled(&source)];
        let rows = scan_all_sources(&sources, &dest, TargetCli::Claude)?;
        assert_eq!(
            rows.iter().map(|row| row.name.as_str()).collect::<Vec<_>>(),
            [
                "settings.json/autoUpdates",
                "settings.json/env.KEEP",
                "settings.json/env.MCP_TIMEOUT",
                "CLAUDE.md"
            ]
        );
        assert!(rows.iter().all(|row| !row.selected));
        let timeout = &rows[2];
        assert_eq!(timeout.status, InstallStatus::Modified);
        assert!(!timeout.is_remove_eligible());
        crate::fs::installer::install_component(timeout, &source, &dest)?;
        let read = || {
            serde_json::from_slice::<serde_json::Value>(&std::fs::read(&target).unwrap()).unwrap()
        };
        assert_eq!(
            read(),
            serde_json::json!({"model": "custom", "env": {"USER_VAR": "preserve", "MCP_TIMEOUT": 10000}})
        );

        let rows = scan_all_sources(&sources, &dest, TargetCli::Claude)?;
        assert_eq!(rows[2].status, InstallStatus::Unchanged);
        assert!(rows[2].is_remove_eligible());
        crate::fs::installer::remove_component(&rows[2], &dest)?;
        assert_eq!(
            read(),
            serde_json::json!({"model": "custom", "env": {"USER_VAR": "preserve"}})
        );
        Ok(())
    }

    #[test]
    fn settings_rows_keep_a_value_edited_since_scan() -> Result<()> {
        let fixture = tempfile::tempdir()?;
        let source = fixture.path().join("source");
        let dest = fixture.path().join(".claude");
        std::fs::create_dir(&source)?;
        std::fs::create_dir(&dest)?;
        let target = dest.join("settings.json");
        std::fs::write(
            source.join("settings.json"),
            br#"{"env":{"MCP_TIMEOUT":10000}}"#,
        )?;
        std::fs::write(&target, br#"{"env":{"MCP_TIMEOUT":10000}}"#)?;
        let sources = [ResolvedSource::bundled(&source)];
        let rows = scan_all_sources(&sources, &dest, TargetCli::Claude)?;
        let timeout = &rows[0];
        assert_eq!(timeout.name, "settings.json/env.MCP_TIMEOUT");
        let edited = br#"{"env":{"MCP_TIMEOUT":8000,"USER_VAR":true}}"#;
        std::fs::write(&target, edited)?;
        let remove = crate::fs::installer::remove_component(timeout, &dest).unwrap_err();
        assert!(format!("{remove:#}").contains("changed"));
        let install = crate::fs::installer::install_component(timeout, &source, &dest).unwrap_err();
        assert!(format!("{install:#}").contains("changed"));
        assert_eq!(std::fs::read(target)?, edited);
        Ok(())
    }

    #[test]
    fn only_a_ko_suffixed_stem_is_a_korean_mirror() {
        assert!(is_korean_mirror(Path::new("skills/why/SKILL-ko.md")));
        assert!(is_korean_mirror(Path::new("agents/architect-ko.md")));
        assert!(!is_korean_mirror(Path::new("skills/why/SKILL.md")));
        assert!(!is_korean_mirror(Path::new("skills/kotlin/SKILL.md")));
        assert!(!is_korean_mirror(Path::new("skills/why/ko-notes.md")));
    }

    fn unique_dir(label: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("hibi_scan_all_{label}_{nanos}"));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn merges_source_components_with_external_dest_files() {
        // Layout:
        //   source/agents/keep.md       -> matches dest -> Unchanged
        //   source/agents/managed.md    -> differs from dest -> Modified
        //   dest/agents/keep.md         (matches source content)
        //   dest/agents/managed.md      (different content)
        //   dest/agents/orphan.md       (only in dest -> External)
        //   dest/skills/my-tool/SKILL.md   (only in dest -> External)
        //   dest/skills/my-tool/data.json  (only in dest, Skills companion -> External)
        let source = unique_dir("source");
        let dest = unique_dir("dest");

        std::fs::create_dir_all(source.join("agents")).unwrap();
        std::fs::write(source.join("agents/keep.md"), "shared\n").unwrap();
        std::fs::write(source.join("agents/managed.md"), "v1\n").unwrap();

        std::fs::create_dir_all(dest.join("agents")).unwrap();
        std::fs::create_dir_all(dest.join("skills/my-tool")).unwrap();
        std::fs::write(dest.join("agents/keep.md"), "shared\n").unwrap();
        std::fs::write(dest.join("agents/managed.md"), "v2\n").unwrap();
        std::fs::write(dest.join("agents/orphan.md"), "user-added\n").unwrap();
        std::fs::write(dest.join("skills/my-tool/SKILL.md"), "ext skill\n").unwrap();
        std::fs::write(dest.join("skills/my-tool/data.json"), "{}").unwrap();

        let sources = vec![ResolvedSource::bundled(&source)];
        let components = scan_all_sources(&sources, &dest, TargetCli::Claude).unwrap();

        // Index by (type, name) for assertion.
        let by_key = |t: ComponentType, n: &str| -> Option<&Component> {
            components
                .iter()
                .find(|c| c.component_type == t && c.name.replace('\\', "/") == n)
        };

        // Source-backed components keep their existing statuses.
        let keep = by_key(ComponentType::Agents, "keep.md").expect("keep.md missing");
        assert_eq!(keep.status, InstallStatus::Unchanged);
        assert_eq!(keep.source_name, "bundled");

        let managed = by_key(ComponentType::Agents, "managed.md").expect("managed.md missing");
        assert_eq!(managed.status, InstallStatus::Modified);
        assert_eq!(managed.source_name, "bundled");

        // External-only files surface with External status + external label.
        let orphan = by_key(ComponentType::Agents, "orphan.md").expect("orphan.md missing");
        assert_eq!(orphan.status, InstallStatus::External);
        assert_eq!(orphan.source_name, "external");
        assert!(!orphan.selected, "External must default unselected");

        let skill_md = by_key(ComponentType::Skills, "my-tool/SKILL.md").expect("SKILL.md missing");
        assert_eq!(skill_md.status, InstallStatus::External);

        // Skills companion non-md file surfaces as External (not filtered).
        let companion = by_key(ComponentType::Skills, "my-tool/data.json")
            .expect("data.json must surface as External companion");
        assert_eq!(companion.status, InstallStatus::External);

        // Source-backed components must precede externals (UI ordering
        // contract: known-managed first, user-discoverable last).
        let first_external_idx = components
            .iter()
            .position(|c| c.status == InstallStatus::External)
            .unwrap();
        let last_source_idx = components
            .iter()
            .rposition(|c| c.source_name == "bundled")
            .unwrap();
        assert!(
            last_source_idx < first_external_idx,
            "source-backed components must precede externals"
        );

        let _ = std::fs::remove_dir_all(&source);
        let _ = std::fs::remove_dir_all(&dest);
    }

    #[test]
    fn source_claim_prevents_external_double_count() {
        // A file that exists in both source and dest must appear ONCE
        // (with source status), never as External as well -- otherwise the
        // user would see two list entries for the same file.
        let source = unique_dir("dup_source");
        let dest = unique_dir("dup_dest");

        std::fs::create_dir_all(source.join("commands")).unwrap();
        std::fs::create_dir_all(dest.join("commands")).unwrap();
        std::fs::write(source.join("commands/hello.md"), "v1").unwrap();
        std::fs::write(dest.join("commands/hello.md"), "v1").unwrap();

        let sources = vec![ResolvedSource::bundled(&source)];
        let components = scan_all_sources(&sources, &dest, TargetCli::Claude).unwrap();

        let hellos: Vec<_> = components
            .iter()
            .filter(|c| c.component_type == ComponentType::Commands && c.name == "hello.md")
            .collect();
        assert_eq!(hellos.len(), 1, "hello.md must appear exactly once");
        assert_ne!(
            hellos[0].status,
            InstallStatus::External,
            "source-claimed file must not be External"
        );

        let _ = std::fs::remove_dir_all(&source);
        let _ = std::fs::remove_dir_all(&dest);
    }
}
