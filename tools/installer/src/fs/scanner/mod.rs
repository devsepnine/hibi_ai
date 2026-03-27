mod validation;
mod components;
mod mcp;
mod plugin;

use std::collections::HashMap;
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
pub fn scan_all_sources(
    sources: &[ResolvedSource],
    dest_dir: &Path,
    target_cli: TargetCli,
) -> Result<Vec<Component>> {
    merge_scanned(
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
    )
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
