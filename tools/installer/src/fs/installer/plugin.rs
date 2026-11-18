use anyhow::Result;
use std::sync::mpsc::Receiver;

use super::mcp::ensure_marketplace_added;
use super::process::{ProcessConfig, spawn_cancelable_process};
use crate::fs::create_cli_command;
use crate::plugin::Plugin;
use crate::target::TargetCli;

pub fn install_plugin(plugin: &Plugin, timeout_secs: u64, cancel_rx: &Receiver<()>) -> Result<()> {
    ensure_marketplace_added(
        &plugin.def.marketplace,
        &plugin.def.source,
        timeout_secs,
        cancel_rx,
    )?;

    let mut command = create_cli_command(TargetCli::Claude);
    command.args(["plugin", "install", &plugin.def.id()]);

    spawn_cancelable_process(
        &mut command,
        ProcessConfig {
            timeout_secs,
            cancel_rx,
            item_name: &plugin.def.name,
            action: "install plugin",
        },
    )
}

pub fn remove_plugin(plugin: &Plugin, timeout_secs: u64, cancel_rx: &Receiver<()>) -> Result<()> {
    let mut command = create_cli_command(TargetCli::Claude);
    command.args(["plugin", "uninstall", &plugin.def.id()]);

    spawn_cancelable_process(
        &mut command,
        ProcessConfig {
            timeout_secs,
            cancel_rx,
            item_name: &plugin.def.name,
            action: "remove plugin",
        },
    )
}
