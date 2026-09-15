use super::types::{FocusArea, Tab, View};
use super::App;
use crate::tree::TreeView;

impl App {
    /// Cycle keyboard focus between the tab bar and the content pane.
    ///
    /// Bound to `Tab`/`Shift+Tab` from the List view, kept alongside the
    /// `1`/`2` pane jumps for muscle memory. While focus sits on `Tabs`,
    /// arrow/`h`/`l` keys move between tabs; while it sits on `Content`,
    /// those same keys drive list / folder navigation.
    pub fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            FocusArea::Content => FocusArea::Tabs,
            FocusArea::Tabs => FocusArea::Content,
        };
    }

    /// Move focus to the tab bar. Bound to `1` — the lazygit convention of
    /// addressing panes by number instead of cycling through them.
    pub fn focus_tabs(&mut self) {
        self.focus = FocusArea::Tabs;
    }

    /// Force focus back to the content pane. Bound to `2`, and to
    /// `Enter`/`Esc`/`↓` from the tab bar so the user has multiple intuitive
    /// ways to "commit" a tab selection and resume list navigation.
    pub fn focus_content(&mut self) {
        self.focus = FocusArea::Content;
    }

    /// Open the `?` keybinding reference.
    ///
    /// The scroll resets so the table always opens at the first section — a
    /// remembered offset from a previous visit would hide the top rows with no
    /// visible reason.
    pub fn open_help(&mut self) {
        self.help_scroll = 0;
        self.current_view = View::Help;
    }

    /// Dismiss the reference and hand the keyboard back to the list.
    ///
    /// Help is only reachable from `View::List`, so returning there is exact
    /// rather than a guess at where the user came from.
    pub fn close_help(&mut self) {
        self.current_view = View::List;
    }

    /// Scroll the binding table down, stopping once the last row is on screen.
    ///
    /// `max` comes from the caller because it depends on the terminal height,
    /// which `App` deliberately knows nothing about. Clamping matters here in a
    /// way it does not for the diff: the table is short, so an unclamped scroll
    /// would leave the user staring at an empty box with no hint that the way
    /// back is `k`.
    pub fn scroll_help_down(&mut self, max: u16) {
        self.help_scroll = self.help_scroll.saturating_add(1).min(max);
    }

    pub fn scroll_help_up(&mut self) {
        self.help_scroll = self.help_scroll.saturating_sub(1);
    }

    /// Answer `Esc` from the content pane: leave for the CLI picker, asking
    /// first if there is anything to lose.
    ///
    /// The prompt is conditional rather than always-on because the cost of
    /// leaving is: with nothing ticked the trip is free and a confirmation
    /// would be noise, while with ticks it is one keystroke away from
    /// discarding work the picker's re-scan cannot restore.
    pub fn request_exit_to_main(&mut self) {
        if self.has_selection() {
            self.current_view = View::ConfirmExit;
        } else {
            self.exit_to_main();
        }
    }

    /// Discard the selections and hand the keyboard back to the CLI picker.
    ///
    /// The stale status message is left alone: the picker and the loading screen
    /// both return before the status bar is drawn, and `finish_loading` always
    /// overwrites it on the way back, so clearing it here would be a write no
    /// render can observe.
    pub fn exit_to_main(&mut self) {
        self.clear_all_selections();
        self.current_view = View::CliSelection;
    }

    /// Dismiss the exit prompt, leaving every selection as it was.
    pub fn cancel_exit(&mut self) {
        self.current_view = View::List;
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
    use crate::app::test_support::fresh_app;

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

    /// Reopening must not resume where the last visit left off, and closing
    /// must land back on the list rather than on whatever view came before.
    #[test]
    fn help_opens_at_the_top_and_closes_back_to_the_list() {
        let mut app = fresh_app();
        app.help_scroll = 7;

        app.open_help();
        assert_eq!(app.current_view, View::Help);
        assert_eq!(app.help_scroll, 0);

        app.close_help();
        assert_eq!(app.current_view, View::List);
    }

    /// The clamp is the point: scrolling past `max` would show a blank box, and
    /// every press held past the end would then need an answering `k` before the
    /// table moved again.
    #[test]
    fn help_scroll_stops_at_the_last_row() {
        let mut app = fresh_app();

        for _ in 0..5 {
            app.scroll_help_down(2);
        }
        assert_eq!(app.help_scroll, 2);

        app.scroll_help_up();
        assert_eq!(app.help_scroll, 1);
    }

    /// A terminal tall enough for the whole table must not scroll at all.
    #[test]
    fn help_does_not_scroll_when_everything_fits() {
        let mut app = fresh_app();
        app.scroll_help_down(0);
        assert_eq!(app.help_scroll, 0);
    }

    /// Coming back from the picker re-scans, and a shorter result would leave
    /// the cursors past the end of their lists. The three are asserted together
    /// because each indexes a different list and only one of them is reset by
    /// anything else — a fix that reached `mcp_index` alone would leave `Space`
    /// dead in the Plugins tab.
    #[test]
    fn re_entering_from_the_picker_resets_every_list_cursor() {
        let mut app = fresh_app();
        app.list_index = 3;
        app.mcp_index = 7;
        app.plugin_index = 5;

        app.exit_to_main();
        assert_eq!(app.current_view, View::CliSelection);

        app.select_cli(crate::app::TargetCli::Claude)
            .expect("select_cli needs only a home directory");
        assert_eq!((app.list_index, app.mcp_index, app.plugin_index), (0, 0, 0));
    }

    #[test]
    fn focus_tabs_is_idempotent() {
        let mut app = fresh_app();
        app.focus_tabs();
        assert_eq!(app.focus, FocusArea::Tabs);
        // Repeated `1` presses must not bounce focus back to Content.
        app.focus_tabs();
        assert_eq!(app.focus, FocusArea::Tabs);
    }
}
