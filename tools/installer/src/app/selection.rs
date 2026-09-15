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

    /// Whether anything is selected anywhere, not just in the visible tab.
    ///
    /// Every other helper here is scoped to `self.tab`, which is right for keys
    /// the user aims at one list. The exit prompt is not one of those: ticks
    /// survive tab switches, so a tab-scoped check would wave the user off the
    /// screen while their choices in another tab were still pending.
    pub fn has_selection(&self) -> bool {
        self.components.iter().any(|c| c.selected)
            || self.mcp_servers.iter().any(|m| m.selected)
            || self.plugins.iter().any(|p| p.selected)
    }

    /// Drop every selection in every tab.
    ///
    /// Leaving the List view for the CLI picker already discards the ticks in
    /// practice, because coming back re-scans and replaces all three lists.
    /// Clearing them here makes that the exit's own doing rather than a
    /// side effect of the loader it happens to be followed by.
    pub fn clear_all_selections(&mut self) {
        for c in &mut self.components {
            c.selected = false;
        }
        for m in &mut self.mcp_servers {
            m.selected = false;
        }
        for p in &mut self.plugins {
            p.selected = false;
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
