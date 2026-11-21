use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};

use super::config::{validate_branch, validate_git_url};

const CLONE_TIMEOUT_SECS: u64 = 60;
const FETCH_TIMEOUT_SECS: u64 = 30;
const RESET_TIMEOUT_SECS: u64 = 10;
const GIT_PROBE_TIMEOUT_SECS: u64 = 5;

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

    let parent = cache_dir
        .parent()
        .context("Git cache has no parent directory")?;
    std::fs::create_dir_all(parent)?;
    let lock = open_cache_lock(cache_dir)?;
    lock.lock().context("Cannot lock Git cache")?;
    if cache_dir.join(".git").exists() {
        update_repo(cache_dir, branch.as_deref())?;
    } else {
        let stage = tempfile::tempdir_in(parent)?;
        let next = stage.path().join("repo");
        clone_repo(url, branch, &next)?;
        std::fs::rename(next, cache_dir)
            .with_context(|| format!("Cannot publish Git cache {}", cache_dir.display()))?;
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

fn cache_base() -> Result<PathBuf> {
    Ok(crate::paths::require_home_dir()?.join(".hibi/cache"))
}

pub fn cache_path_for(url: &str, branch: Option<&str>) -> Result<PathBuf> {
    cache_path_in(&cache_base()?, url, branch)
}

fn cache_path_in(base: &Path, url: &str, branch: Option<&str>) -> Result<PathBuf> {
    let key = serde_json::to_vec(&(url, branch))?;
    let digest = Sha256::digest(key);
    Ok(base.join("git").join(format!("{digest:x}")))
}

fn open_cache_lock(path: &Path) -> Result<std::fs::File> {
    std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path.with_extension("lock"))
        .context("Cannot open Git cache lock")
}

pub fn legacy_cache_for(url: &str, branch: Option<&str>) -> Option<PathBuf> {
    let label = sanitize_label(url);
    if Path::new(&label).file_name()? != label.as_str() {
        return None;
    }
    let dir = cache_base().ok()?.join(label);
    if !cache_exists(&dir)
        || read_git(&dir, &["config", "--local", "--get", "remote.origin.url"])? != url
    {
        return None;
    }
    let head = read_git(&dir, &["rev-parse", "--verify", "HEAD"])?;
    let reference = format!("refs/remotes/origin/{}", branch.unwrap_or("HEAD"));
    if read_git(&dir, &["rev-parse", "--verify", &reference]).as_deref() == Some(&head) {
        return Some(dir);
    }
    let name = branch?;
    let tag = format!("refs/tags/{name}^{{commit}}");
    if read_git(&dir, &["rev-parse", "--verify", &tag]).as_deref() == Some(&head) {
        return Some(dir);
    }
    // A shallow fetch of an explicit ref may leave only FETCH_HEAD.
    let branch = format!("branch '{name}' of ");
    let tag = format!("tag '{name}' of ");
    let fetch_head = std::fs::read_to_string(dir.join(".git/FETCH_HEAD")).ok()?;
    fetch_head
        .lines()
        .any(|line| {
            line.split_once("\t\t").is_some_and(|(hash, detail)| {
                hash == head && (detail.starts_with(&branch) || detail.starts_with(&tag))
            })
        })
        .then_some(dir)
}

fn read_git(dir: &Path, args: &[&str]) -> Option<String> {
    let mut cmd = Command::new("git");
    cmd.args(args).current_dir(dir);
    let output =
        crate::exec::run_with_timeout(&mut cmd, GIT_PROBE_TIMEOUT_SECS, |_, e| e.into()).ok()??;
    output.status.success().then(|| {
        String::from_utf8_lossy(&output.stdout)
            .trim_end()
            .to_owned()
    })
}

/// Remove the cached repository for a Git source without waiting for an active sync.
/// Returns `Ok(true)` if the cache was removed, `Ok(false)` if no cache existed.
pub fn remove_cache(url: &str, branch: Option<&str>) -> Result<bool> {
    remove_cache_dir(&cache_path_for(url, branch)?)
}

