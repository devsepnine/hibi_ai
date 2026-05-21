use super::types::{FocusArea, Tab};
use super::App;
use crate::tree::TreeView;

impl App {
    /// Cycle keyboard focus between the tab bar and the content pane.
    ///
    /// Bound to `Tab`/`Shift+Tab` from the List view. While focus sits on
    /// `Tabs`, arrow/`h`/`l` keys move between tabs; while it sits on
    /// `Content`, those same keys drive list / folder navigation. Keeping a
    /// single toggle (rather than two distinct keybindings) means the user
    /// only needs to remember one shortcut to switch panes.
    pub fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            FocusArea::Content => FocusArea::Tabs,
            FocusArea::Tabs => FocusArea::Content,
        };
    }

    /// Force focus back to the content pane. Used by `Enter`/`Esc`/`↓` from
    /// the tab bar so the user has multiple intuitive ways to "commit" a
    /// tab selection and resume list navigation.
    pub fn focus_content(&mut self) {
        self.focus = FocusArea::Content;
    }

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::App;

    fn fresh_app() -> App {
        // App::new() does filesystem I/O for source resolution which would
        // make the test brittle, so we hand-construct only the fields the
        // focus helpers actually touch.
        App {
            target_cli: None,
            available_tabs: Vec::new(),
            tab: Tab::Skills,
            current_view: crate::app::View::List,
            focus: FocusArea::Content,
            cli_selection_index: 0,
            should_quit: false,
            theme: crate::theme::Theme::default(),
            components: Vec::new(),
            list_index: 0,
            tree_views: std::collections::HashMap::new(),
            mcp_servers: Vec::new(),
            mcp_index: 0,
            mcp_scope: crate::mcp::McpScope::default(),
            mcp_project_path: String::new(),
            plugins: Vec::new(),
            plugin_index: 0,
            diff_content: None,
            diff_scroll: 0,
            source_dir: std::path::PathBuf::new(),
            bundled_git_root: None,
            sources: Vec::new(),
            dest_dir: std::path::PathBuf::new(),
            status_message: None,
            current_output_style: None,
            current_statusline: None,
            processing_progress: None,
            processing_total: None,
            processing_log: Vec::new(),
            processing_queue: Vec::new(),
            is_removing: false,
            animation_frame: 0,
            needs_refresh: false,
            refreshing: false,
            processing_complete: false,
            cancelling: false,
            env_input_server_idx: None,
            env_input_vars: Vec::new(),
            env_input_current: 0,
            env_input_buffer: String::new(),
            env_input_values: Vec::new(),
            project_path_buffer: String::new(),
            source_entries: Vec::new(),
            source_auto_update: false,
            source_list_index: 0,
            source_add_kind: None,
            source_input_buffer: String::new(),
            source_edit_index: None,
            source_sync_status: None,
            source_sync_cancel_tx: None,
            source_input_error: None,
            source_pending_url: String::new(),
            source_pending_branch: None,
            source_pending_root: None,
            source_sync_rx: None,
        }
    }

    #[test]
    fn toggle_focus_cycles_between_panes() {
        let mut app = fresh_app();
        assert_eq!(app.focus, FocusArea::Content);
        app.toggle_focus();
        assert_eq!(app.focus, FocusArea::Tabs);
        app.toggle_focus();
        assert_eq!(app.focus, FocusArea::Content);
    }

    #[test]
    fn focus_content_is_idempotent() {
        let mut app = fresh_app();
        app.focus = FocusArea::Tabs;
        app.focus_content();
        assert_eq!(app.focus, FocusArea::Content);
        // Calling again must not flip back to Tabs.
        app.focus_content();
        assert_eq!(app.focus, FocusArea::Content);
    }
}
