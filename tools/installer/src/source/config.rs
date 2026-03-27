use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// YAML config entry for a source.
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum SourceEntry {
    #[serde(rename = "git")]
    Git {
        url: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        branch: Option<String>,
        /// Subdirectory within the repo to use as source root (e.g., "config/claude").
        #[serde(default, skip_serializing_if = "Option::is_none")]
        root: Option<String>,
        /// Map all files to a specific component type (e.g., "rules", "skills").
        #[serde(default, skip_serializing_if = "Option::is_none")]
        map_to: Option<String>,
    },
    #[serde(rename = "local")]
    Local {
        path: PathBuf,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        root: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        map_to: Option<String>,
    },
}

/// Runtime-resolved source kind.
#[derive(Clone, Debug, PartialEq)]
pub enum SourceKind {
    Bundled,
    Git,
    Local,
}

/// Runtime-resolved source with filesystem path.
#[derive(Clone, Debug)]
pub struct ResolvedSource {
    pub label: String,
    pub kind: SourceKind,
    pub path: PathBuf,
    pub is_stale: bool,
    pub branch: Option<String>,
    /// Map all files to a specific component type (e.g., "rules", "skills").
    pub map_to: Option<String>,
}

impl ResolvedSource {
    pub fn bundled(path: &Path) -> Self {
        Self {
            label: "bundled".to_string(),
            kind: SourceKind::Bundled,
            path: path.to_path_buf(),
            is_stale: false,
            branch: None,
            map_to: None,
        }
    }
}

/// Top-level YAML structure for `~/.hibi/sources.yaml`.
#[derive(Deserialize, Serialize, Debug)]
struct SourcesConfig {
    #[serde(default)]
    sources: Vec<SourceEntry>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    auto_update: Option<bool>,
}

/// Load source entries from `~/.hibi/sources.yaml`.
/// Returns empty vec if file doesn't exist or is unparseable.
/// Maximum allowed size for sources.yaml (64 KB).
const MAX_CONFIG_SIZE: u64 = 65_536;

pub fn load_config() -> Result<(Vec<SourceEntry>, bool)> {
    let config_path = config_path()?;
    if !config_path.exists() {
        return Ok((Vec::new(), true));
    }

    let metadata = std::fs::metadata(&config_path)?;
    if metadata.len() > MAX_CONFIG_SIZE {
        anyhow::bail!("sources.yaml exceeds 64KB size limit");
    }

    let content = std::fs::read_to_string(&config_path)?;
    let config: SourcesConfig = serde_yaml::from_str(&content)?;
    let auto_update = config.auto_update.unwrap_or(true);
    Ok((config.sources, auto_update))
}

/// Path to `~/.hibi/sources.yaml`.
fn config_path() -> Result<PathBuf> {
    let home = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;
    Ok(home.join(".hibi").join("sources.yaml"))
}

/// Save source entries to `~/.hibi/sources.yaml`.
pub fn save_config(entries: &[SourceEntry], auto_update: bool) -> Result<()> {
    let config = SourcesConfig {
        sources: entries.to_vec(),
        // Omit auto_update from YAML when it's the default (true)
        auto_update: if auto_update { None } else { Some(false) },
    };
    let config_path = config_path()?;
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let yaml = serde_yaml::to_string(&config)?;
    std::fs::write(&config_path, yaml)?;
    Ok(())
}

/// Expand `~` prefix to home directory.
pub fn expand_tilde(path: &Path) -> PathBuf {
    let s = path.to_string_lossy();
    if s.starts_with("~/") || s.starts_with("~\\") {
        if let Some(home) = dirs::home_dir() {
            return home.join(&s[2..]);
        }
    }
    path.to_path_buf()
}

/// Validate a git URL: HTTPS only, no credentials.
pub fn validate_git_url(url: &str) -> Result<()> {
    // Check credentials first to avoid leaking them in error messages
    if url.contains('@') {
        anyhow::bail!("Git URL must not contain credentials");
    }
    if !url.starts_with("https://") {
        anyhow::bail!("Only HTTPS git URLs are allowed");
    }
    Ok(())
}

/// Validate a git branch name: no flags, no traversal.
pub fn validate_branch(branch: &str) -> Result<()> {
    if branch.starts_with('-') {
        anyhow::bail!("Branch name must not start with '-': {}", branch);
    }
    if branch.contains("..") || branch.contains('\0') {
        anyhow::bail!("Invalid branch name: {}", branch);
    }
    Ok(())
}

