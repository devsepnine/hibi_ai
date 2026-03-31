pub mod config;
pub mod git;

pub use config::{ResolvedSource, SourceEntry, SourceKind};

use std::path::Path;

use anyhow::Result;

/// Result of resolving sources: the resolved list plus any non-fatal warnings.
pub struct ResolveResult {
    pub sources: Vec<ResolvedSource>,
    pub warnings: Vec<String>,
}

/// Resolve all sources: bundled (implicit first) + config entries in order.
/// Last entry in config = highest priority.
/// Warnings are collected instead of printed to stderr (TUI-safe).
pub fn resolve_all_sources(bundled_dir: &Path) -> Result<ResolveResult> {
    let mut sources = vec![ResolvedSource::bundled(bundled_dir)];
    let mut warnings = Vec::new();

    let (entries, auto_update) = match config::load_config() {
        Ok(result) => result,
        Err(e) => {
            warnings.push(format!("Failed to load sources.yaml: {}", e));
            return Ok(ResolveResult { sources, warnings });
        }
    };

    for entry in entries {
        match resolve_entry(&entry, auto_update, &mut warnings) {
            Ok(resolved) => {
                // map_to sources skip structure validation (flat files are valid)
                if resolved.map_to.is_some() || validate_source_dir(&resolved.path) {
                    sources.push(resolved);
                } else {
                    warnings.push(format!(
                        "Source dir lacks expected structure, skipping: {}",
                        resolved.path.display()
                    ));
                }
            }
            Err(e) => {
                warnings.push(format!("Failed to resolve source: {}", e));
            }
        }
    }

    Ok(ResolveResult { sources, warnings })
}

/// Update all git sources (fetch latest). Returns new sources and summaries.
/// Immutable: produces a new Vec instead of mutating in place.
pub fn update_git_sources(sources: &[ResolvedSource]) -> (Vec<ResolvedSource>, Vec<String>) {
    let mut updated = Vec::with_capacity(sources.len());
    let mut summaries = Vec::new();

    for source in sources {
        if source.kind != SourceKind::Git {
            updated.push(source.clone());
            continue;
        }

        // source.path may include root subdirectory; clone needs the base cache dir
        let cache_dir = match git::cache_path_for(&source.label) {
            Ok(dir) => dir,
            Err(e) => {
                updated.push(ResolvedSource { is_stale: true, ..source.clone() });
                summaries.push(format!("  {}: failed ({})", source.label, e));
                continue;
            }
        };
        match git::clone_or_update(&source.label, &source.branch, &cache_dir) {
            Ok(_) => {
                updated.push(ResolvedSource {
                    is_stale: false,
                    ..source.clone()
                });
                summaries.push(format!("  {}: updated", source.label));
            }
            Err(e) => {
                updated.push(ResolvedSource {
                    is_stale: true,
                    ..source.clone()
                });
                summaries.push(format!("  {}: failed ({})", source.label, e));
            }
        }
    }

    (updated, summaries)
}

fn resolve_entry(
    entry: &SourceEntry,
    auto_update: bool,
    warnings: &mut Vec<String>,
) -> Result<ResolvedSource> {
    match entry {
        SourceEntry::Git { url, branch, root, map_to } => {
            config::validate_git_url(url)?;
            let cache_dir = git::cache_path_for(url)?;

            let make_resolved = |base_path: std::path::PathBuf, stale: bool| {
                let path = apply_root(base_path, root.as_deref());
                ResolvedSource {
                    label: url.clone(),
                    kind: SourceKind::Git,
                    path,
                    is_stale: stale,
                    branch: branch.clone(),
                    map_to: map_to.clone(),
                }
            };

            if auto_update {
                match git::clone_or_update(url, branch, &cache_dir) {
                    Ok(path) => Ok(make_resolved(path, false)),
                    Err(e) => {
                        if git::cache_exists(&cache_dir) {
                            warnings.push(format!("Git fetch failed, using stale cache: {}", e));
                            Ok(make_resolved(cache_dir, true))
                        } else {
                            Err(e)
                        }
                    }
                }
            } else if git::cache_exists(&cache_dir) {
                Ok(make_resolved(cache_dir, false))
            } else {
                match git::clone_or_update(url, branch, &cache_dir) {
                    Ok(path) => Ok(make_resolved(path, false)),
                    Err(e) => Err(e),
                }
            }
        }
        SourceEntry::Local { path, root, map_to } => {
            config::validate_local_path(path)?;
            let expanded = config::expand_tilde(path);

            if !expanded.exists() {
                anyhow::bail!("Local source path does not exist: {}", expanded.display());
            }

            let final_path = apply_root(expanded, root.as_deref());
            let label = path.to_string_lossy().to_string();
            Ok(ResolvedSource {
                label,
                kind: SourceKind::Local,
                path: final_path,
                is_stale: false,
                branch: None,
                map_to: map_to.clone(),
            })
        }
    }
}

/// Apply `root` subdirectory to a base path.
fn apply_root(base: std::path::PathBuf, root: Option<&str>) -> std::path::PathBuf {
    match root {
        Some(sub) if !sub.is_empty() => base.join(sub),
        _ => base,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Regression test: update_git_sources must use the base cache dir (without root),
    /// not source.path (which may include root subdirectory).
    /// Bug: clone_or_update received "~/.hibi/cache/.../skills/" instead of "~/.hibi/cache/.../"
    #[test]
    fn test_cache_path_is_prefix_of_resolved_path_with_root() {
        let url = "https://github.com/vercel-labs/agent-skills";
        let cache_dir = git::cache_path_for(url).unwrap();
        let resolved_path = apply_root(cache_dir.clone(), Some("skills"));

        // cache_dir must be a proper prefix of resolved_path
        assert!(resolved_path.starts_with(&cache_dir));
        assert_ne!(resolved_path, cache_dir);

        // cache_dir must NOT contain the root subdirectory
        assert!(!cache_dir.ends_with("skills"));
    }

    #[test]
    fn test_cache_path_equals_resolved_path_without_root() {
        let url = "https://github.com/user/repo.git";
        let cache_dir = git::cache_path_for(url).unwrap();
        let resolved_path = apply_root(cache_dir.clone(), None);

        assert_eq!(resolved_path, cache_dir);
    }
}

/// Check if a directory looks like a valid hibi source.
/// At least one known subdirectory or config file must exist.
fn validate_source_dir(path: &Path) -> bool {
    let markers = [
        "agents", "commands", "contexts", "rules", "skills",
        "hooks", "output-styles", "statusline",
        "mcps/mcps.yaml", "plugins/plugins.yaml",
        "settings.json", "CLAUDE.md", "AGENTS.md",
    ];
    markers.iter().any(|m| path.join(m).exists())
}
