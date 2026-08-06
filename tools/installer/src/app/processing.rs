use anyhow::Result;

use super::types::{Tab, View};
use super::{App, build_tree_views};
use crate::component::Component;
use crate::mcp::McpServer;
use crate::plugin::Plugin;

/// Run the CLI presence pre-flight only for tabs that actually shell out
/// to `claude`/`codex`. Components are pure filesystem ops and don't need
/// the CLI; running the probe for them would just slow the flow down.
pub(crate) fn needs_cli_preflight(tab: Tab) -> bool {
    matches!(tab, Tab::Plugins | Tab::McpServers)
}

impl App {
    pub fn install_selected(&mut self) -> Result<()> {
        // Build install queue
        let indices: Vec<usize> = if self.tab == Tab::McpServers {
            self.mcp_servers
                .iter()
                .enumerate()
                .filter(|(_, m)| m.selected)
                .map(|(i, _)| i)
                .collect()
        } else if self.tab == Tab::Plugins {
            self.plugins
                .iter()
                .enumerate()
                .filter(|(_, p)| p.selected)
                .map(|(i, _)| i)
                .collect()
        } else if let Some(comp_type) = self.tab.to_component_type() {
            self.components
                .iter()
                .enumerate()
                .filter(|(_, c)| {
                    c.selected && c.component_type == comp_type && c.is_install_eligible()
                })
                .map(|(i, _)| i)
                .collect()
        } else {
            Vec::new()
        };

        if indices.is_empty() {
            self.status_message = Some("No items selected".to_string());
            return Ok(());
        }

        self.processing_queue = indices;
        self.is_removing = false;

        // Plugin/MCP install shells out to the CLI; hand off to the
        // Preflighting view so the 8-second `--version` probe runs on
        // a background thread instead of blocking the TUI tick.
        // `handle_preflighting_view` spawns the worker on first entry.
        // Component tabs are pure filesystem ops and skip the probe.
        if needs_cli_preflight(self.tab) {
            self.current_view = View::Preflighting;
            return Ok(());
        }

        self.complete_install_setup()
    }

    pub fn remove_selected(&mut self) -> Result<()> {
        // Build remove queue
        let indices: Vec<usize> = if self.tab == Tab::McpServers {
            self.mcp_servers
                .iter()
                .enumerate()
                .filter(|(_, m)| m.selected)
                .map(|(i, _)| i)
                .collect()
        } else if self.tab == Tab::Plugins {
            self.plugins
                .iter()
                .enumerate()
                .filter(|(_, p)| p.selected)
                .map(|(i, _)| i)
                .collect()
        } else if let Some(comp_type) = self.tab.to_component_type() {
            self.components
                .iter()
                .enumerate()
                .filter(|(_, c)| c.selected && c.component_type == comp_type)
                .map(|(i, _)| i)
                .collect()
        } else {
            Vec::new()
        };

        if indices.is_empty() {
            self.status_message = Some("No items selected".to_string());
            return Ok(());
        }

        self.processing_queue = indices;
        self.is_removing = true;

        // Same reasoning as install_selected: send Plugin/MCP through
        // the async Preflighting view; component tabs skip it.
        if needs_cli_preflight(self.tab) {
            self.current_view = View::Preflighting;
            return Ok(());
        }

        self.complete_remove_setup();
        Ok(())
    }

    /// Finish the install setup after the (optional) preflight has succeeded.
    ///
    /// Reads the already-populated `processing_queue` and either prompts
    /// for missing MCP env vars or transitions to the Installing view.
    /// Called from two sites: `install_selected` itself (component tabs,
    /// no preflight) and `handle_preflighting_view` after the background
    /// `--version` probe returns Ok.
    pub(crate) fn complete_install_setup(&mut self) -> Result<()> {
        // For MCP servers, check if any have missing env vars
        if self.tab == Tab::McpServers {
            let indices = self.processing_queue.clone();
            for &idx in &indices {
                if let Some(server) = self.mcp_servers.get(idx) {
                    let missing: Vec<String> = server.def.env.iter()
                        .filter(|e| std::env::var(e).is_err())
                        .cloned()
                        .collect();

                    if !missing.is_empty() {
                        self.start_env_input(idx, missing);
                        return Ok(());
                    }
                }
            }
        }

        self.processing_total = Some(self.processing_queue.len());
        self.processing_progress = Some(0);
        self.processing_log.clear();
        self.processing_log.push(format!("Starting installation of {} items...", self.processing_queue.len()));
        self.is_removing = false;
        self.cancelling = false;
        self.current_view = View::Installing;

        Ok(())
    }