/// Validate a local path: no path traversal, not inside ~/.claude/.
/// Canonicalizes to resolve symlinks before checking.
pub fn validate_local_path(path: &Path) -> Result<()> {
    let expanded = expand_tilde(path);

    // Reject path traversal in the raw string
    let s = expanded.to_string_lossy();
    if s.contains("..") {
        anyhow::bail!("Path traversal (..) not allowed in source path: {}", s);
    }

    // Canonicalize to resolve symlinks, then re-check
    let canonical = expanded.canonicalize()
        .unwrap_or_else(|_| expanded.clone());

    if let Some(home) = dirs::home_dir() {
        let claude_dir = home.join(".claude");
        let canonical_claude = claude_dir.canonicalize()
            .unwrap_or(claude_dir);
        if canonical.starts_with(&canonical_claude) {
            anyhow::bail!(
                "Source path resolves to inside ~/.claude/: {}",
                canonical.display()
            );
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_tilde() {
        let path = Path::new("~/foo/bar");
        let expanded = expand_tilde(path);
        assert!(!expanded.to_string_lossy().starts_with('~'));
        assert!(expanded.to_string_lossy().ends_with("foo/bar")
            || expanded.to_string_lossy().ends_with("foo\\bar"));
    }

    #[test]
    fn test_validate_git_url_https_ok() {
        assert!(validate_git_url("https://github.com/user/repo.git").is_ok());
    }

    #[test]
    fn test_validate_git_url_ssh_rejected() {
        assert!(validate_git_url("git@github.com:user/repo.git").is_err());
    }

    #[test]
    fn test_validate_git_url_credentials_rejected() {
        assert!(validate_git_url("https://user@github.com/repo.git").is_err());
    }

    #[test]
    fn test_validate_git_url_http_rejected() {
        assert!(validate_git_url("http://github.com/user/repo.git").is_err());
    }

    #[test]
    fn test_validate_branch_normal_ok() {
        assert!(validate_branch("main").is_ok());
        assert!(validate_branch("feature/foo").is_ok());
    }

    #[test]
    fn test_validate_branch_flag_rejected() {
        assert!(validate_branch("--upload-pack=evil").is_err());
        assert!(validate_branch("-c").is_err());
    }

    #[test]
    fn test_validate_branch_traversal_rejected() {
        assert!(validate_branch("main..evil").is_err());
    }

    #[test]
    fn test_validate_local_path_traversal_rejected() {
        assert!(validate_local_path(Path::new("/tmp/../etc/passwd")).is_err());
    }

    #[test]
    fn test_validate_local_path_claude_dir_rejected() {
        let home = dirs::home_dir().unwrap();
        let claude_path = home.join(".claude").join("agents");
        assert!(validate_local_path(&claude_path).is_err());
    }

    #[test]
    fn test_validate_local_path_normal_ok() {
        assert!(validate_local_path(Path::new("/tmp/my-configs")).is_ok());
    }

    #[test]
    fn test_source_entry_yaml_roundtrip() {
        let entries = vec![
            SourceEntry::Git {
                url: "https://github.com/user/repo.git".to_string(),
                branch: Some("main".to_string()),
                root: None,
                map_to: None,
            },
            SourceEntry::Git {
                url: "https://github.com/other/cfg.git".to_string(),
                branch: None,
                root: Some("config/claude".to_string()),
                map_to: None,
            },
            SourceEntry::Local {
                path: PathBuf::from("/tmp/my-configs"),
                root: None,
                map_to: Some("rules".to_string()),
            },
        ];

        let config = super::SourcesConfig {
            sources: entries.clone(),
            auto_update: None,
        };

        let yaml = serde_yaml::to_string(&config).unwrap();
        let parsed: super::SourcesConfig = serde_yaml::from_str(&yaml).unwrap();

        assert_eq!(parsed.sources.len(), 3);
        // Verify tag serialization produces "type: git" / "type: local"
        assert!(yaml.contains("type: git"));
        assert!(yaml.contains("type: local"));
        assert!(yaml.contains("https://github.com/user/repo.git"));
        assert!(yaml.contains("root: config/claude")); // root field serialized
        assert!(yaml.contains("map_to: rules"));       // map_to field serialized
        assert!(!yaml.contains("auto_update")); // Omitted when None
    }
}
