mod validation;
mod components;
mod external;
mod mcp;
mod plugin;

use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::path::Path;
use anyhow::Result;

use crate::app::TargetCli;
use crate::component::{Component, ComponentType};
use crate::mcp::McpServer;
use crate::plugin::Plugin;
use crate::source::ResolvedSource;

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
        |c| format!("{}/{}", c.component_type.display_name(), c.name.replace('\\', "/")),
        |c, label| c.source_name = label.to_string(),
    )?;

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
) -> Result<(Vec<McpServer>, Option<String>)> {
    let (installed, warning) = match target_cli {
        TargetCli::Claude => mcp::get_installed_claude_servers(),
        TargetCli::Codex => mcp::get_installed_codex_servers(),
    };

    let servers = merge_scanned(
        sources,
        |source| mcp::scan_with_installed(&source.path, &installed),
        |s| s.def.name.clone(),
        |s, label| s.source_name = label.to_string(),
    )?;

    Ok((servers, warning))
}

/// Scan plugins from all sources. Later sources override earlier ones.
pub fn scan_all_plugin_sources(sources: &[ResolvedSource]) -> Result<Vec<Plugin>> {
    merge_scanned(
        sources,
        |source| plugin::scan_plugins(&source.path),
        |p| p.def.name.clone(),
        |p, label| p.source_name = label.to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::InstallStatus;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_dir(label: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
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
            components.iter().find(|c| c.component_type == t && c.name.replace('\\', "/") == n)
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
        let first_external_idx = components.iter().position(|c| c.status == InstallStatus::External).unwrap();
        let last_source_idx = components.iter().rposition(|c| c.source_name == "bundled").unwrap();
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
