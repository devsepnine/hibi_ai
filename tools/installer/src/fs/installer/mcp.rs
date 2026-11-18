use anyhow::{Context, Result, ensure};
use std::process::Command;
use std::sync::mpsc::Receiver;

use super::process::{
    ProcessConfig, ProcessExit, QUICK_COMMAND_TIMEOUT_SECS, run_until_output, run_with_timeout,
    spawn_cancelable_process,
};
use crate::fs::create_cli_command;
use crate::mcp::{McpScope, McpServer, McpSetup};
use crate::target::TargetCli;

/// Split a command string into arguments.
/// Uses shlex (POSIX rules) on Unix, Windows-aware splitting on Windows.
fn split_command(cmd: &str) -> Option<Vec<String>> {
    #[cfg(windows)]
    {
        // Windows: split on whitespace, respecting double and single-quoted strings.
        // Backslashes are literal (not escape chars) to support Windows paths.
        let mut args = Vec::new();
        let mut current = String::new();
        let mut quote_char: Option<char> = None;
        for ch in cmd.chars() {
            match ch {
                '"' | '\'' if quote_char == Some(ch) => {
                    quote_char = None;
                }
                '"' | '\'' if quote_char.is_none() => {
                    quote_char = Some(ch);
                }
                ' ' | '\t' if quote_char.is_none() => {
                    if !current.is_empty() {
                        args.push(std::mem::take(&mut current));
                    }
                }
                _ => current.push(ch),
            }
        }
        if !current.is_empty() {
            args.push(current);
        }
        if quote_char.is_some() {
            None
        } else {
            Some(args)
        }
    }
    #[cfg(not(windows))]
    {
        shlex::split(cmd)
    }
}

fn removal_command(
    server: &McpServer,
    target_cli: TargetCli,
    scope: McpScope,
    project_path: Option<&str>,
) -> Command {
    let mut command = create_cli_command(target_cli);
    command.args(["mcp", "remove", &server.def.name]);
    if target_cli == TargetCli::Claude {
        command.args(["--scope", scope.display()]);
        if let Some(path) = project_path {
            command.current_dir(path);
        }
    }
    command
}

/// Configuration for MCP server installation.
pub struct McpInstallConfig<'a> {
    pub scope: McpScope,
    pub project_path: Option<&'a str>,
    pub env_values: &'a [(String, String)],
    pub target_cli: TargetCli,
    pub timeout_secs: u64,
    pub cancel_rx: &'a Receiver<()>,
}

pub enum McpRegistration {
    AlreadyRegistered,
    Registered(Option<String>),
}

// Codex exposes separate list and add operations. Serialize that pair across installer instances.
fn lock_codex_registration() -> Result<std::fs::File> {
    let dir = crate::paths::require_home_dir()?.join(".hibi");
    std::fs::create_dir_all(&dir)?;
    let lock = std::fs::File::options()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(dir.join("codex-mcp.lock"))
        .context("Cannot open the installer lock for Codex MCP")?;
    lock.try_lock().context(
        "Cannot lock Codex MCP registrations. Check for another active installer before retrying",
    )?;
    Ok(lock)
}

pub fn install_mcp_server(server: &McpServer, config: McpInstallConfig) -> Result<McpRegistration> {
    if config.cancel_rx.try_recv().is_ok() {
        anyhow::bail!("Cancelled by user");
    }
    let _registration_lock = if config.target_cli == TargetCli::Codex {
        let lock = lock_codex_registration()?;
        if crate::fs::codex_mcp_names()?.contains(&server.def.name) {
            return Ok(McpRegistration::AlreadyRegistered);
        }
        Some(lock)
    } else {
        None
    };
    let setup = server.def.registration_setup(|key| std::env::var_os(key));
    let mut command = registration_command(server, &config, setup)?;
    let codex_http = config.target_cli == TargetCli::Codex && server.is_http();
    let process = ProcessConfig {
        timeout_secs: config.timeout_secs,
        cancel_rx: config.cancel_rx,
        item_name: &server.def.name,
        action: "register MCP server",
    };
    let marker = codex_http.then_some("Detected OAuth support. Starting OAuth flow");
    let exit = run_until_output(&mut command, process, marker)?;
    if exit == ProcessExit::MatchedOutput {
        let mut get = create_cli_command(TargetCli::Codex);
        get.args(["mcp", "get", "--json", &server.def.name]);
        let output = run_with_timeout(&mut get, QUICK_COMMAND_TIMEOUT_SECS)?;
        ensure!(
            output.status.success(),
            "Could not confirm registration of {}",
            server.def.name
        );
        let entry: serde_json::Value = serde_json::from_slice(&output.stdout)?;
        ensure!(
            entry["transport"]["url"].as_str() == server.def.url.as_deref(),
            "Registered endpoint differs for {}",
            server.def.name
        );
        return Ok(McpRegistration::Registered(Some(format!(
            "Finish login with: codex mcp login {}",
            server.def.name
        ))));
    }
    let hint = match setup {
        Some(McpSetup::Bearer { variable, .. }) => {
            Some(format!("Set {variable} in your MCP client's environment."))
        }
        _ if config.target_cli == TargetCli::Claude && server.is_http() => {
            Some("Check connection and authentication in /mcp.".into())
        }
        _ => None,
    };
    Ok(McpRegistration::Registered(hint))
}

