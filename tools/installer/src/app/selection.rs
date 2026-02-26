use super::types::Tab;
use super::App;

impl App {
    pub fn toggle_selected(&mut self) {
        if self.tab == Tab::McpServers {
            if let Some(m) = self.mcp_servers.get_mut(self.mcp_index) {
                m.selected = !m.selected;
            }
        } else if self.tab == Tab::Plugins {
            if let Some(p) = self.plugins.get_mut(self.plugin_index) {
                p.selected = !p.selected;
            }
        } else if self.is_cursor_on_folder() {
            // Toggle all components under folder
            self.toggle_folder_selection();
        } else if let Some(idx) = self.selected_component_index() {
            if let Some(c) = self.components.get_mut(idx) {
                c.selected = !c.selected;
            }
        }
    }

    /// Toggle selection for all components under current folder
    fn toggle_folder_selection(&mut self) {
        if let Some(tree) = self.tree_views.get(&self.tab) {
            if let Some(node_idx) = tree.current_node_idx() {
                let indices = tree.get_folder_component_indices(node_idx);
                if indices.is_empty() {
                    return;
                }

                // Check if all are currently selected
                let all_selected = indices.iter()
                    .all(|&idx| self.components.get(idx).map(|c| c.selected).unwrap_or(false));

                // Toggle: if all selected -> deselect all, otherwise select all
                let new_state = !all_selected;
                for &idx in &indices {
                    if let Some(c) = self.components.get_mut(idx) {
                        c.selected = new_state;
                    }
                }
            }
        }
    }

    pub fn select_all(&mut self) {
        if self.tab == Tab::McpServers {
            for m in &mut self.mcp_servers {
                m.selected = true;
            }
        } else if self.tab == Tab::Plugins {
            for p in &mut self.plugins {
                p.selected = true;
            }
        } else if let Some(comp_type) = self.tab.to_component_type() {
            for c in &mut self.components {
                if c.component_type == comp_type {
                    c.selected = true;
                }
            }
        }
    }

    pub fn deselect_all(&mut self) {
        if self.tab == Tab::McpServers {
            for m in &mut self.mcp_servers {
                m.selected = false;
            }
        } else if self.tab == Tab::Plugins {
            for p in &mut self.plugins {
                p.selected = false;
            }
        } else if let Some(comp_type) = self.tab.to_component_type() {
            for c in &mut self.components {
                if c.component_type == comp_type {
                    c.selected = false;
                }
            }
        }
    }
}
