use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
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
    /// Retained for display; consumed during clone_or_update in resolve_git.
    #[allow(dead_code)]
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

/// Maximum allowed size for sources.yaml, 64 KB.
const MAX_CONFIG_SIZE: u64 = 65_536;

/// Load source entries, using defaults only when the file is absent.
pub fn load_config() -> Result<(Vec<SourceEntry>, bool)> {
    load_config_at(&config_path()?)
}

fn load_config_at(path: &Path) -> Result<(Vec<SourceEntry>, bool)> {
    let metadata = match std::fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok((Vec::new(), true));
        }
        Err(error) => {
            return Err(error).with_context(|| format!("Cannot inspect {}", path.display()));
        }
    };
    if !metadata.is_file() {
        anyhow::bail!("Source configuration is not a file: {}", path.display());
    }
    if metadata.len() > MAX_CONFIG_SIZE {
        anyhow::bail!("sources.yaml exceeds 64KB size limit");
    }
    let mut content = String::new();
    std::fs::File::open(path)?
        .take(MAX_CONFIG_SIZE + 1)
        .read_to_string(&mut content)?;
    if content.len() as u64 > MAX_CONFIG_SIZE {
        anyhow::bail!("sources.yaml exceeds 64KB size limit");
    }
    let config: SourcesConfig = serde_yaml_bw::from_str(&content)?;
    Ok((config.sources, config.auto_update.unwrap_or(true)))
}

/// Path to `~/.hibi/sources.yaml`.
fn config_path() -> Result<PathBuf> {
    let home = crate::paths::require_home_dir()?;
    Ok(home.join(".hibi").join("sources.yaml"))
}

/// Save source entries to `~/.hibi/sources.yaml`.
pub fn save_config(entries: &[SourceEntry], auto_update: bool) -> Result<()> {
    save_config_at(&config_path()?, entries, auto_update)
}

fn save_config_at(path: &Path, entries: &[SourceEntry], auto_update: bool) -> Result<()> {
    load_config_at(path).with_context(|| {
        format!(
            "Cannot update {} because its existing contents could not be loaded",
            path.display()
        )
    })?;
    let config = SourcesConfig {
        sources: entries.to_vec(),
        // Omit auto_update from YAML when it's the default (true)
        auto_update: if auto_update { None } else { Some(false) },
    };
    let yaml = serde_yaml_bw::to_string(&config)?;
    if yaml.len() as u64 > MAX_CONFIG_SIZE {
        anyhow::bail!("New sources.yaml exceeds 64KB size limit");
    }
    write_config_atomic(path, yaml.as_bytes(), |file, bytes| file.write_all(bytes))
}

fn write_config_atomic(
    path: &Path,
    bytes: &[u8],
    writer: impl FnOnce(&mut std::fs::File, &[u8]) -> std::io::Result<()>,
) -> Result<()> {
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(metadata) => Some(metadata),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => {
            return Err(error).with_context(|| format!("Cannot inspect {}", path.display()));
        }
    };
    let path = if metadata
        .as_ref()
        .is_some_and(|m| m.file_type().is_symlink())
    {
        std::fs::canonicalize(path)?
    } else {
        path.to_path_buf()
    };
    let permissions = match std::fs::metadata(&path) {
        Ok(metadata) => {
            if !metadata.is_file() {
                anyhow::bail!("Source configuration is not a file: {}", path.display());
            }
            if metadata.permissions().readonly() {
                anyhow::bail!("Source configuration is read only: {}", path.display());
            }
            Some(metadata.permissions())
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => {
            return Err(error).with_context(|| format!("Cannot inspect {}", path.display()));
        }
    };
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let serial = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_nanos();
    let temporary_path =
        path.with_extension(format!("{}.{stamp}.{serial}.tmp", std::process::id()));
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let file = options
        .open(&temporary_path)
        .with_context(|| format!("Cannot create {}", temporary_path.display()))?;
    let pending = PendingConfig(temporary_path);
    stage_config(file, bytes, permissions, writer)?;
    std::fs::rename(&pending.0, &path).with_context(|| format!("Cannot replace {}", path.display()))
}

