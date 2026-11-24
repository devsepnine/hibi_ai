use anyhow::Result;

use super::App;
use super::types::{Tab, View};
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
    pub(crate) fn is_claude_config(&self) -> bool {
        self.tab == Tab::Config && self.target_cli == Some(crate::target::TargetCli::Claude)
    }

    fn has_selected_config_items(&self) -> bool {
        self.is_claude_config() && self.visible_components().iter().any(|(_, c)| c.selected)
    }

    pub fn install_selected(&mut self) -> Result<()> {
        // Build install queue
        let indices: Vec<usize> = if self.tab == Tab::McpServers {
            self.visible_mcp_servers()
                .into_iter()
                .filter(|(_, m)| {
                    m.selected
                        && !(self.target_cli == Some(crate::target::TargetCli::Codex)
                            && m.status == crate::mcp::McpStatus::Installed)
                })
                .map(|(i, _)| i)
                .collect()
        } else if self.tab == Tab::Plugins {
            self.visible_plugins()
                .into_iter()
                .filter(|(_, p)| p.selected)
                .map(|(i, _)| i)
                .collect()
        } else {
            self.visible_components()
                .into_iter()
                .filter(|(_, c)| c.selected && c.is_install_eligible())
                .map(|(i, _)| i)
                .collect()
        };

        if indices.is_empty() {
            self.status_message = Some(
                if self.has_selected_config_items() {
                    "Selected configuration items cannot be installed. Check the diff for details."
                } else if self.tab == Tab::McpServers
                    && self
                        .visible_mcp_servers()
                        .iter()
                        .any(|(_, server)| server.selected)
                {
                    "Selected MCP servers are already registered"
                } else {
                    "No items selected"
                }
                .into(),
            );
            return Ok(());
        }

        self.clear_env_inputs();
        self.processing_complete = false;
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

    pub(crate) fn removal_selection(&self) -> Vec<usize> {
        if self.tab == Tab::McpServers {
            self.visible_mcp_servers()
                .into_iter()
                .filter(|(_, m)| m.selected)
                .map(|(i, _)| i)
                .collect()
        } else if self.tab == Tab::Plugins {
            self.visible_plugins()
                .into_iter()
                .filter(|(_, p)| p.selected)
                .map(|(i, _)| i)
                .collect()
        } else {
            self.visible_components()
                .into_iter()
                .filter(|(_, c)| c.selected && c.is_remove_eligible())
                .map(|(i, _)| i)
                .collect()
        }
    }

    pub(crate) fn removal_confirmation_count(&mut self) -> usize {
        let count = self.removal_selection().len();
        if count == 0 {
            self.status_message = Some(
                if self.has_selected_config_items() {
                    "No selected configuration items can be removed. Check the diff for details."
                } else {
                    "Select items in this category to remove."
                }
                .into(),
            );
        }
        count
    }

    pub(crate) fn remove_confirmed(&mut self, count: usize) -> Result<()> {
        let indices = self.removal_selection();
        anyhow::ensure!(
            self.current_view == View::List && count > 0 && indices.len() == count,
            "Selection changed. Review your selection before removing items."
        );
        self.start_removal(indices);
        Ok(())
    }

    fn start_removal(&mut self, indices: Vec<usize>) {
        self.clear_env_inputs();
        self.processing_complete = false;
        self.processing_queue = indices;
        self.is_removing = true;

        // Same reasoning as install_selected: send Plugin/MCP through
        // the async Preflighting view; component tabs skip it.
        if needs_cli_preflight(self.tab) {
            self.current_view = View::Preflighting;
            return;
        }

        self.complete_remove_setup();
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
                    let missing: Vec<String> = server
                        .def
                        .inputs()
                        .filter(|&name| {
                            let entered = self
                                .mcp_env_values
                                .get(&idx)
                                .is_some_and(|values| values.iter().any(|(key, _)| key == name));
                            let inherited = std::env::var(name).is_ok_and(|value| {
                                !value.is_empty()
                                    && server
                                        .def
                                        .setup
                                        .as_ref()
                                        .filter(|s| s.input() == Some(name))
                                        .is_none_or(|s| s.accepts(&value))
                            });
                            !entered && !inherited
                        })
                        .map(str::to_owned)
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
        self.processing_log.push(format!(
            "Starting installation of {} items...",
            self.processing_queue.len()
        ));
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
        self.processing_log.push(format!(
            "Starting removal of {} items...",
            self.processing_queue.len()
        ));
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
        let idx = self
            .env_input_server_idx
            .filter(|&idx| self.mcp_servers.get(idx).is_some())
            .ok_or_else(|| anyhow::anyhow!("No MCP server is awaiting credentials"))?;
        self.mcp_env_values
            .insert(idx, std::mem::take(&mut self.env_input_values));
        self.env_input_server_idx = None;
        self.env_input_vars.clear();
        self.env_input_current = 0;
        self.env_input_buffer.clear();
        self.complete_install_setup()
    }

    pub(crate) fn clear_env_inputs(&mut self) {
        self.env_input_server_idx = None;
        self.env_input_vars.clear();
        self.env_input_current = 0;
        self.env_input_buffer.clear();
        self.env_input_values.clear();
        self.mcp_env_values.clear();
    }

    pub fn start_finish_processing(&mut self) {
        self.clear_env_inputs();
        self.processing_log.push("Refreshing status...".to_string());
        self.needs_refresh = true;
    }

    /// Swap only the component list (and rebuild its tree views), then
    /// mark the refresh complete. Used when the just-finished install /
    /// remove targeted filesystem-backed components — MCP and plugin
    /// data are untouched, so we skip both.
    pub fn apply_components_refresh(&mut self, components: Vec<Component>) {
        self.components = components;
        self.refresh_settings_defaults();
        self.rebuild_tree_views();
        self.reset_visible_list_cursors();
        self.finish_refresh_status(Ok(()));
    }

    /// Swap only the MCP server list. Tree views and other tabs stay as
    /// they were because this scope is reached only from the MCP tab.
    pub fn apply_mcp_refresh(&mut self, mcp_servers: Vec<McpServer>) {
        self.mcp_servers = mcp_servers;
        self.reset_visible_list_cursors();
        self.finish_refresh_status(Ok(()));
    }

    /// Swap only the plugin list.
    pub fn apply_plugins_refresh(&mut self, plugins: Vec<Plugin>) {
        self.plugins = plugins;
        self.reset_visible_list_cursors();
        self.finish_refresh_status(Ok(()));
    }

    /// Include the refresh outcome in the final error count.
    pub(crate) fn finish_refresh_status(&mut self, result: Result<()>) {
        let failed = result.is_err();
        self.processing_log.push(match result {
            Ok(()) => "[OK] Status refresh complete!".into(),
            Err(error) => format!("[ERROR] {error:#}"),
        });
        let summary = self.processing_summary();
        self.processing_log.push(format!("[INFO] {summary}"));
        self.status_message = Some(if failed {
            "Operation finished; status refresh failed. Review the log.".into()
        } else {
            summary
        });
        self.needs_refresh = false;
        self.refreshing = false;
        self.processing_complete = true;
    }

    pub(crate) fn can_cancel_processing(&self) -> bool {
        matches!(self.current_view, View::Preflighting | View::Installing)
            && !self.processing_complete
            && !self.cancelling
            && !self.needs_refresh
            && !self.refreshing
    }

    pub(crate) fn processing_errors(&self) -> usize {
        self.processing_log
            .iter()
            .filter(|line| line.starts_with("[ERR]") || line.starts_with("[ERROR]"))
            .count()
    }

    pub(crate) fn processing_cancelled(&self) -> bool {
        self.processing_log
            .iter()
            .any(|line| line == "[WARN] Cancelled by user")
    }

    pub(crate) fn processing_summary(&self) -> String {
        let action = if self.is_removing {
            "Removal"
        } else {
            "Installation"
        };
        format!(
            "{} {}: {} of {} items attempted; errors: {}",
            action,
            if self.processing_cancelled() {
                "cancelled"
            } else {
                "finished"
            },
            self.processing_progress.unwrap_or(0),
            self.processing_total.unwrap_or(0),
            self.processing_errors()
        )
    }

    pub fn close_processing(&mut self) {
        self.clear_env_inputs();
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
    fn component_refresh_reloads_default_labels_from_settings() -> Result<()> {
        let fixture = tempfile::tempdir()?;
        let mut app = super::super::test_support::fresh_app();
        app.dest_dir = fixture.path().to_owned();
        let path = app.dest_dir.join("settings.json");
        std::fs::write(
            &path,
            br#"{"outputStyle":"my-style","statusLine":{"command":"./statusline_linux"}}"#,
        )?;
        app.apply_components_refresh(Vec::new());
        assert_eq!(app.current_output_style.as_deref(), Some("my-style"));
        assert_eq!(app.current_statusline.as_deref(), Some("statusline_linux"));

        std::fs::write(path, b"{}")?;
        app.apply_components_refresh(Vec::new());
        assert!(app.current_output_style.is_none());
        assert!(app.current_statusline.is_none());
        Ok(())
    }

    #[test]
    fn config_removal_queues_only_removable_files() {
        use crate::component::{ComponentType, InstallStatus};

        let mut app = super::super::test_support::fresh_app();
        app.tab = Tab::Config;
        app.components = ["settings.json", "CLAUDE.md"]
            .map(|name| {
                Component::new(
                    ComponentType::ConfigFile,
                    name.into(),
                    name.into(),
                    name.into(),
                    InstallStatus::Managed,
                )
            })
            .into();
        app.remove_confirmed(1).unwrap();
        assert_eq!(app.processing_queue, vec![1]);
        assert_eq!(app.current_view, View::Installing);
    }

    #[test]
    fn codex_batch_skips_installed_entries_before_prompting() {
        let mut app = queued_mcp_batch();
        app.target_cli = Some(crate::target::TargetCli::Codex);
        app.mcp_servers[0].status = crate::mcp::McpStatus::Installed;
        app.install_selected().unwrap();
        assert_eq!(app.processing_queue, vec![1]);
        app.complete_install_setup().unwrap();
        assert_eq!(app.env_input_server_idx, Some(1));
    }

    fn queued_mcp_batch() -> App {
        let mut app = super::super::test_support::fresh_app();
        app.tab = Tab::McpServers;
        for index in 0..2 {
            let mut server = super::super::test_support::make_mcp_server();
            server.def.name = format!("server{index}");
            server.def.env = vec![format!("HIBI_TEST_MISSING_BATCH_SECRET_{index}_91EDB7")];
            server.selected = true;
            app.mcp_servers.push(server);
        }
        app.processing_queue = vec![0, 1];
        app
    }

    #[test]
    fn mcp_batch_collects_separate_credentials_before_any_install() {
        let mut app = queued_mcp_batch();
        app.complete_install_setup().unwrap();
        assert_eq!(app.env_input_server_idx, Some(0));
        app.env_input_buffer = "first secret".into();
        app.env_input_submit().unwrap();
        assert_eq!(app.current_view, View::EnvInput);
        assert_eq!(app.env_input_server_idx, Some(1));
        assert_eq!(app.mcp_env_values[&0][0].1, "first secret");
        assert!(app.env_input_buffer.is_empty());
        app.env_input_buffer = "두번째 비밀".into();
        app.env_input_submit().unwrap();
        assert_eq!(app.current_view, View::Installing);
        assert_eq!(app.processing_queue, vec![0, 1]);
        assert_eq!(app.mcp_env_values[&1][0].1, "두번째 비밀");
        assert_eq!(app.processing_total, Some(2));
        assert!(
            app.processing_log
                .iter()
                .all(|line| !line.contains("secret") && !line.contains("비밀"))
        );
        assert!(app.env_input_values.is_empty());
        assert!(app.env_input_server_idx.is_none());
    }

    #[test]
    fn cancelling_second_mcp_prompt_discards_all_collected_credentials() {
        let mut app = queued_mcp_batch();
        app.complete_install_setup().unwrap();
        app.env_input_buffer = "first secret".into();
        app.env_input_submit().unwrap();
        app.env_input_buffer = "partial second secret".into();
        app.env_input_cancel();
        assert_eq!(app.current_view, View::List);
        assert!(app.mcp_env_values.is_empty());
        assert!(app.env_input_values.is_empty());
        assert!(app.env_input_buffer.is_empty());
        assert!(app.processing_queue.is_empty());
    }

    #[test]
    fn new_operation_and_completion_do_not_reuse_mcp_credentials() {
        let mut app = queued_mcp_batch();
        app.mcp_env_values
            .insert(0, vec![("TOKEN".into(), "stale".into())]);
        app.env_input_values
            .push(("TOKEN".into(), "partial".into()));
        app.processing_complete = true;
        app.install_selected().unwrap();
        assert_eq!(app.current_view, View::Preflighting);
        assert!(app.mcp_env_values.is_empty());
        assert!(app.env_input_values.is_empty());
        assert!(!app.processing_complete);
        app.mcp_env_values
            .insert(0, vec![("TOKEN".into(), "secret".into())]);
        app.start_finish_processing();
        assert!(app.mcp_env_values.is_empty());
        app.close_processing();
        assert!(app.env_input_values.is_empty());
    }

    #[test]
    fn failed_and_cancelled_batches_never_report_all_items_installed() {
        let mut app = super::super::test_support::fresh_app();
        app.processing_total = Some(5);
        app.processing_progress = Some(2);
        app.processing_log = vec![
            "[ERR] Permission denied".into(),
            "[WARN] Cancelled by user".into(),
        ];
        app.start_finish_processing();
        app.apply_mcp_refresh(Vec::new());
        assert_eq!(
            app.status_message.as_deref(),
            Some("Installation cancelled: 2 of 5 items attempted; errors: 1")
        );
        assert!(
            !app.processing_log
                .iter()
                .any(|line| line.contains("Installation complete"))
        );
        assert!(app.processing_complete);
    }

    #[test]
    fn successful_removal_summary_uses_attempted_count() {
        let mut app = super::super::test_support::fresh_app();
        app.is_removing = true;
        app.processing_total = Some(2);
        app.processing_progress = Some(2);
        app.apply_mcp_refresh(Vec::new());
        assert_eq!(
            app.status_message.as_deref(),
            Some("Removal finished: 2 of 2 items attempted; errors: 0")
        );
    }

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

#[cfg(test)]
mod search_tests {
    use super::*;
    use crate::app::test_support::{fresh_app, make_mcp_server, make_plugin};

    fn app_with_selected_lists() -> App {
        let mut app = fresh_app();
        app.mcp_servers = ["hidden", "target"]
            .map(|name| {
                let mut server = make_mcp_server();
                server.def.name = name.into();
                server.selected = true;
                server
            })
            .into();
        app.plugins = ["hidden", "target"]
            .map(|name| {
                let mut plugin = make_plugin();
                plugin.def.name = name.into();
                plugin.selected = true;
                plugin
            })
            .into();
        app.set_search_query("target".into());
        app
    }

    #[test]
    fn install_and_removal_queues_exclude_selected_search_nonmatches() {
        for tab in [Tab::McpServers, Tab::Plugins] {
            for removing in [false, true] {
                let mut app = app_with_selected_lists();
                app.tab = tab;
                if removing {
                    app.remove_confirmed(1).unwrap();
                } else {
                    app.install_selected().unwrap();
                }
                assert_eq!(app.processing_queue, vec![1]);
                assert_eq!(app.current_view, View::Preflighting);
                assert!(app.mcp_servers[0].selected);
                assert!(app.plugins[0].selected);
            }
        }
    }

    #[test]
    fn hidden_selected_mcp_never_prompts_for_credentials() {
        let mut app = app_with_selected_lists();
        app.tab = Tab::McpServers;
        app.mcp_servers[0].def.env = vec!["HIBI_SEARCH_HIDDEN_CREDENTIAL_729A8762".into()];
        app.install_selected().unwrap();
        app.complete_install_setup().unwrap();
        assert_eq!(app.processing_queue, vec![1]);
        assert_eq!(app.current_view, View::Installing);
        assert!(app.env_input_server_idx.is_none());
        assert!(app.env_input_vars.is_empty());
    }

    #[test]
    fn no_matches_cannot_queue_hidden_selected_items() {
        for tab in [Tab::McpServers, Tab::Plugins] {
            let mut app = app_with_selected_lists();
            app.tab = tab;
            app.set_search_query("no-match".into());
            app.install_selected().unwrap();
            assert_eq!(app.removal_confirmation_count(), 0);
            assert!(app.processing_queue.is_empty());
            assert_eq!(app.current_view, View::List);
        }
    }

    #[test]
    fn refresh_restarts_at_first_filtered_global_index() {
        let mut app = app_with_selected_lists();
        app.mcp_index = usize::MAX;
        app.plugin_index = usize::MAX;
        app.apply_mcp_refresh(app.mcp_servers.clone());
        assert_eq!(app.mcp_index, 1);
        assert_eq!(app.plugin_index, 1);
        let mut plugins = app.plugins.clone();
        plugins[1].def.name = "no longer a match".into();
        app.apply_plugins_refresh(plugins);
        assert_eq!(app.plugin_index, 0);
        assert_eq!(app.search_query, "target");
    }
}
