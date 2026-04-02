pub mod config;
pub mod git;

pub use config::{ResolvedSource, SourceEntry, SourceKind};

use std::path::{Path, PathBuf};

use anyhow::Result;

/// Hardcoded repo URL for remote sync. Not user-configurable (unlike sources.yaml).
const BUNDLED_REPO_URL: &str = "https://github.com/devsepnine/hibi_ai";

/// Discover the source directory containing bundled components.
/// Prefers cached clone (from previous sync) over package-installed files.
pub(crate) fn find_source_dir() -> Result<PathBuf> {
    // Cached bundled source takes priority (previously synced)
    if let Some(cached) = bundled_cache_src_dir() {
        if validate_source_dir(&cached) {
            return Ok(cached);
        }
    }
    find_package_source_dir()
}

/// Find the source dir from package-installed paths (exe-relative, cwd, etc.)
fn find_package_source_dir() -> Result<PathBuf> {
    let exe_dir = std::env::current_exe()?
        .parent()
        .map(|p| p.to_path_buf())
        .ok_or_else(|| anyhow::anyhow!("Cannot get executable directory"))?;

    let candidates = [
        exe_dir.clone(),                          // Scoop: exe and config in same dir
        exe_dir.join("../share/hibi"),            // Homebrew standard
        exe_dir.join("../share/hibi-ai"),         // Homebrew alternative
        exe_dir.join("../../.."),                  // From target/release
        exe_dir.join("../.."),                     // From target
        std::env::current_dir()?,                  // Current directory
        std::env::current_dir()?.join("config/ai/claude"),
    ];

    for candidate in candidates {
        // canonicalize resolves `..` and symlinks; if it fails (path doesn't exist),
        // the raw candidate is used — but marker check below will reject it safely.
        let resolved = candidate.canonicalize().unwrap_or(candidate);
        if validate_source_dir(&resolved) {
            return Ok(resolved);
        }
    }

    let default = std::env::current_dir()?.join("config/ai/claude");
    if validate_source_dir(&default) {
        return Ok(default);
    }

    anyhow::bail!("Cannot find source directory. Run from dotfiles root or config/ai/claude/tools/installer")
}

/// `~/.hibi/cache/bundled/src/` — the cached bundled source directory.
fn bundled_cache_src_dir() -> Option<PathBuf> {
    Some(dirs::home_dir()?.join(".hibi").join("cache").join("bundled").join("src"))
}

/// `~/.hibi/cache/bundled/` — the cache base (git clone root).
fn bundled_cache_base() -> Result<PathBuf> {
    let home = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;
    Ok(home.join(".hibi").join("cache").join("bundled"))
}

/// Clone or update the remote hibi_ai repo to cache. Returns the `src/` path.
fn sync_bundled_cache() -> Result<PathBuf> {
    let cache_dir = bundled_cache_base()?;
    git::clone_or_update(BUNDLED_REPO_URL, &None, &cache_dir)?;
    Ok(cache_dir.join("src"))
}

// ---------------------------------------------------------------------------
// Sync & Resolve
// ---------------------------------------------------------------------------

/// Result of resolving sources: the resolved list plus any non-fatal warnings.
pub struct ResolveResult {
    pub sources: Vec<ResolvedSource>,
    pub warnings: Vec<String>,
}

/// Result of a full sync operation (bundled pull + source resolve).
pub(crate) struct SyncReport {
    pub resolved: Vec<ResolvedSource>,
    pub summaries: Vec<String>,
    pub had_error: bool,
}