fn registration_command(
    server: &McpServer,
    config: &McpInstallConfig<'_>,
    setup: Option<&McpSetup>,
) -> Result<Command> {
    let mut command = create_cli_command(config.target_cli);
    command.args(["mcp", "add"]);
    let claude = config.target_cli == TargetCli::Claude;
    if claude {
        command.args(["--scope", config.scope.display()]);
        if let Some(path) = config.project_path {
            command.current_dir(path);
        }
    }
    command.arg(&server.def.name);
    let env_flag = if claude { "-e" } else { "--env" };
    for (key, value) in config.env_values {
        if setup.and_then(McpSetup::input) != Some(key) {
            command.arg(env_flag).arg(format!("{key}={value}"));
        }
    }
    if let Some(setup) = setup {
        match setup {
            McpSetup::Bearer { variable, .. } if claude => {
                command.args([
                    "--header",
                    &format!("Authorization: Bearer ${{{variable}}}"),
                ]);
            }
            McpSetup::Bearer { variable, .. } => {
                command.args(["--bearer-token-env-var", variable]);
            }
            McpSetup::FileEnv { variable } => {
                let value = config
                    .env_values
                    .iter()
                    .find(|(key, _)| key == variable)
                    .map(|(_, value)| value.clone())
                    .or_else(|| std::env::var(variable).ok())
                    .with_context(|| format!("Missing {variable}"))?;
                let path = crate::mcp::writable_storage_file(&value)
                    .with_context(|| format!("Invalid {variable}. {}", setup.hint()))?;
                let mut assignment = std::ffi::OsString::from(format!("{variable}="));
                assignment.push(path);
                command.arg(env_flag).arg(assignment);
            }
        }
    }
    if server.is_http() {
        if claude {
            command.args(["-t", "http"]);
        } else {
            command.arg("--url");
        }
        command.arg(server.def.url.as_ref().context("Missing MCP endpoint")?);
    } else {
        let parts = server
            .def
            .command
            .as_deref()
            .and_then(split_command)
            .context("Missing or invalid MCP command")?;
        ensure!(!parts.is_empty(), "Empty MCP command");
        command.arg("--").args(parts);
    }
    Ok(command)
}

pub fn remove_mcp_server(
    server: &McpServer,
    target_cli: TargetCli,
    scope: McpScope,
    project_path: Option<&str>,
    timeout_secs: u64,
    cancel_rx: &Receiver<()>,
) -> Result<()> {
    let mut command = removal_command(server, target_cli, scope, project_path);
    spawn_cancelable_process(
        &mut command,
        ProcessConfig {
            timeout_secs,
            cancel_rx,
            item_name: &server.def.name,
            action: "remove MCP server",
        },
    )
}

