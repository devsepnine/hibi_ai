use anyhow::Result;

use super::types::{Tab, View};
use super::{App, build_tree_views};
use crate::component::Component;
use crate::mcp::McpServer;
use crate::plugin::Plugin;

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

        // For MCP servers, check if any have missing env vars
        if self.tab == Tab::McpServers {
            for &idx in &indices {
                if let Some(server) = self.mcp_servers.get(idx) {
                    let missing: Vec<String> = server.def.env.iter()
                        .filter(|e| std::env::var(e).is_err())
                        .cloned()
                        .collect();

                    if !missing.is_empty() {
                        // Start env input for this server
                        self.processing_queue = indices;
                        self.start_env_input(idx, missing);
                        return Ok(());
                    }
                }
            }
        }

        // Initialize install state (no env input needed)
        self.processing_queue = indices;
        self.processing_total = Some(self.processing_queue.len());
        self.processing_progress = Some(0);
        self.processing_log.clear();
        self.processing_log.push(format!("Starting installation of {} items...", self.processing_queue.len()));
        self.is_removing = false;
        self.cancelling = false;
        self.current_view = View::Installing;

        Ok(())
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

        // Initialize remove state
        self.processing_queue = indices;
        self.processing_total = Some(self.processing_queue.len());
        self.processing_progress = Some(0);
        self.processing_log.clear();
        self.processing_log.push(format!("Starting removal of {} items...", self.processing_queue.len()));
        self.is_removing = true;
        self.cancelling = false;
        self.current_view = View::Installing;

        Ok(())
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

    pub fn apply_refresh_result(&mut self, components: Vec<Component>, mcp_servers: Vec<McpServer>, plugins: Vec<Plugin>) {
        self.components = components;
        self.mcp_servers = mcp_servers;
        self.plugins = plugins;

        // Rebuild tree views with new components
        self.tree_views = build_tree_views(&self.components);

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