fn stage_config(
    mut file: std::fs::File,
    bytes: &[u8],
    permissions: Option<std::fs::Permissions>,
    writer: impl FnOnce(&mut std::fs::File, &[u8]) -> std::io::Result<()>,
) -> Result<()> {
    writer(&mut file, bytes)?;
    if let Some(permissions) = permissions {
        file.set_permissions(permissions)?;
    }
    file.sync_all()?;
    Ok(())
}

struct PendingConfig(PathBuf);

impl Drop for PendingConfig {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

/// Expand `~` prefix to home directory. Handles `~/path`, `~\path`, and bare `~`.
pub fn expand_tilde(path: &Path) -> PathBuf {
    let s = path.to_string_lossy();
    if s == "~" {
        return crate::paths::home_dir().unwrap_or_else(|| path.to_path_buf());
    }
    if s.starts_with("~/") || s.starts_with("~\\") {
        if let Some(home) = crate::paths::home_dir() {
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
    if path.as_os_str().is_empty() {
        anyhow::bail!("Local source path is empty");
    }
    let expanded = expand_tilde(path);

    // Reject path traversal in the raw string
    let s = expanded.to_string_lossy();
    if s.contains("..") {
        anyhow::bail!("Path traversal (..) not allowed in source path: {}", s);
    }

    // Canonicalize to resolve symlinks, then re-check
    let canonical = expanded.canonicalize().with_context(|| {
        format!(
            "Local source directory does not exist or cannot be accessed: {}",
            expanded.display()
        )
    })?;
    if !canonical.is_dir() {
        anyhow::bail!(
            "Local source path is not a directory: {}",
            canonical.display()
        );
    }

    if let Some(home) = crate::paths::home_dir() {
        let claude_dir = home.join(".claude");
        let canonical_claude = claude_dir.canonicalize().unwrap_or(claude_dir);
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
        assert!(
            expanded.to_string_lossy().ends_with("foo/bar")
                || expanded.to_string_lossy().ends_with("foo\\bar")
        );
    }

    #[test]
    fn test_expand_tilde_bare() {
        let path = Path::new("~");
        let expanded = expand_tilde(path);
        if dirs::home_dir().is_some() {
            assert!(!expanded.to_string_lossy().contains('~'));
        }
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
        let fixture = ConfigFixture::new();
        assert!(validate_local_path(&fixture.0).is_ok());
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

        let yaml = serde_yaml_bw::to_string(&config).unwrap();
        let parsed: super::SourcesConfig = serde_yaml_bw::from_str(&yaml).unwrap();

        assert_eq!(parsed.sources.len(), 3);
        // Verify tag serialization produces "type: git" / "type: local"
        assert!(yaml.contains("type: git"));
        assert!(yaml.contains("type: local"));
        assert!(yaml.contains("https://github.com/user/repo.git"));
        assert!(yaml.contains("root: config/claude")); // root field serialized
        assert!(yaml.contains("map_to: rules")); // map_to field serialized
        assert!(!yaml.contains("auto_update")); // Omitted when None
    }

    struct ConfigFixture(PathBuf);

    impl ConfigFixture {
        fn new() -> Self {
            static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
            let serial = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let stamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "hibi_atomic_sources_{}_{stamp}_{serial}",
                std::process::id()
            ));
            std::fs::create_dir(&path).unwrap();
            Self(path)
        }

        fn path(&self) -> PathBuf {
            self.0.join("sources.yaml")
        }

        fn assert_no_staged_files(&self) {
            assert!(std::fs::read_dir(&self.0).unwrap().all(|entry| {
                entry
                    .unwrap()
                    .path()
                    .extension()
                    .is_none_or(|extension| extension != "tmp")
            }));
        }
    }

    impl Drop for ConfigFixture {
        fn drop(&mut self) {
            std::fs::remove_dir_all(&self.0).unwrap();
        }
    }

    #[test]
    fn partial_config_write_preserves_original_and_removes_temporary_file() {
        let fixture = ConfigFixture::new();
        let path = fixture.path();
        std::fs::write(&path, "sources: []\nauto_update: false\n").unwrap();
        let original = std::fs::read(&path).unwrap();
        // Inject a failure after actual staged bytes, as a disk-full write can do.
        let result = write_config_atomic(&path, b"replacement", |file, bytes| {
            file.write_all(&bytes[..4])?;
            Err(std::io::Error::other("simulated disk full"))
        });
        assert!(result.unwrap_err().to_string().contains("disk full"));
        assert_eq!(std::fs::read(&path).unwrap(), original);
        fixture.assert_no_staged_files();
        write_config_atomic(&path, b"sources: []\n", |file, bytes| file.write_all(bytes)).unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"sources: []\n");
        fixture.assert_no_staged_files();
    }