/// Ensure the plugin's marketplace is registered before installation.
///
/// Step 1: Quick check if already added (non-cancelable, short timeout).
/// Step 2: If not found, add it using `spawn_cancelable_process` (cancelable, respects timeout).
pub(super) fn ensure_marketplace_added(
    marketplace: &str,
    source: &str,
    timeout_secs: u64,
    cancel_rx: &Receiver<()>,
) -> Result<()> {
    // Step 1: Check if marketplace is already added (quick, non-cancelable)
    let mut list_cmd = create_cli_command(TargetCli::Claude);
    list_cmd.args(["plugin", "marketplace", "list", "--json"]);

    match run_with_timeout(&mut list_cmd, QUICK_COMMAND_TIMEOUT_SECS) {
        Ok(list_output) if list_output.status.success() => {
            #[derive(serde::Deserialize)]
            struct Marketplace {
                name: String,
            }
            let entries: Vec<Marketplace> = serde_json::from_slice(&list_output.stdout)
                .context("Invalid Claude marketplace list")?;
            if entries.iter().any(|entry| entry.name == marketplace) {
                return Ok(()); // Already added
            }
        }
        Ok(_) => {}  // Non-success status: fall through to add
        Err(_) => {} // Timeout or spawn error: fall through to add
    }

    // Step 2: Add the marketplace (cancelable, full timeout)
    let mut command = create_cli_command(TargetCli::Claude);
    command.args(["plugin", "marketplace", "add", source]);

    spawn_cancelable_process(
        &mut command,
        ProcessConfig {
            timeout_secs,
            cancel_rx,
            item_name: marketplace,
            action: "add marketplace",
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::{McpCatalog, McpStatus};

    fn bundled(name: &str) -> McpServer {
        let catalog: McpCatalog =
            serde_yaml_bw::from_str(include_str!("../../../../../src/mcps/mcps.yaml")).unwrap();
        McpServer::new(
            catalog
                .servers
                .into_iter()
                .find(|s| s.name == name)
                .unwrap(),
            McpStatus::NotInstalled,
        )
    }

    fn args(
        server: &McpServer,
        cli: TargetCli,
        values: &[(String, String)],
        setup: Option<&McpSetup>,
    ) -> Result<Vec<String>> {
        let (_, rx) = std::sync::mpsc::channel();
        let command = registration_command(
            server,
            &McpInstallConfig {
                scope: McpScope::User,
                project_path: None,
                env_values: values,
                target_cli: cli,
                timeout_secs: 10,
                cancel_rx: &rx,
            },
            setup,
        )?;
        Ok(command
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect())
    }

    #[test]
    fn context7_registers_a_token_reference_only_when_configured() {
        let server = bundled("context7");

        for (cli, flag, reference) in [
            (
                TargetCli::Claude,
                "--header",
                "Authorization: Bearer ${CONTEXT7_API_KEY}",
            ),
            (
                TargetCli::Codex,
                "--bearer-token-env-var",
                "CONTEXT7_API_KEY",
            ),
        ] {
            for value in [None, Some(""), Some("fixture-key")] {
                let setup = server.def.registration_setup(|key| {
                    assert_eq!(key, "CONTEXT7_API_KEY");
                    value.map(Into::into)
                });
                let arguments = args(&server, cli, &[], setup).unwrap();
                assert_eq!(arguments.last().unwrap(), "https://mcp.context7.com/mcp");
                let setting = arguments.windows(2).find(|pair| pair[0] == flag);
                assert_eq!(
                    setting.map(|pair| pair[1].as_str()),
                    value.filter(|v| !v.is_empty()).map(|_| reference)
                );
                assert!(!arguments.iter().any(|arg| arg.contains("fixture-key")));
            }
        }
    }

    #[test]
    fn memory_accepts_writable_graph_paths_without_modifying_the_graph() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("한 글.jsonl");
        let value = path.to_str().unwrap();
        let server = bundled("memory");
        for cli in [TargetCli::Claude, TargetCli::Codex] {
            for bad in [
                "relative.jsonl",
                root.path().to_str().unwrap(),
                &format!("{value}/"),
            ] {
                assert!(
                    args(
                        &server,
                        cli,
                        &[("MEMORY_FILE_PATH".into(), bad.into())],
                        server.def.setup.as_ref()
                    )
                    .is_err()
                );
            }
            let arguments = args(
                &server,
                cli,
                &[("MEMORY_FILE_PATH".into(), value.into())],
                server.def.setup.as_ref(),
            )
            .unwrap();
            assert_eq!(
                arguments
                    .iter()
                    .filter(|arg| **arg == format!("MEMORY_FILE_PATH={value}"))
                    .count(),
                1
            );
            if path.exists() {
                assert_eq!(std::fs::read(&path).unwrap(), b"existing graph");
            } else {
                std::fs::write(&path, "existing graph").unwrap();
            }
        }
        assert_eq!(root.path().read_dir().unwrap().count(), 1);
    }

    #[cfg(unix)]
    #[test]
    fn memory_registers_the_symlink_target_for_atomic_replacements() {
        let root = tempfile::tempdir().unwrap();
        let target = root.path().join("graph.jsonl");
        std::fs::write(&target, "").unwrap();
        let link = root.path().join("alias.jsonl");
        std::os::unix::fs::symlink(&target, &link).unwrap();
        let server = bundled("memory");
        for cli in [TargetCli::Codex, TargetCli::Claude] {
            let arguments = args(
                &server,
                cli,
                &[("MEMORY_FILE_PATH".into(), link.display().to_string())],
                server.def.setup.as_ref(),
            )
            .unwrap();
            assert!(arguments.contains(&format!(
                "MEMORY_FILE_PATH={}",
                target.canonicalize().unwrap().display()
            )));
        }
        assert!(link.is_symlink());
    }

    #[cfg(unix)]
    #[test]
    fn memory_checks_the_parent_even_when_the_graph_exists() {
        use std::os::unix::fs::PermissionsExt;
        let root = tempfile::tempdir().unwrap();
        let graph = root.path().join("graph.jsonl");
        std::fs::write(&graph, "preserve").unwrap();
        let server = bundled("memory");
        let setup = server.def.setup.unwrap();
        std::fs::set_permissions(root.path(), std::fs::Permissions::from_mode(0o555)).unwrap();
        // Privileged test runners can still create the lock in this directory.
        let can_create_lock = std::fs::File::create(root.path().join("graph.lock")).is_ok();
        let existing = setup.accepts(graph.to_str().unwrap());
        let missing = setup.accepts(root.path().join("new.jsonl").to_str().unwrap());
        std::fs::set_permissions(root.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        assert_eq!((existing, missing), (can_create_lock, can_create_lock));
        assert_eq!(std::fs::read(graph).unwrap(), b"preserve");
    }
}
