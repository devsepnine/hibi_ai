use anyhow::{Context, Result};
use serde_json::{Map, Value};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use super::validation::validate_mcp_server;
use crate::mcp::{McpCatalog, McpServer, McpStatus};

/// Scan MCP catalog and mark each server as installed or not.
pub(super) fn scan_with_installed(
    source_dir: &Path,
    installed: Option<&[String]>,
) -> Result<Vec<McpServer>> {
    let catalog_path = source_dir.join("mcps/mcps.yaml");
    if !catalog_path.exists() {
        return Ok(Vec::new());
    }

    let content = std::fs::read_to_string(&catalog_path)?;
    let catalog: McpCatalog = serde_yaml_bw::from_str(&content)?;

    let servers = catalog
        .servers
        .into_iter()
        .filter_map(|def| {
            if validate_mcp_server(&def).is_some() {
                return None;
            }
            let status = match installed {
                Some(names) if names.contains(&def.name) => McpStatus::Installed,
                Some(_) => McpStatus::NotInstalled,
                None => McpStatus::Unknown,
            };
            Some(McpServer::new(def, status))
        })
        .collect();

    Ok(servers)
}

/// A removal can only target the selected scope. Claude's list merges scopes.
pub(super) fn installed_claude_names(project: Option<&Path>) -> Result<Vec<String>> {
    let config_dir = match std::env::var_os("CLAUDE_CONFIG_DIR").filter(|path| !path.is_empty()) {
        Some(dir) => project.unwrap_or(Path::new(".")).join(dir),
        None => crate::paths::require_home_dir()?,
    };
    let registry = config_dir.join(".claude.json");
    read_claude_names(&registry, project)
        .map_err(|error| anyhow::anyhow!("{error}. Check {}", registry.display()))
}

fn read_claude_names(registry: &Path, project: Option<&Path>) -> Result<Vec<String>> {
    let project = project.map(claude_project_key).transpose()?;
    let data = match std::fs::read(registry) {
        Ok(data) => data,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error).context("Cannot read Claude MCP registrations"),
    };
    let root: Map<String, Value> = serde_json::from_slice(&data).map_err(|error| {
        anyhow::anyhow!(
            "Invalid Claude configuration at line {}, column {}",
            error.line(),
            error.column()
        )
    })?;
    let scope = match project {
        Some(key) => {
            let Some(entry) = optional_object(&root, "projects")?.and_then(|map| map.get(&key))
            else {
                return Ok(Vec::new());
            };
            entry.as_object().context("Invalid Claude project entry")?
        }
        None => &root,
    };
    Ok(optional_object(scope, "mcpServers")?
        .into_iter()
        .flat_map(|map| map.keys().cloned())
        .collect())
}

fn optional_object<'a>(
    root: &'a Map<String, Value>,
    name: &str,
) -> Result<Option<&'a Map<String, Value>>> {
    root.get(name)
        .map(|value| {
            value
                .as_object()
                .with_context(|| format!("Invalid Claude configuration field {name}"))
        })
        .transpose()
}

fn claude_project_key(project: &Path) -> Result<String> {
    let project = project
        .canonicalize()
        .context("Cannot resolve the selected Claude MCP project")?;
    anyhow::ensure!(
        project.is_dir(),
        "The selected Claude MCP project is not a directory"
    );
    let root = claude_project_root(&project)?;
    let key = root
        .to_str()
        .context("Invalid text in Claude MCP project path")?;
    #[cfg(windows)]
    let key = windows_project_key(key);
    #[cfg(not(windows))]
    let key = key.to_owned();
    Ok(key)
}