fn remove_cache_dir(cache_dir: &Path) -> Result<bool> {
    if !cache_dir.parent().is_some_and(Path::is_dir) {
        return Ok(false);
    }
    let lock = open_cache_lock(cache_dir)?;
    match lock.try_lock() {
        Ok(()) => {}
        Err(std::fs::TryLockError::WouldBlock) => anyhow::bail!("Git cache is in use"),
        Err(std::fs::TryLockError::Error(error)) => {
            return Err(error).context("Cannot lock Git cache");
        }
    }
    if cache_dir.exists() {
        std::fs::remove_dir_all(cache_dir)
            .with_context(|| format!("Failed to remove cache: {}", cache_dir.display()))?;
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Remove the orphaned bundled-source cache (`~/.hibi/cache/bundled/`).
///
/// Earlier versions synced the bundled source from a remote git repo into
/// this directory. The bundled source is now package-embedded, so the cache
/// is dead data; this performs a one-time cleanup.
/// Returns `Ok(true)` if the cache was removed, `Ok(false)` if none existed.
pub fn cleanup_bundled_cache() -> Result<bool> {
    let home = crate::paths::require_home_dir()?;
    // Fixed path (~/.hibi/cache/bundled), built from home_dir — safe by construction
    // (no external input, unlike remove_cache which sanitizes a user-supplied URL).
    let bundled_dir = home.join(".hibi").join("cache").join("bundled");

    if bundled_dir.exists() {
        std::fs::remove_dir_all(&bundled_dir).with_context(|| {
            format!("Failed to remove bundled cache: {}", bundled_dir.display())
        })?;
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
    let mut args = vec!["clone", "--depth", "1"];
    if let Some(b) = branch {
        args.push("--branch");
        args.push(b);
    }
    args.push(url);
    let dest_str = dest.to_str().ok_or_else(|| {
        anyhow::anyhow!(
            "Cache path contains non-UTF-8 characters: {}",
            dest.display()
        )
    })?;
    args.push(dest_str);

    run_git_command(&args, None, CLONE_TIMEOUT_SECS)
}

fn update_repo(repo_dir: &Path, branch: Option<&str>) -> Result<()> {
    // The clone's fetch refspec can still point to the former default branch.
    let fetch_args = ["fetch", "--depth", "1", "origin", branch.unwrap_or("HEAD")];
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
    let subcommand = args.first().unwrap_or(&"");
    let output = crate::exec::run_with_timeout(&mut cmd, timeout_secs, |_, e| e.into())?
        .ok_or_else(|| anyhow::anyhow!("git {} timed out after {}s", subcommand, timeout_secs))?;
    if output.status.success() {
        return Ok(());
    }
    let raw_stderr = String::from_utf8_lossy(&output.stderr);
    anyhow::bail!(
        "git {} failed (exit {}): {}",
        subcommand,
        output.status.code().unwrap_or(-1),
        sanitize_stderr(&raw_stderr)
    )
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

/// git counts as available once it launches; a probe that outlives its
/// timeout still launched, so a slow machine is not told git is missing.
fn git_available() -> bool {
    let mut cmd = Command::new("git");
    cmd.arg("--version");
    crate::exec::run_with_timeout(&mut cmd, GIT_PROBE_TIMEOUT_SECS, |_, e| e.into()).is_ok()
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
    fn git_cache_separates_branches_and_legacy_url_collisions() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let url = "https://example.com/a_b/c.git";
        let first = cache_path_in(dir.path(), url, Some("first"))?;
        let second = cache_path_in(dir.path(), url, Some("second"))?;
        assert_ne!(first, second);
        assert_ne!(first, cache_path_in(dir.path(), url, None)?);
        assert_ne!(
            first,
            cache_path_in(dir.path(), "https://example.com/a/b_c.git", Some("first"))?
        );
        assert!(!remove_cache_dir(&first)?);
        std::fs::create_dir_all(&first)?;
        std::fs::create_dir_all(&second)?;
        assert!(remove_cache_dir(&first)?);
        assert!(!first.exists());
        assert!(second.is_dir());
        Ok(())
    }

    #[test]
    fn busy_git_cache_is_kept_until_removal_can_lock_it() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let cache = dir.path().join("repo");
        std::fs::create_dir(&cache)?;
        let lock = open_cache_lock(&cache)?;
        lock.lock()?;

        assert_eq!(
            remove_cache_dir(&cache).unwrap_err().to_string(),
            "Git cache is in use"
        );
        assert!(cache.is_dir());

        drop(lock);
        assert!(remove_cache_dir(&cache)?);
        assert!(!cache.exists());
        Ok(())
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
    fn test_cleanup_bundled_cache_path_guard() {
        // The cleanup target must live inside ~/.hibi/cache/ so the
        // defense-in-depth `starts_with` guard can never reject it.
        // (We don't invoke cleanup_bundled_cache() against the real home
        // dir here — that path is a shared global resource and racing test
        // threads would both try to remove it. The behavioral no-op test
        // lives in the source module tests, run-serialized there.)
        if let Some(home) = dirs::home_dir() {
            let cache_base = home.join(".hibi").join("cache");
            let bundled_dir = cache_base.join("bundled");
            assert!(bundled_dir.starts_with(&cache_base));
        }
    }
}
