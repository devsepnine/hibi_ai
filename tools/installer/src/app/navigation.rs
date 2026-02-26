use super::types::Tab;
use super::App;
use crate::tree::TreeView;

impl App {
    pub fn next_tab(&mut self) {
        if let Some(current_idx) = self.available_tabs.iter().position(|t| *t == self.tab) {
            let next_idx = (current_idx + 1) % self.available_tabs.len();
            self.tab = self.available_tabs[next_idx];
            self.list_index = 0;
        }
    }

    pub fn prev_tab(&mut self) {
        if let Some(current_idx) = self.available_tabs.iter().position(|t| *t == self.tab) {
            let prev_idx = if current_idx == 0 {
                self.available_tabs.len() - 1
            } else {
                current_idx - 1
            };
            self.tab = self.available_tabs[prev_idx];
            self.list_index = 0;
        }
    }

    pub fn set_tab(&mut self, idx: usize) {
        if let Some(&tab) = self.available_tabs.get(idx) {
            self.tab = tab;
            self.list_index = 0;
        }
    }

    pub fn next_item(&mut self) {
        if self.tab == Tab::McpServers {
            let len = self.mcp_servers.len();
            if len > 0 {
                self.mcp_index = (self.mcp_index + 1) % len;
            }
        } else if self.tab == Tab::Plugins {
            let len = self.plugins.len();
            if len > 0 {
                self.plugin_index = (self.plugin_index + 1) % len;
            }
        } else if let Some(tree) = self.tree_views.get_mut(&self.tab) {
            tree.next();
        }
    }

    pub fn prev_item(&mut self) {
        if self.tab == Tab::McpServers {
            let len = self.mcp_servers.len();
            if len > 0 {
                self.mcp_index = if self.mcp_index == 0 { len - 1 } else { self.mcp_index - 1 };
            }
        } else if self.tab == Tab::Plugins {
            let len = self.plugins.len();
            if len > 0 {
                self.plugin_index = if self.plugin_index == 0 { len - 1 } else { self.plugin_index - 1 };
            }
        } else if let Some(tree) = self.tree_views.get_mut(&self.tab) {
            tree.prev();
        }
    }

    /// Get the actual component index in self.components for current selection
    pub fn selected_component_index(&self) -> Option<usize> {
        if let Some(tree) = self.tree_views.get(&self.tab) {
            tree.current_component_idx()
        } else {
            // Fallback for non-tree tabs
            let filtered = self.current_components();
            filtered.get(self.list_index).map(|(idx, _)| *idx)
        }
    }

    /// Check if cursor is on a folder
    pub fn is_cursor_on_folder(&self) -> bool {
        self.tree_views.get(&self.tab)
            .map(|t| t.is_on_folder())
            .unwrap_or(false)
    }

    /// Check if current folder is expanded
    pub fn is_current_folder_expanded(&self) -> bool {
        self.tree_views.get(&self.tab)
            .map(|t| t.is_current_folder_expanded())
            .unwrap_or(false)
    }

    /// Toggle expand/collapse for current folder
    pub fn toggle_folder_expand(&mut self) {
        if let Some(tree) = self.tree_views.get_mut(&self.tab) {
            tree.toggle_expand();
        }
    }

    /// Expand current folder
    pub fn expand_folder(&mut self) {
        if let Some(tree) = self.tree_views.get_mut(&self.tab) {
            tree.expand();
        }
    }

    /// Collapse current folder
    pub fn collapse_folder(&mut self) {
        if let Some(tree) = self.tree_views.get_mut(&self.tab) {
            tree.collapse();
        }
    }

    /// Collapse parent folder (when cursor is on a file or subfolder)
    pub fn collapse_parent_folder(&mut self) {
        if let Some(tree) = self.tree_views.get_mut(&self.tab) {
            tree.collapse_parent();
        }
    }

    /// Get current tree view
    pub fn get_tree_view(&self) -> Option<&TreeView> {
        self.tree_views.get(&self.tab)
    }
}
