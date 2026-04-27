use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use anyhow::{Context, Result};
use wait_timeout::ChildExt;

use super::config::{validate_branch, validate_git_url};

const CLONE_TIMEOUT_SECS: u64 = 60;
const FETCH_TIMEOUT_SECS: u64 = 30;
const RESET_TIMEOUT_SECS: u64 = 10;

/// Detect if `source_dir` lives inside the hibi_ai git repository.
/// Returns the hibi_ai git root path, or `None` if not in a hibi_ai repo.
///
/// Why the repo-marker check: `share/hibi/` installed by package managers
/// (e.g., Homebrew's `/opt/homebrew/Cellar/hibi/*/share/hibi/`) can sit
/// inside an unrelated parent git repo (`/opt/homebrew/.git`). Without
/// the marker guard, `sync` would `git pull` Homebrew itself instead of
/// hibi_ai, and the cache clone path (`sync_bundled_cache`) would never
/// be taken — leaving the user's bundled content permanently stale.
pub fn find_git_root(source_dir: &Path) -> Option<PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(source_dir)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let path_str = String::from_utf8(output.stdout).ok()?;
    let root = PathBuf::from(normalize_git_path(path_str.trim()));
    // Canonicalize both to handle symlinks (e.g., macOS /tmp -> /private/tmp)
    let root_canonical = root.canonicalize().unwrap_or_else(|_| root.clone());
    let source_canonical = source_dir.canonicalize().unwrap_or_else(|_| source_dir.to_path_buf());
    // Guard 1: source_dir must live inside root.
    if !source_canonical.starts_with(&root_canonical) {
        return None;
    }
    // Guard 2: root must look like the hibi_ai repo. The installer's own
    // Cargo.toml is a reliable fingerprint that survives rename/restructure
    // better than checking for `src/agents` alone.
    if !root_canonical.join("tools/installer/Cargo.toml").exists() {
        return None;
    }
    Some(root)
}

/// Pull latest changes in a local (non-shallow) git repository.
/// Uses `--ff-only` to avoid creating merge commits.
pub fn pull_local_repo(repo_dir: &Path) -> Result<()> {
    if !git_available() {
        anyhow::bail!("git is not installed or not in PATH");
    }
    // Always fetch with `--tags --force` so upstream tag rewrites don't
    // poison the operation (see git pull --ff-only's tag-clobber
    // failure mode that v1.9.7 -> v1.9.8 had to hotfix).
    run_git_command(
        &["fetch", "--tags", "--force", "origin"],
        Some(repo_dir),
        FETCH_TIMEOUT_SECS,
    )?;

    // Shallow repos (the bundled cache is cloned with `--depth 1`)
    // cannot fast-forward across an upstream history rewrite because
    // the old and new commits share no ancestor in the local object
    // graph — `merge --ff-only` rejects with "refusing to merge
    // unrelated histories". Reset --hard is the only recovery, and
    // it is safe here because shallow caches never carry local user
    // commits. Full clones (dev checkouts) might, so keep `--ff-only`
    // there to refuse silently dropping local work.
    if is_shallow_repo(repo_dir)? {
        run_git_command(
            &["reset", "--hard", "FETCH_HEAD"],
            Some(repo_dir),
            RESET_TIMEOUT_SECS,
        )
    } else {
        run_git_command(
            &["merge", "--ff-only", "@{u}"],
            Some(repo_dir),
            FETCH_TIMEOUT_SECS,
        )
    }
}

/// Detect a shallow clone (e.g., one made with `--depth N`).
/// Returns false on any unexpected git failure so callers default to
/// the safer `merge --ff-only` path for full clones.
fn is_shallow_repo(repo_dir: &Path) -> Result<bool> {
    let output = Command::new("git")
        .args(["rev-parse", "--is-shallow-repository"])
        .current_dir(repo_dir)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .output()?;
    if !output.status.success() {
        return Ok(false);
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim() == "true")
}

/// Clone or update a git repository into the cache directory.
/// Returns the local path to the cached repo.
pub fn clone_or_update(url: &str, branch: &Option<String>, cache_dir: &Path) -> Result<PathBuf> {
    validate_git_url(url)?;
    if let Some(b) = branch {
        validate_branch(b)?;
    }

    if !git_available() {
        anyhow::bail!("git is not installed or not in PATH");
    }

    if cache_dir.join(".git").exists() {
        update_repo(cache_dir, branch.as_deref())?;
    } else {
        clone_repo(url, branch, cache_dir)?;
    }

    // Record last fetch timestamp
    let timestamp_file = cache_dir.join(".hibi_last_fetch");
    let _ = std::fs::write(&timestamp_file, unix_timestamp_now());

    Ok(cache_dir.to_path_buf())
}

/// Check if a cached repo exists (even if stale).
pub fn cache_exists(cache_dir: &Path) -> bool {
    cache_dir.join(".git").exists()
}

