use std::sync::mpsc::Receiver;
use anyhow::Result;

use crate::plugin::Plugin;
use super::process::{spawn_cancelable_process, run_cleanup_command, ProcessConfig};
use super::mcp::ensure_marketplace_added;
use crate::fs::create_claude_command;

/// Cleanup helper: try to remove plugin without blocking
/// Returns true if cleanup succeeded, false otherwise
fn cleanup_plugin_installation(plugin: &Plugin) -> bool {
    let mut command = create_claude_command();
    command.args(["plugin", "uninstall", &plugin.def.name]);
    run_cleanup_command(&mut command)
}

pub fn install_plugin(
    plugin: &Plugin,
    timeout_secs: u64,
    cancel_rx: &Receiver<()>,
) -> Result<()> {
    ensure_marketplace_added(
        &plugin.def.marketplace,
        &plugin.def.source,
        timeout_secs,
        cancel_rx,
    )?;

    let plugin_ref = format!("{}@{}", plugin.def.name, plugin.def.marketplace);
    let mut command = create_claude_command();
    command.args(["plugin", "install", &plugin_ref]);

    let plugin_clone = plugin.clone();
    spawn_cancelable_process(
        &mut command,
        ProcessConfig {
            timeout_secs,
            cancel_rx,
            item_name: &plugin.def.name,
            action: "install plugin",
            cleanup: Some(Box::new(move || cleanup_plugin_installation(&plugin_clone))),
        },
    )
}

pub fn remove_plugin(
    plugin: &Plugin,
    timeout_secs: u64,
    cancel_rx: &Receiver<()>,
) -> Result<()> {
    let mut command = create_claude_command();
    command.args(["plugin", "uninstall", &plugin.def.name]);

    spawn_cancelable_process(
        &mut command,
        ProcessConfig {
            timeout_secs,
            cancel_rx,
            item_name: &plugin.def.name,
            action: "remove plugin",
            cleanup: None,
        },
    )
}