    /// Finish the remove setup after the (optional) preflight has succeeded.
    ///
    /// Mirrors `complete_install_setup` for removal — no env input needed,
    /// so it transitions straight to the Installing view in remove mode.
    pub(crate) fn complete_remove_setup(&mut self) {
        self.processing_total = Some(self.processing_queue.len());
        self.processing_progress = Some(0);
        self.processing_log.clear();
        self.processing_log.push(format!("Starting removal of {} items...", self.processing_queue.len()));
        self.is_removing = true;
        self.cancelling = false;
        self.current_view = View::Installing;
    }

    pub(super) fn start_env_input(&mut self, server_idx: usize, missing_vars: Vec<String>) {
        self.env_input_server_idx = Some(server_idx);
        self.env_input_vars = missing_vars;
        self.env_input_current = 0;
        self.env_input_buffer.clear();
        self.env_input_values.clear();
        self.current_view = View::EnvInput;
    }

    pub fn continue_mcp_install(&mut self) -> Result<()> {
        // Initialize install state
        self.processing_total = Some(self.processing_queue.len());
        self.processing_progress = Some(0);
        self.processing_log.clear();
        self.processing_log.push(format!("Starting installation of {} items...", self.processing_queue.len()));
        self.is_removing = false;
        self.current_view = View::Installing;
        Ok(())
    }

    pub fn start_finish_processing(&mut self) {
        let action = if self.is_removing { "Removal" } else { "Installation" };
        self.processing_log.push(format!("[OK] {} complete!", action));
        self.processing_log.push("".to_string());  // Empty line for spacing
        self.processing_log.push("Refreshing status...".to_string());
        self.needs_refresh = true;
    }

    /// Swap only the component list (and rebuild its tree views), then
    /// mark the refresh complete. Used when the just-finished install /
    /// remove targeted filesystem-backed components — MCP and plugin
    /// data are untouched, so we skip both.
    pub fn apply_components_refresh(&mut self, components: Vec<Component>) {
        self.components = components;
        self.tree_views = build_tree_views(&self.components);
        self.finish_refresh_status();
    }

    /// Swap only the MCP server list. Tree views and other tabs stay as
    /// they were because this scope is reached only from the MCP tab.
    pub fn apply_mcp_refresh(&mut self, mcp_servers: Vec<McpServer>) {
        self.mcp_servers = mcp_servers;
        self.finish_refresh_status();
    }

    /// Swap only the plugin list.
    pub fn apply_plugins_refresh(&mut self, plugins: Vec<Plugin>) {
        self.plugins = plugins;
        self.finish_refresh_status();
    }

    /// Shared "wrap up after refresh" bookkeeping. Each scope-specific
    /// apply method calls this last so the user-visible status line and
    /// internal flags stay consistent across scopes.
    fn finish_refresh_status(&mut self) {
        let verb = if self.is_removing { "Removed" } else { "Installed" };
        self.status_message = Some(format!("{} {} items", verb, self.processing_total.unwrap_or(0)));
        self.processing_log.push("[OK] Status refresh complete!".to_string());
        self.needs_refresh = false;
        self.refreshing = false;
        self.processing_complete = true;
    }

    pub fn close_processing(&mut self) {
        self.current_view = View::List;
        self.processing_queue.clear();
        self.processing_progress = None;
        self.processing_total = None;
        self.processing_log.clear();
        self.is_removing = false;
        self.needs_refresh = false;
        self.refreshing = false;
        self.processing_complete = false;
    }

    pub fn tick(&mut self) {
        // Update animation frame for spinner
        self.animation_frame = (self.animation_frame + 1) % 10;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preflight_runs_for_plugin_and_mcp_tabs() {
        assert!(needs_cli_preflight(Tab::Plugins));
        assert!(needs_cli_preflight(Tab::McpServers));
    }

    #[test]
    fn preflight_skipped_for_component_tabs() {
        // Component tabs are pure filesystem ops; probing the CLI for
        // them would just add 8 seconds of latency on a slow disk.
        assert!(!needs_cli_preflight(Tab::Commands));
        assert!(!needs_cli_preflight(Tab::Agents));
        assert!(!needs_cli_preflight(Tab::Hooks));
        assert!(!needs_cli_preflight(Tab::Skills));
    }
}
