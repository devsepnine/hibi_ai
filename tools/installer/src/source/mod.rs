pub mod config;
pub mod git;

pub use config::{ResolvedSource, SourceEntry, SourceKind};

use std::path::{Path, PathBuf};

use anyhow::Result;

/// Discover the source directory containing bundled components.
/// Always uses the package-embedded source files (no remote sync).
pub(crate) fn find_source_dir() -> Result<PathBuf> {
    find_package_source_dir()
}

/// Find the source dir from package-installed paths (exe-relative, cwd, etc.)
fn find_package_source_dir() -> Result<PathBuf> {
    let exe_dir = std::env::current_exe()?
        .parent()
        .map(|p| p.to_path_buf())
        .ok_or_else(|| anyhow::anyhow!("Cannot get executable directory"))?;

    #[cfg(target_os = "macos")]
    if exe_dir.ends_with("Contents/MacOS") {
        let resources = exe_dir.with_file_name("Resources");
        if validate_source_dir(&resources) {
            return Ok(resources);
        }
    }

    let working_dir = std::env::current_dir()?;
    let candidates = [
        exe_dir.clone(),                  // Scoop: exe and config in same dir
        exe_dir.join("../share/hibi"),    // Homebrew standard
        exe_dir.join("../share/hibi-ai"), // Homebrew alternative
        exe_dir.join("../../.."),         // From target/release
        exe_dir.join("../.."),            // From target
        exe_dir.join("../../../../src"),  // Repository development build
        working_dir.join("src"),          // Cargo run from repository root
        working_dir.clone(),              // Current directory
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

    anyhow::bail!(
        "Cannot find source directory. Run from the repository root or a release package containing bundled configuration"
    )
}

// ---------------------------------------------------------------------------
// Sync & Resolve
// ---------------------------------------------------------------------------

/// Result of resolving sources: the resolved list plus any non-fatal warnings.
pub struct ResolveResult {
    pub sources: Vec<ResolvedSource>,
    pub warnings: Vec<String>,
}

/// Result of a full sync operation (source resolve).
pub(crate) struct SyncReport {
    pub resolved: Vec<ResolvedSource>,
    pub summaries: Vec<String>,
    pub had_error: bool,
}

/// Resolve all user-configured sources. The bundled source is package-embedded
/// (`source_dir`) and is never fetched from a remote.
/// Checks `cancel_rx` before resolving for cooperative cancellation.
pub(crate) fn sync_all_sources(
    source_dir: &Path,
    cancel_rx: &std::sync::mpsc::Receiver<()>,
) -> SyncReport {
    let mut summaries = Vec::new();

    // Bundled source is package-embedded; nothing to fetch.
    summaries.push("  bundled: local".to_string());

    // Cooperative cancel check before resolving user sources
    if cancel_rx.try_recv().is_ok() {
        summaries.push("  cancelled".to_string());
        return SyncReport {
            resolved: vec![ResolvedSource::bundled(source_dir)],
            summaries,
            had_error: true,
        };
    }

    report_resolution(
        source_dir,
        summaries,
        resolve_sources(source_dir, Refresh::Manual),
    )
}

fn report_resolution(
    source_dir: &Path,
    mut summaries: Vec<String>,
    result: Result<ResolveResult>,
) -> SyncReport {
    match result {
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
            let had_error = !r.warnings.is_empty();
            summaries.extend(r.warnings);
            SyncReport {
                resolved: r.sources,
                summaries,
                had_error,
            }
        }
        Err(e) => {
            summaries.push(format!("  re-resolve failed: {}", e));
            SyncReport {
                resolved: vec![ResolvedSource::bundled(source_dir)],
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
    resolve_sources(bundled_dir, Refresh::Configured)
}

enum Refresh {
    Configured,
    Manual,
}

fn resolve_sources(bundled_dir: &Path, refresh: Refresh) -> Result<ResolveResult> {
    let mut sources = vec![ResolvedSource::bundled(bundled_dir)];
    let mut warnings = Vec::new();

    let (entries, auto_update) = match config::load_config() {
        Ok(result) => result,
        Err(e) => {
            warnings.push(format!("Failed to load sources.yaml: {}", e));
            return Ok(ResolveResult { sources, warnings });
        }
    };

    let fetch = auto_update || matches!(refresh, Refresh::Manual);
    for entry in entries {
        match resolve_entry(&entry, fetch, &mut warnings) {
            Ok(resolved) => retain_source(resolved, &mut sources, &mut warnings),
            Err(e) => {
                warnings.push(format!("Failed to resolve source: {}", e));
            }
        }
    }

    Ok(ResolveResult { sources, warnings })
}

fn retain_source(
    resolved: ResolvedSource,
    sources: &mut Vec<ResolvedSource>,
    warnings: &mut Vec<String>,
) {
    if !resolved.path.is_dir() {
        warnings.push(format!(
            "Source path is not an existing directory, skipping: {}",
            resolved.path.display()
        ));
    } else if resolved.map_to.is_some() || validate_user_source_dir(&resolved.path) {
        sources.push(resolved);
    } else {
        warnings.push(format!(
            "Source dir lacks expected structure, skipping: {}",
            resolved.path.display()
        ));
    }
}

fn resolve_entry(
    entry: &SourceEntry,
    auto_update: bool,
    warnings: &mut Vec<String>,
) -> Result<ResolvedSource> {
    match entry {
        SourceEntry::Git {
            url,
            branch,
            root,
            map_to,
        } => resolve_git(url, branch, root.as_deref(), map_to, auto_update, warnings),
        SourceEntry::Local { path, root, map_to } => resolve_local(path, root.as_deref(), map_to),
    }
}

/// Resolve a git source to a directory on disk.
///
/// `auto_update` picks which cost the caller accepts: set, every resolve reaches
/// the network so the components are current; unset, an existing cache is taken
/// as-is and the network is touched only when there is no cache to take.
fn resolve_git(
    url: &str,
    branch: &Option<String>,
    root: Option<&str>,
    map_to: &Option<String>,
    auto_update: bool,
    warnings: &mut Vec<String>,
) -> Result<ResolvedSource> {
    config::validate_git_url(url)?;
    if let Some(branch) = branch {
        config::validate_branch(branch)?;
    }
    let cache_dir = git::cache_path_for(url, branch.as_deref())?;

    let make_resolved = |base: PathBuf, is_stale: bool| ResolvedSource {
        label: url.to_string(),
        kind: SourceKind::Git,
        path: apply_root(base, root),
        is_stale,
        branch: branch.clone(),
        map_to: map_to.clone(),
    };

    let fallback = || {
        if git::cache_exists(&cache_dir) {
            Some(cache_dir.clone())
        } else {
            git::legacy_cache_for(url, branch.as_deref())
        }
    };
    if !auto_update && let Some(cached) = fallback() {
        return Ok(make_resolved(cached, false));
    }

    match git::clone_or_update(url, branch, &cache_dir) {
        Ok(path) => Ok(make_resolved(path, false)),
        // A failed fetch is survivable while a previous clone is still on disk:
        // an offline user gets what they last synced, flagged stale, rather than
        // an empty component list.
        Err(e) => {
            let Some(cached) = fallback() else {
                return Err(e);
            };
            warnings.push(format!("Git fetch failed, using stale cache: {}", e));
            Ok(make_resolved(cached, true))
        }
    }
}

/// Resolve a local source, which must already exist — there is nothing to fetch.
fn resolve_local(
    path: &Path,
    root: Option<&str>,
    map_to: &Option<String>,
) -> Result<ResolvedSource> {
    config::validate_local_path(path)?;
    let expanded = config::expand_tilde(path);

    if !expanded.exists() {
        anyhow::bail!("Local source path does not exist: {}", expanded.display());
    }

    Ok(ResolvedSource {
        label: path.to_string_lossy().to_string(),
        kind: SourceKind::Local,
        path: apply_root(expanded, root),
        is_stale: false,
        branch: None,
        map_to: map_to.clone(),
    })
}

/// Apply `root` subdirectory to a base path.
fn apply_root(base: PathBuf, root: Option<&str>) -> PathBuf {
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
        "agents",
        "commands",
        "contexts",
        "rules",
        "skills",
        "hooks",
        "output-styles",
        "statusline",
        "mcps/mcps.yaml",
        "plugins/plugins.yaml",
        "settings.json",
        "CLAUDE.md",
        "AGENTS.md",
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
        let cache_dir = git::cache_path_for(url, None).unwrap();
        let resolved_path = apply_root(cache_dir.clone(), Some("skills"));

        assert!(resolved_path.starts_with(&cache_dir));
        assert_ne!(resolved_path, cache_dir);
        assert!(!cache_dir.ends_with("skills"));
    }

    #[test]
    fn test_cache_path_equals_resolved_path_without_root() {
        let url = "https://github.com/user/repo.git";
        let cache_dir = git::cache_path_for(url, None).unwrap();
        let resolved_path = apply_root(cache_dir.clone(), None);

        assert_eq!(resolved_path, cache_dir);
    }

    #[test]
    fn test_cleanup_bundled_cache_noop_when_absent() {
        // When the orphaned cache dir is absent, cleanup is a safe no-op
        // returning Ok(false). This is the only test that invokes the
        // function against the real home dir, so no parallel test races it
        // on that shared path.
        let absent = dirs::home_dir()
            .map(|h| !h.join(".hibi").join("cache").join("bundled").exists())
            .unwrap_or(false);
        // Only invoke against the real home dir when the cache is absent, so the
        // test can never delete a real user's bundled cache. When present, skip.
        if absent {
            let result = git::cleanup_bundled_cache();
            assert!(result.is_ok());
            assert!(!result.unwrap());
        }
    }
    #[test]
    fn unparseable_source_configuration_is_reported_as_error_with_bundled_fallback() {
        let bundled = Path::new("/test/bundled");
        let report = report_resolution(
            bundled,
            vec!["bundled: local".into()],
            Ok(ResolveResult {
                sources: vec![ResolvedSource::bundled(bundled)],
                warnings: vec!["Failed to load sources.yaml: invalid YAML".into()],
            }),
        );
        assert!(report.had_error);
        assert_eq!(report.resolved.len(), 1);
        assert_eq!(report.resolved[0].kind, SourceKind::Bundled);
        assert!(
            report
                .summaries
                .iter()
                .any(|message| message.contains("invalid YAML"))
        );
    }

    #[test]
    fn warning_free_source_resolution_is_reported_as_success() {
        let bundled = Path::new("/test/bundled");
        let report = report_resolution(
            bundled,
            Vec::new(),
            Ok(ResolveResult {
                sources: vec![ResolvedSource::bundled(bundled)],
                warnings: Vec::new(),
            }),
        );
        assert!(!report.had_error);
    }
    #[test]
    fn explicit_mapping_does_not_accept_missing_or_file_root() {
        let serial = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let base = std::env::temp_dir().join(format!(
            "hibi_root_validation_{}_{serial}",
            std::process::id()
        ));
        std::fs::create_dir(&base).unwrap();
        std::fs::write(base.join("file-root"), b"not a directory").unwrap();
        for root in ["missing-root", "file-root"] {
            let resolved = resolve_entry(
                &SourceEntry::Local {
                    path: base.clone(),
                    root: Some(root.into()),
                    map_to: Some("skills".into()),
                },
                false,
                &mut Vec::new(),
            )
            .unwrap();
            let mut sources = vec![ResolvedSource::bundled(&base)];
            let mut warnings = Vec::new();
            retain_source(resolved, &mut sources, &mut warnings);
            assert_eq!(sources.len(), 1);
            assert_eq!(sources[0].kind, SourceKind::Bundled);
            assert_eq!(warnings.len(), 1);
            assert!(warnings[0].contains("not an existing directory"));
            assert!(warnings[0].contains(root));
        }
        std::fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn explicit_mapping_accepts_existing_directory_without_structural_markers() {
        let base = std::env::temp_dir();
        let resolved = resolve_entry(
            &SourceEntry::Local {
                path: base.clone(),
                root: None,
                map_to: Some("skills".into()),
            },
            false,
            &mut Vec::new(),
        )
        .unwrap();
        let mut sources = Vec::new();
        let mut warnings = Vec::new();
        retain_source(resolved, &mut sources, &mut warnings);
        assert_eq!(sources.len(), 1);
        assert!(warnings.is_empty());
    }
}
