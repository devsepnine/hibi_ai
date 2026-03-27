use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use anyhow::Result;
use wait_timeout::ChildExt;

use super::config::{validate_branch, validate_git_url};

const CLONE_TIMEOUT_SECS: u64 = 60;
const FETCH_TIMEOUT_SECS: u64 = 30;
const RESET_TIMEOUT_SECS: u64 = 10;

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

/// Sanitize a URL into a filesystem-safe directory name.
fn sanitize_label(url: &str) -> String {
    url.trim_start_matches("https://")
        .trim_end_matches(".git")
        .replace(['/', ':', '\\'], "_")
}

fn clone_repo(url: &str, branch: &Option<String>, dest: &Path) -> Result<()> {
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut args = vec!["clone", "--depth", "1"];
    if let Some(b) = branch {
        args.push("--branch");
        args.push(b);
    }
    args.push(url);
    let dest_str = dest.to_string_lossy().to_string();
    args.push(&dest_str);

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
}