#[cfg(any(windows, test))]
fn windows_project_key(path: &str) -> String {
    // Claude stores forward slashes and preserves drive letter case.
    if let Some(path) = path.strip_prefix(r"\\?\UNC\") {
        return format!("//{}", path.replace('\\', "/"));
    }
    path.strip_prefix(r"\\?\")
        .unwrap_or(path)
        .replace('\\', "/")
}

fn claude_project_root(project: &Path) -> Result<PathBuf> {
    for dir in project.ancestors() {
        let git = dir.join(".git");
        let metadata = match std::fs::metadata(&git) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == ErrorKind::NotFound => continue,
            Err(error) => return Err(error).context("Cannot inspect the selected Git project"),
        };
        if metadata.is_file() {
            let git_file = std::fs::read_to_string(&git).context("Cannot read the Git link")?;
            let git_dir = git_file
                .trim_end_matches(['\r', '\n'])
                .strip_prefix("gitdir: ")
                .context("Invalid Git link in the selected Claude MCP project")?;
            let git_dir = dir.join(git_dir);
            // Claude shares local registrations between a linked worktree and its main checkout.
            match std::fs::read_to_string(git_dir.join("commondir")) {
                Ok(path) => {
                    let shared = git_dir
                        .join(path.trim_end_matches(['\r', '\n']))
                        .canonicalize()
                        .context("Cannot resolve the main Git checkout")?;
                    if shared.ends_with(".git") {
                        return Ok(shared.parent().context("Invalid main Git checkout")?.into());
                    }
                }
                Err(error) if error.kind() == ErrorKind::NotFound => {}
                Err(error) => return Err(error).context("Cannot read the main Git checkout path"),
            }
        }
        return Ok(dir.into());
    }
    Ok(project.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;

    #[test]
    fn claude_windows_project_key_preserves_case_and_uses_forward_slashes() {
        assert_eq!(windows_project_key(r"\\?\d:\Dev\Project"), "d:/Dev/Project");
        assert_eq!(
            windows_project_key(r"\\?\UNC\host\share\project"),
            "//host/share/project"
        );
    }

    #[test]
    fn claude_mcp_status_uses_only_registrations_in_the_selected_scope() -> Result<()> {
        let fixture = tempfile::tempdir()?;
        let project = fixture.path().join("checkout");
        let nested = project.join("nested");
        let other = fixture.path().join("other");
        fs::create_dir_all(&nested)?;
        fs::create_dir_all(project.join(".git"))?;
        fs::create_dir(&other)?;
        fs::write(
            project.join(".mcp.json"),
            r#"{"mcpServers":{"project-only":{}}}"#,
        )?;
        let registry = fixture.path().join(".claude.json");
        fs::write(
            &registry,
            serde_json::to_vec(&json!({
                "mcpServers": {"shared": {}, "user-only": {}},
                "projects": {
                    claude_project_key(&project)?: {
                        "mcpServers": {"shared": {}, "local-only": {}},
                    },
                    claude_project_key(&other)?: {
                        "mcpServers": {"other-only": {}},
                    },
                },
            }))?,
        )?;

        assert_eq!(read_claude_names(&registry, None)?, ["shared", "user-only"]);
        assert_eq!(
            read_claude_names(&registry, Some(&nested))?,
            ["local-only", "shared"]
        );
        assert_eq!(read_claude_names(&registry, Some(&other))?, ["other-only"]);

        let worktree = fixture.path().join("worktree");
        let worktree_metadata = project.join(".git/worktrees/linked");
        fs::create_dir_all(&worktree_metadata)?;
        fs::create_dir(&worktree)?;
        fs::write(
            worktree.join(".git"),
            "gitdir: ../checkout/.git/worktrees/linked\n",
        )?;
        fs::write(worktree_metadata.join("commondir"), "../..\n")?;
        assert_eq!(
            read_claude_names(&registry, Some(&worktree))?,
            ["local-only", "shared"]
        );
        Ok(())
    }

    #[test]
    fn claude_mcp_scan_distinguishes_absent_and_invalid_registries() -> Result<()> {
        let fixture = tempfile::tempdir()?;
        let registry = fixture.path().join(".claude.json");
        assert!(read_claude_names(&registry, None)?.is_empty());
        assert!(read_claude_names(&registry, Some(&fixture.path().join("missing"))).is_err());

        for data in [r#"{"mcpServers":"private-value"}"#, r#"{"private-value":"#] {
            fs::write(&registry, data)?;
            let error = read_claude_names(&registry, None).unwrap_err();
            assert!(!error.to_string().contains("private-value"));
        }
        fs::create_dir(fixture.path().join("directory"))?;
        assert!(read_claude_names(&fixture.path().join("directory"), None).is_err());
        Ok(())
    }
}