/// Compute cache directory path for a git source.
/// `~/.hibi/cache/<sanitized_label>/`
pub fn cache_path_for(url: &str) -> Result<PathBuf> {
    let home = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;
    let label = sanitize_label(url);
    Ok(home.join(".hibi").join("cache").join(label))
}

/// Remove the cached repository for a git source URL.
/// Returns `Ok(true)` if the cache was removed, `Ok(false)` if no cache existed.
pub fn remove_cache(url: &str) -> Result<bool> {
    let cache_dir = cache_path_for(url)?;

    // Defense-in-depth: ensure we only delete within ~/.hibi/cache/
    let home = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;
    let cache_base = home.join(".hibi").join("cache");
    if !cache_dir.starts_with(&cache_base) {
        anyhow::bail!("Refusing to remove path outside cache dir: {}", cache_dir.display());
    }

    if cache_dir.exists() {
        std::fs::remove_dir_all(&cache_dir)
            .with_context(|| format!("Failed to remove cache: {}", cache_dir.display()))?;
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Sanitize a URL into a filesystem-safe directory name.
fn sanitize_label(url: &str) -> String {
    url.trim_start_matches("https://")
        .trim_end_matches(".git")
        .replace(['/', ':', '\\'], "_")
}

fn clone_repo(url: &str, branch: &Option<String>, dest: &Path) -> Result<()> {
    // Remove stale cache directory (exists but no .git) before cloning
    if dest.exists() && !dest.join(".git").exists() {
        std::fs::remove_dir_all(dest)
            .with_context(|| format!("Failed to remove stale cache (no .git): {}", dest.display()))?;
    }

    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut args = vec!["clone", "--depth", "1"];
    if let Some(b) = branch {
        args.push("--branch");
        args.push(b);
    }
    args.push(url);
    let dest_str = dest.to_str()
        .ok_or_else(|| anyhow::anyhow!("Cache path contains non-UTF-8 characters: {}", dest.display()))?;
    args.push(dest_str);

    run_git_command(&args, None, CLONE_TIMEOUT_SECS)
}

fn update_repo(repo_dir: &Path, branch: Option<&str>) -> Result<()> {
    // When no branch specified, fetch default branch (omit refspec)
    let mut fetch_args = vec!["fetch", "--depth", "1", "origin"];
    if let Some(b) = branch {
        fetch_args.push(b);
    }

    run_git_command(&fetch_args, Some(repo_dir), FETCH_TIMEOUT_SECS)?;

    run_git_command(
        &["reset", "--hard", "FETCH_HEAD"],
        Some(repo_dir),
        RESET_TIMEOUT_SECS,
    )
}

fn run_git_command(args: &[&str], working_dir: Option<&Path>, timeout_secs: u64) -> Result<()> {
    let mut cmd = Command::new("git");
    cmd.args(args);
    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }
    cmd.stdin(std::process::Stdio::null());
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    let mut child = cmd.spawn()?;

    match child.wait_timeout(Duration::from_secs(timeout_secs))? {
        Some(status) if status.success() => Ok(()),
        Some(status) => {
            let mut stderr_buf = Vec::new();
            if let Some(mut stderr) = child.stderr.take() {
                let _ = std::io::Read::read_to_end(&mut stderr, &mut stderr_buf);
            }
            let raw_stderr = String::from_utf8_lossy(&stderr_buf);
            let safe_stderr = sanitize_stderr(&raw_stderr);
            anyhow::bail!(
                "git {} failed (exit {}): {}",
                args.first().unwrap_or(&""),
                status.code().unwrap_or(-1),
                safe_stderr
            )
        }
        None => {
            let _ = child.kill();
            let _ = child.wait();
            anyhow::bail!("git {} timed out after {}s", args.first().unwrap_or(&""), timeout_secs)
        }
    }
}

/// Remove lines from stderr that may contain credentials or sensitive info.
fn sanitize_stderr(stderr: &str) -> String {
    let sensitive_keywords = ["password", "token", "credential", "authorization", "secret"];
    stderr
        .lines()
        .filter(|line| {
            let lower = line.to_lowercase();
            // Filter keyword matches
            if sensitive_keywords.iter().any(|kw| lower.contains(kw)) {
                return false;
            }
            // Filter URLs with embedded credentials (user:pass@host)
            if lower.contains("https://") && line.contains('@') {
                return false;
            }
            true
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn git_available() -> bool {
    Command::new("git")
        .arg("--version")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok()
}

/// Convert MSYS/Unix-style git paths to native OS paths.
/// On Windows, git returns paths like `/c/Users/...` which must become `C:/Users/...`.
/// On other platforms, returns the path unchanged.
fn normalize_git_path(path: &str) -> String {
    #[cfg(windows)]
    {
        // MSYS path: /c/Users/... -> C:/Users/...
        let bytes = path.as_bytes();
        if bytes.len() >= 3 && bytes[0] == b'/' && bytes[2] == b'/' && bytes[1].is_ascii_alphabetic() {
            let drive = (bytes[1] as char).to_ascii_uppercase();
            return format!("{}:{}", drive, &path[2..]);
        }
        // Cygwin path: /cygdrive/c/Users/... -> C:/Users/...
        if let Some(rest) = path.strip_prefix("/cygdrive/") {
            let bytes = rest.as_bytes();
            if bytes.len() >= 2 && bytes[1] == b'/' && bytes[0].is_ascii_alphabetic() {
                let drive = (bytes[0] as char).to_ascii_uppercase();
                return format!("{}:{}", drive, &rest[1..]);
            }
        }
        path.to_string()
    }
    #[cfg(not(windows))]
    {
        path.to_string()
    }
}

fn unix_timestamp_now() -> String {
    use std::time::SystemTime;
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(d) => d.as_secs().to_string(),
        Err(_) => "0".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_label() {
        assert_eq!(
            sanitize_label("https://github.com/user/repo.git"),
            "github.com_user_repo"
        );
    }

    #[test]
    fn test_cache_path_for() {
        let path = cache_path_for("https://github.com/user/repo.git").unwrap();
        assert!(path.to_string_lossy().contains(".hibi"));
        assert!(path.to_string_lossy().contains("cache"));
        assert!(path.to_string_lossy().contains("github.com_user_repo"));
    }

    #[test]
    fn test_remove_cache_nonexistent() {
        let result = remove_cache("https://github.com/nonexistent/repo-never-cloned-99999.git");
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_remove_cache_existing() {
        let url = "https://github.com/test/remove-cache-test-unique-42.git";
        let cache_dir = cache_path_for(url).unwrap();
        std::fs::create_dir_all(&cache_dir).unwrap();
        std::fs::write(cache_dir.join("dummy"), "test").unwrap();

        let result = remove_cache(url);
        assert!(result.is_ok());
        assert!(result.unwrap());
        assert!(!cache_dir.exists());
    }

    #[test]
    fn test_sanitize_stderr_removes_sensitive() {
        let stderr = "fatal: could not read Username\nPassword for 'https://github.com': \nremote: Repository not found.";
        let sanitized = sanitize_stderr(stderr);
        assert!(!sanitized.to_lowercase().contains("password"));
        assert!(sanitized.contains("Repository not found"));
    }

    #[test]
    fn test_sanitize_stderr_keeps_normal() {
        let stderr = "fatal: remote error: repository not found\nerror: Could not fetch origin";
        let sanitized = sanitize_stderr(stderr);
        assert_eq!(sanitized, stderr);
    }

    #[test]
    #[cfg(windows)]
    fn test_normalize_git_path_msys() {
        assert_eq!(normalize_git_path("/c/Users/test"), "C:/Users/test");
        assert_eq!(normalize_git_path("/d/workspace"), "D:/workspace");
    }

    #[test]
    #[cfg(windows)]
    fn test_normalize_git_path_cygwin() {
        assert_eq!(normalize_git_path("/cygdrive/c/Users/test"), "C:/Users/test");
        assert_eq!(normalize_git_path("/cygdrive/d/workspace"), "D:/workspace");
    }

    #[test]
    fn test_normalize_git_path_passthrough() {
        // Non-MSYS paths pass through unchanged on all platforms
        assert_eq!(normalize_git_path("/Users/test"), "/Users/test");
        assert_eq!(normalize_git_path("/home/user"), "/home/user");
    }

    /// Regression: Homebrew installs `share/hibi/` inside `/opt/homebrew`
    /// which is itself a git repo. `find_git_root` must NOT treat that
    /// unrelated parent repo as hibi_ai; otherwise `sync` pulls Homebrew
    /// and never populates `~/.hibi/cache/bundled/`. The marker check
    /// (`tools/installer/Cargo.toml` at the root) defends against this.
    #[test]
    fn test_find_git_root_requires_hibi_marker() {
        use std::process::Command;

        let probe = std::env::temp_dir().join(format!(
            "hibi_find_git_root_probe_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::remove_dir_all(&probe);
        std::fs::create_dir_all(&probe).unwrap();

        // Seed a git repo without any hibi_ai marker
        let init_ok = Command::new("git")
            .arg("init")
            .current_dir(&probe)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        if !init_ok {
            // Environment lacks git; skip (CI configs without git skip here).
            let _ = std::fs::remove_dir_all(&probe);
            return;
        }

        // Without marker: must be rejected (Homebrew-like case)
        assert!(
            find_git_root(&probe).is_none(),
            "git repo without tools/installer/Cargo.toml must not be treated as hibi_ai"
        );

        // With marker: must be accepted
        std::fs::create_dir_all(probe.join("tools").join("installer")).unwrap();
        std::fs::write(
            probe.join("tools").join("installer").join("Cargo.toml"),
            "[package]\nname = \"probe\"\n",
        )
        .unwrap();

        assert!(
            find_git_root(&probe).is_some(),
            "git repo with hibi marker must be accepted"
        );

        let _ = std::fs::remove_dir_all(&probe);
    }
}