/// Pull bundled repo (or clone from remote) then resolve all sources.
/// Checks `cancel_rx` between phases for cooperative cancellation.
pub(crate) fn sync_all_sources(
    bundled_git_root: Option<&Path>,
    source_dir: &Path,
    cancel_rx: &std::sync::mpsc::Receiver<()>,
) -> SyncReport {
    let mut summaries = Vec::new();
    let mut had_error = false;

    // Phase 1: sync bundled source
    let effective_source_dir = if let Some(git_root) = bundled_git_root {
        // In-repo: git pull
        match git::pull_local_repo(git_root) {
            Ok(()) => summaries.push("  bundled: updated".to_string()),
            Err(e) => {
                summaries.push(format!("  bundled: failed ({})", e));
                had_error = true;
            }
        }
        source_dir.to_path_buf()
    } else {
        // Package manager install: clone/update from remote
        match sync_bundled_cache() {
            Ok(cached_dir) => {
                summaries.push("  bundled: updated (remote)".to_string());
                cached_dir
            }
            Err(e) => {
                summaries.push(format!("  bundled: failed ({})", e));
                had_error = true;
                source_dir.to_path_buf()
            }
        }
    };

    // Cooperative cancel check between phases
    if cancel_rx.try_recv().is_ok() {
        summaries.push("  cancelled".to_string());
        return SyncReport {
            resolved: vec![ResolvedSource::bundled(&effective_source_dir)],
            summaries,
            had_error: true,
        };
    }

    // Phase 2: resolve all sources using effective source dir
    match resolve_all_sources(&effective_source_dir) {
        Ok(r) => {
            for s in &r.sources {
                if s.kind == SourceKind::Git {
                    if s.is_stale {
                        summaries.push(format!("  {}: stale", s.label));
                    } else {
                        summaries.push(format!("  {}: updated", s.label));
                    }
                }
            }
            summaries.extend(r.warnings);
            SyncReport { resolved: r.sources, summaries, had_error }
        }
        Err(e) => {
            summaries.push(format!("  re-resolve failed: {}", e));
            SyncReport {
                resolved: vec![ResolvedSource::bundled(&effective_source_dir)],
                summaries,
                had_error: true,
            }
        }
    }
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
                if resolved.map_to.is_some() || validate_user_source_dir(&resolved.path) {
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

/// Strict check: bundled source must have both agents/ and settings.json.
fn validate_source_dir(path: &Path) -> bool {
    path.join("agents").exists() && path.join("settings.json").exists()
}

/// Permissive check: user sources just need at least one known marker.
fn validate_user_source_dir(path: &Path) -> bool {
    let markers = [
        "agents", "commands", "contexts", "rules", "skills",
        "hooks", "output-styles", "statusline",
        "mcps/mcps.yaml", "plugins/plugins.yaml",
        "settings.json", "CLAUDE.md", "AGENTS.md",
    ];
    markers.iter().any(|m| path.join(m).exists())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Regression test: resolve_entry must use the base cache dir, not source.path,
    /// when a `root` subdirectory is configured.
    /// Bug history: clone_or_update once received "~/.hibi/cache/.../skills/" instead of "~/.hibi/cache/.../"
    #[test]
    fn test_cache_path_is_prefix_of_resolved_path_with_root() {
        let url = "https://github.com/vercel-labs/agent-skills";
        let cache_dir = git::cache_path_for(url).unwrap();
        let resolved_path = apply_root(cache_dir.clone(), Some("skills"));

        assert!(resolved_path.starts_with(&cache_dir));
        assert_ne!(resolved_path, cache_dir);
        assert!(!cache_dir.ends_with("skills"));
    }

    #[test]
    fn test_cache_path_equals_resolved_path_without_root() {
        let url = "https://github.com/user/repo.git";
        let cache_dir = git::cache_path_for(url).unwrap();
        let resolved_path = apply_root(cache_dir.clone(), None);

        assert_eq!(resolved_path, cache_dir);
    }

    #[test]
    fn test_bundled_cache_src_dir() {
        let dir = bundled_cache_src_dir();
        assert!(dir.is_some());
        let path = dir.unwrap();
        assert!(path.to_string_lossy().contains("bundled"));
        assert!(path.to_string_lossy().ends_with("src"));
    }
}
