use std::sync::mpsc::Sender;
use std::thread;

use anyhow::Result;

use crate::app::{App, Tab, TargetCli};
use crate::component;
use crate::fs;
use crate::mcp;
use crate::plugin;

/// Outcome of a scan thread, scoped to the kind of data it actually
/// touched. Used to skip the expensive `claude/codex mcp list` shell
/// call (which dominates refresh wall time) when an install only
/// affected filesystem-backed components or plugin metadata.
pub(crate) enum RefreshResult {
    /// First-time scan after CLI selection. Returns the full picture so
    /// every tab has something to render, and reports any deprecated
    /// hooks that were silently cleaned up during the load.
    InitialLoad {
        components: Vec<component::Component>,
        mcp_servers: Vec<mcp::McpServer>,
        plugins: Vec<plugin::Plugin>,
        cleaned_hooks: Vec<String>,
    },
    /// Refresh limited to filesystem-backed component types — Agents,
    /// Commands, Contexts, Rules, Skills, Hooks, Styles, Statusline,
    /// Config. Pure filesystem ops, sub-100ms typical.
    Components {
        components: Vec<component::Component>,
        /// Why the provenance manifest could not be written, if it failed.
        /// Carried back instead of logged in the worker so the TUI thread
        /// stays the only writer of the processing log.
        manifest_warning: Option<String>,
    },
    /// Refresh limited to MCP servers. Always shells out to the CLI
    /// (`mcp list`) and is the slow path — only run when the user
    /// actually installed/removed an MCP server.
    Mcp(Vec<mcp::McpServer>),
    /// Refresh limited to plugins (filesystem only).
    Plugins(Vec<plugin::Plugin>),
}

/// Which scan to run for a post-install refresh, derived from the tab
/// that initiated the install/remove. Kept private to this module so
/// `RefreshResult` remains the single public boundary between scan
/// threads and the app state.
enum RefreshScope {
    Components,
    Mcp,
    Plugins,
}

impl RefreshScope {
    fn for_tab(tab: Tab) -> Self {
        match tab {
            Tab::McpServers => Self::Mcp,
            Tab::Plugins => Self::Plugins,
            _ => Self::Components,
        }
    }
}

/// Start a background thread to rescan only the data the just-finished
/// install/remove could have changed.
///
/// Before this was scope-aware, every refresh re-ran all three scans
/// including `claude/codex mcp list` — a CLI shell-out with up to a
/// 30-second timeout — even when the user installed e.g. an Agent
/// (filesystem-only). Now we look at `app.tab` to pick the single scan
/// that's actually relevant, dropping non-MCP refresh latency from
/// "several seconds" to "tens of milliseconds".
pub(super) fn start_refresh_thread(app: &mut App, refresh_tx: &Sender<Result<RefreshResult>>) {
    app.refreshing = true;

    let tx_clone = refresh_tx.clone();
    let sources = app.sources.clone();
    let dest_dir = app.dest_dir.clone();
    let target_cli = app.target_cli.unwrap_or(TargetCli::Claude);
    let scope = RefreshScope::for_tab(app.tab);

    thread::spawn(move || {
        let result = match scope {
            RefreshScope::Components => fs::scanner::scan_all_sources(&sources, &dest_dir, target_cli)
                .map(|components| {
                    // Record provenance here rather than in the consumer: the
                    // consumer runs on the TUI tick, and a home directory can
                    // be network-mounted or virus-scanned, which would stall
                    // the very loop this thread exists to keep free.
                    let manifest_warning =
                        manifest_warning_from(fs::manifest::write(&dest_dir, &components));
                    RefreshResult::Components { components, manifest_warning }
                }),
            RefreshScope::Mcp => fs::scanner::scan_all_mcp_sources(&sources, target_cli)
                .map(|(servers, _warning)| RefreshResult::Mcp(servers)),
            RefreshScope::Plugins => fs::scanner::scan_all_plugin_sources(&sources)
                .map(RefreshResult::Plugins),
        };
        let _ = tx_clone.send(result);
    });
}

/// Turn a manifest write result into the warning to carry back to the TUI.
///
/// Split out of the worker closure because the closure itself cannot run in a
/// test without writing to a real home directory, which left the
/// failure-reporting path — the whole reason a failed write does not fail the
/// install — unverified.
fn manifest_warning_from(result: Result<()>) -> Option<String> {
    result.err().map(|e| format!("{:#}", e))
}

/// Log line for a manifest that could not be written.
///
/// The install itself already succeeded at this point, so the wording says
/// what was skipped rather than implying the install failed.
pub(super) fn manifest_warning_line(reason: &str) -> String {
    format!("[WARN] Install manifest not written: {}", reason)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_successful_manifest_write_produces_no_warning() {
        assert_eq!(manifest_warning_from(Ok(())), None);
    }

    #[test]
    fn a_failed_manifest_write_carries_its_full_context() {
        // `{:#}` rather than `{}`: the outer context alone ("Failed to
        // replace ...") does not say why, and the reason is all the user gets.
        let err = anyhow::anyhow!("Permission denied")
            .context("Failed to replace /Users/x/.hibi/install.json");

        let warning = manifest_warning_from(Err(err)).expect("a failure must warn");

        assert!(warning.contains("Failed to replace"), "got: {warning}");
        assert!(warning.contains("Permission denied"), "got: {warning}");
    }

    #[test]
    fn the_warning_line_says_what_was_skipped_not_that_the_install_failed() {
        // The components are already on disk when this fires, so the line must
        // not read as a failed install.
        let line = manifest_warning_line("Failed to create /Users/x/.hibi");

        assert!(line.starts_with("[WARN] Install manifest not written:"));
        assert!(line.contains("Failed to create /Users/x/.hibi"));
        assert!(!line.contains("[ERROR]"));
    }

    #[test]
    fn refresh_scope_routes_by_tab() {
        // Locks in the optimization: only MCP/Plugin tabs trigger the
        // matching scan. Every component tab routes to the cheap
        // filesystem-only Components scan, skipping `mcp list` entirely.
        assert!(matches!(RefreshScope::for_tab(Tab::McpServers), RefreshScope::Mcp));
        assert!(matches!(RefreshScope::for_tab(Tab::Plugins), RefreshScope::Plugins));
        for tab in [
            Tab::Agents, Tab::Commands, Tab::Contexts, Tab::Rules, Tab::Skills,
            Tab::Hooks, Tab::OutputStyles, Tab::Statusline, Tab::Config,
        ] {
            assert!(
                matches!(RefreshScope::for_tab(tab), RefreshScope::Components),
                "tab {:?} should route to Components scope",
                tab,
            );
        }
    }
}