    #[test]
    fn failed_new_config_does_not_leave_partial_configuration() {
        let fixture = ConfigFixture::new();
        let result = write_config_atomic(&fixture.path(), b"incomplete", |file, bytes| {
            file.write_all(&bytes[..1])?;
            Err(std::io::Error::other("simulated interrupted write"))
        });
        assert!(result.is_err());
        assert!(!fixture.path().exists());
        fixture.assert_no_staged_files();
    }

    #[test]
    fn directory_destination_is_preserved_and_no_temporary_file_is_created() {
        let fixture = ConfigFixture::new();
        std::fs::create_dir(fixture.path()).unwrap();
        assert!(
            write_config_atomic(&fixture.path(), b"sources: []", |_, _| {
                panic!("directory destination must be rejected before staging");
            })
            .is_err()
        );
        assert!(fixture.path().is_dir());
        fixture.assert_no_staged_files();
    }

    #[cfg(unix)]
    #[test]
    fn replacement_preserves_permissions_and_new_config_is_private() {
        use std::os::unix::fs::PermissionsExt;
        let fixture = ConfigFixture::new();
        let path = fixture.path();
        write_config_atomic(&path, b"initial", |file, bytes| file.write_all(bytes)).unwrap();
        assert_eq!(
            std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o640)).unwrap();
        write_config_atomic(&path, b"replacement", |file, bytes| file.write_all(bytes)).unwrap();
        assert_eq!(
            std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o640
        );
        fixture.assert_no_staged_files();
    }

    #[cfg(unix)]
    #[test]
    fn read_only_configuration_is_not_bypassed_by_rename() {
        use std::os::unix::fs::PermissionsExt;
        let fixture = ConfigFixture::new();
        let path = fixture.path();
        std::fs::write(&path, b"original").unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o400)).unwrap();
        assert!(
            write_config_atomic(&path, b"replacement", |_, _| {
                panic!("read only configuration must be rejected before staging");
            })
            .is_err()
        );
        assert_eq!(std::fs::read(&path).unwrap(), b"original");
        fixture.assert_no_staged_files();
    }

    #[cfg(unix)]
    #[test]
    fn saving_a_symlink_updates_its_target_and_keeps_the_link() {
        let fixture = ConfigFixture::new();
        let target = fixture.0.join("linked-config.yaml");
        std::fs::write(&target, b"original").unwrap();
        std::os::unix::fs::symlink(&target, fixture.path()).unwrap();
        write_config_atomic(&fixture.path(), b"replacement", |file, bytes| {
            file.write_all(bytes)
        })
        .unwrap();
        assert!(
            std::fs::symlink_metadata(fixture.path())
                .unwrap()
                .file_type()
                .is_symlink()
        );
        assert_eq!(std::fs::read(&target).unwrap(), b"replacement");
        fixture.assert_no_staged_files();
    }
    #[test]
    fn saving_after_a_parse_failure_preserves_the_entire_original() {
        let fixture = ConfigFixture::new();
        let original = b"sources: [\n  important but malformed configuration\n";
        std::fs::write(fixture.path(), original).unwrap();
        assert!(load_config_at(&fixture.path()).is_err());
        let error = save_config_at(&fixture.path(), &[], true).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("existing contents could not be loaded")
        );
        assert_eq!(std::fs::read(fixture.path()).unwrap(), original);
        fixture.assert_no_staged_files();
    }

    #[test]
    fn oversized_existing_configuration_cannot_be_replaced_by_empty_defaults() {
        let fixture = ConfigFixture::new();
        let original = vec![b'#'; MAX_CONFIG_SIZE as usize + 1];
        std::fs::write(fixture.path(), &original).unwrap();
        assert!(save_config_at(&fixture.path(), &[], true).is_err());
        assert_eq!(std::fs::read(fixture.path()).unwrap(), original);
        fixture.assert_no_staged_files();
    }

    #[test]
    fn save_rejects_directory_configuration_before_staging() {
        let fixture = ConfigFixture::new();
        std::fs::create_dir(fixture.path()).unwrap();
        assert!(save_config_at(&fixture.path(), &[], true).is_err());
        assert!(fixture.path().is_dir());
        fixture.assert_no_staged_files();
    }

    #[test]
    fn valid_existing_and_absent_configuration_can_be_saved_and_loaded() {
        let fixture = ConfigFixture::new();
        let entry = SourceEntry::Local {
            path: PathBuf::from("/tmp/my-source"),
            root: None,
            map_to: None,
        };
        save_config_at(&fixture.path(), &[entry], false).unwrap();
        let (entries, auto_update) = load_config_at(&fixture.path()).unwrap();
        assert_eq!(entries.len(), 1);
        assert!(!auto_update);
        save_config_at(&fixture.path(), &[], true).unwrap();
        let (entries, auto_update) = load_config_at(&fixture.path()).unwrap();
        assert!(entries.is_empty());
        assert!(auto_update);
        fixture.assert_no_staged_files();
    }

    #[test]
    fn invalid_utf8_original_is_preserved() {
        let fixture = ConfigFixture::new();
        let original = [0xff, 0xfe, 0xf0];
        std::fs::write(fixture.path(), original).unwrap();
        assert!(save_config_at(&fixture.path(), &[], true).is_err());
        assert_eq!(std::fs::read(fixture.path()).unwrap(), original);
        fixture.assert_no_staged_files();
    }

    #[test]
    fn oversized_new_configuration_is_not_written() {
        let fixture = ConfigFixture::new();
        let original = b"sources: []\nauto_update: false\n";
        std::fs::write(fixture.path(), original).unwrap();
        let entry = SourceEntry::Local {
            path: PathBuf::from("a".repeat(MAX_CONFIG_SIZE as usize)),
            root: None,
            map_to: None,
        };
        assert!(save_config_at(&fixture.path(), &[entry], true).is_err());
        assert_eq!(std::fs::read(fixture.path()).unwrap(), original);
        fixture.assert_no_staged_files();
    }

    #[cfg(unix)]
    #[test]
    fn malformed_symlink_target_is_not_replaced_or_unlinked() {
        let fixture = ConfigFixture::new();
        let target = fixture.0.join("linked-config.yaml");
        let original = b"sources: [invalid";
        std::fs::write(&target, original).unwrap();
        std::os::unix::fs::symlink(&target, fixture.path()).unwrap();
        assert!(save_config_at(&fixture.path(), &[], true).is_err());
        assert!(
            std::fs::symlink_metadata(fixture.path())
                .unwrap()
                .file_type()
                .is_symlink()
        );
        assert_eq!(std::fs::read(target).unwrap(), original);
        fixture.assert_no_staged_files();
    }

    #[test]
    fn empty_and_missing_local_source_paths_are_rejected() {
        assert!(
            validate_local_path(Path::new(""))
                .unwrap_err()
                .to_string()
                .contains("empty")
        );
        let fixture = ConfigFixture::new();
        assert!(validate_local_path(&fixture.0.join("missing-directory")).is_err());
    }

    #[test]
    fn regular_file_cannot_be_added_as_local_source_directory() {
        let fixture = ConfigFixture::new();
        std::fs::write(fixture.path(), b"some file").unwrap();
        let error = validate_local_path(&fixture.path()).unwrap_err();
        assert!(error.to_string().contains("not a directory"));
    }

    #[cfg(unix)]
    #[test]
    fn local_source_symlink_must_resolve_to_existing_directory() {
        let fixture = ConfigFixture::new();
        let directory = fixture.0.join("source-directory");
        std::fs::create_dir(&directory).unwrap();
        let link = fixture.0.join("source-link");
        std::os::unix::fs::symlink(&directory, &link).unwrap();
        assert!(validate_local_path(&link).is_ok());
        std::fs::remove_dir(&directory).unwrap();
        assert!(validate_local_path(&link).is_err());
    }
}
