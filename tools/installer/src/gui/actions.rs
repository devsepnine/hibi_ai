use crossterm::event::KeyCode;

use crate::app::{ComponentFilter, HiddenVisibility, Tab, TargetCli, View};
use crate::{cli, loading};

use super::runtime::Runtime;

#[derive(Debug, Clone)]
pub enum Action {
    Key(KeyCode),
    Command(KeyCode),
    SelectTarget(TargetCli),
    SelectTab(Tab),
    SelectRow(usize),
    ToggleRow(usize),
    ExpandRow(usize),
    HideSelectedFolders,
    UnhideSelectedFolders,
    SetFilter(ComponentFilter, bool),
    ToggleFilterMenu,
    CloseFilterMenu,
    SetHiddenVisibility(HiddenVisibility),
    SetSearch(String),
    ClearSearch,
    SetInput(String),
    SetProjectPath(String),
    SetEnvInput(String),
    ToggleTheme,
    OpenSources,
    SelectSource(usize),
    Back,
    Cancel,
    Retry,
    DismissError,
    RequestClose,
    ConfirmClose,
    DismissClose,
    ConfirmRemove,
    CancelRemove,
}

impl Runtime {
    pub fn dispatch(&mut self, action: Action) {
        if self.filter_menu_open && matches!(action, Action::Key(KeyCode::Esc)) {
            self.filter_menu_open = false;
            return;
        }
        if !matches!(
            action,
            Action::ToggleFilterMenu | Action::SetFilter(..) | Action::SetHiddenVisibility(_)
        ) {
            self.filter_menu_open = false;
        }
        match action {
            Action::CloseFilterMenu => {
                self.filter_menu_open = false;
                return;
            }
            Action::RequestClose => {
                self.remove_requested = false;
                self.remove_count = 0;
                let selected = self.app.as_ref().is_some_and(|app| app.has_selection());
                if self.operation_active() || selected {
                    self.close_requested = true;
                } else {
                    self.closing = true;
                    self.should_close = !self.busy();
                }
                return;
            }
            Action::DismissClose => {
                if !self.closing {
                    self.close_requested = false;
                }
                return;
            }
            Action::ConfirmClose => {
                if !self.close_requested {
                    return;
                }
                self.close_requested = false;
                self.closing = true;
                self.cancel();
                if !self.busy() {
                    self.should_close = true;
                }
                return;
            }
            Action::Key(KeyCode::Esc) if self.close_requested && !self.closing => {
                self.close_requested = false;
                return;
            }
            _ if self.close_requested || self.closing => return,
            Action::ConfirmRemove => {
                self.confirm_removal();
                return;
            }
            Action::CancelRemove | Action::Cancel | Action::Back if self.remove_requested => {
                self.remove_requested = false;
                self.remove_count = 0;
                return;
            }
            Action::Key(KeyCode::Esc | KeyCode::Char('n')) if self.remove_requested => {
                self.remove_requested = false;
                self.remove_count = 0;
                return;
            }
            _ if self.remove_requested => return,
            Action::Key(KeyCode::Char('r'))
                if self.app.as_ref().is_some_and(|app| {
                    app.current_view == View::List && app.focus == crate::app::FocusArea::Content
                }) =>
            {
                self.request_removal();
                return;
            }
            Action::Key(KeyCode::Char('o'))
                if self.app.as_ref().is_some_and(|app| {
                    app.current_view == View::List
                        && app.tab == Tab::McpServers
                        && app.target_cli == Some(TargetCli::Codex)
                }) =>
            {
                if let Some(app) = self.app.as_mut() {
                    app.mcp_scope = crate::mcp::McpScope::User;
                    app.status_message = Some(
                        "Codex MCP uses user scope. Local scope is available for Claude Code."
                            .into(),
                    );
                }
                return;
            }
            Action::Command(key) => {
                if let Some(app) = self.app.as_mut()
                    && app.current_view == View::List
                {
                    app.focus_content();
                }
                self.dispatch(Action::Key(key));
                return;
            }
            Action::DismissError => {
                self.error = None;
                return;
            }
            Action::Retry if self.app.is_none() && self.init_rx.is_none() => {
                self.start_initialization();
                return;
            }
            Action::Cancel => {
                self.cancel();
                return;
            }
            _ => {}
        }
        let previous_theme = self.app.as_ref().map(|app| app.theme.mode());
        let result = self.apply(action);
        if let Some(app) = self.app.as_mut()
            && previous_theme != Some(app.theme.mode())
        {
            self.appearance.request(app.theme.mode());
        }
        if let Err(error) = result {
            let message = format!("{error:#}");
            if let Some(app) = self.app.as_mut() {
                app.status_message = Some(message.clone());
            }
            self.error = Some(message);
        }
    }

    fn apply(&mut self, action: Action) -> anyhow::Result<()> {
        let Some(app) = self.app.as_mut() else {
            return Ok(());
        };
        match action {
            Action::ToggleTheme => app.theme.toggle(),
            Action::SelectTarget(target) if app.current_view == View::CliSelection => {
                app.select_cli(target)?;
                loading::start_loading_thread(app, &self.channels.refresh_tx);
            }
            Action::SelectTab(tab)
                if app.current_view == View::List && app.available_tabs.contains(&tab) =>
            {
                app.tab = tab;
                app.reset_visible_list_cursors();
                app.focus_tabs();
            }
            Action::SelectRow(index) => {
                select_row(app, index);
            }
            Action::SelectSource(index) if app.current_view == View::Sources => {
                if index <= app.source_entries.len() {
                    app.source_list_index = index;
                }
            }
            Action::ToggleRow(index) if app.current_view == View::List => {
                if select_row(app, index) {
                    app.toggle_selected();
                }
            }
            Action::ExpandRow(index) if app.current_view == View::List => {
                if select_row(app, index) && app.is_cursor_on_folder() {
                    app.toggle_folder_expand();
                }
            }
            Action::HideSelectedFolders if app.current_view == View::List => {
                app.focus_content();
                app.hide_selected_folders();
            }
            Action::UnhideSelectedFolders if app.current_view == View::List => {
                app.focus_content();
                app.unhide_selected_folders();
            }
            Action::SetSearch(query) if app.current_view == View::List => {
                app.set_search_query(query);
                app.focus_content();
            }
            Action::ClearSearch if app.current_view == View::List => {
                app.set_search_query(String::new());
                app.focus_content();
            }
            Action::SetFilter(filter, checked)
                if app.current_view == View::List && app.tab.to_component_type().is_some() =>
            {
                app.focus_content();
                app.set_component_filter(filter, checked);
            }
            Action::ToggleFilterMenu
                if app.current_view == View::List && app.tab.to_component_type().is_some() =>
            {
                self.filter_menu_open = !self.filter_menu_open;
            }
            Action::SetHiddenVisibility(visibility) if app.current_view == View::List => {
                app.focus_content();
                app.set_hidden_visibility(visibility);
            }
            Action::SetInput(value) if source_input_view(app.current_view) => {
                app.source_input_buffer = value;
                app.source_input_error = None;
            }
            Action::SetProjectPath(value) if app.current_view == View::ProjectPath => {
                app.project_path_buffer = value;
            }
            Action::SetEnvInput(value) if app.current_view == View::EnvInput => {
                app.env_input_buffer = value;
            }
            Action::OpenSources if app.current_view == View::CliSelection => {
                app.current_view = View::Sources;
            }
            Action::Back if app.current_view == View::Installing && app.processing_complete => {
                loading::installing_key(app, KeyCode::Esc, &mut self.channels);
            }
            Action::Back if app.current_view == View::List => app.request_exit_to_main(),
            Action::Back if !self_busy_view(app) => {
                cli::dispatch_key(app, KeyCode::Esc, &self.channels.refresh_tx)?;
            }
            Action::Key(KeyCode::Esc)
                if app.current_view == View::List && !app.search_query.is_empty() =>
            {
                app.set_search_query(String::new());
            }
            Action::Key(key) => {
                if app.current_view == View::SourceConfirmRemove
                    && key == KeyCode::Char('y')
                    && app.source_list_index == 0
                {
                    app.current_view = View::Sources;
                    anyhow::bail!("The bundled source cannot be removed");
                }
                if app.current_view == View::Installing {
                    loading::installing_key(app, key, &mut self.channels);
                } else if app.current_view == View::Preflighting {
                    if key == KeyCode::Esc {
                        loading::preflighting_key(app, key, &mut self.channels);
                    }
                } else if !self_busy_view(app) {
                    if key == KeyCode::Enter
                        && app.current_view == View::EnvInput
                        && app.current_env_var().is_none()
                    {
                        app.env_input_cancel();
                        anyhow::bail!("No environment variable is awaiting a value");
                    }
                    let key = if app.current_view == View::List
                        && app.focus == crate::app::FocusArea::Tabs
                    {
                        super::shortcuts::sidebar_key(key)
                    } else {
                        key
                    };
                    cli::dispatch_key(app, key, &self.channels.refresh_tx)?;
                }
                if app.should_quit {
                    app.should_quit = false;
                    self.dispatch(Action::RequestClose);
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn cancel(&mut self) {
        let Some(app) = self.app.as_mut() else {
            return;
        };
        match app.current_view {
            View::Installing => loading::installing_key(app, KeyCode::Esc, &mut self.channels),
            View::Preflighting => {
                loading::preflighting_key(app, KeyCode::Esc, &mut self.channels);
            }
            View::SourceSyncing => {
                if !self.source_cancelling {
                    if let Some(tx) = app.source_sync_cancel_tx.take() {
                        let _ = tx.send(());
                    }
                    self.source_cancelling = true;
                    app.source_sync_status = Some(crate::app::SyncStatus::Error(
                        "Cancelling; waiting for the source worker to stop...".into(),
                    ));
                }
            }
            View::Loading => {}
            _ => {
                if let Err(error) = cli::dispatch_key(app, KeyCode::Esc, &self.channels.refresh_tx)
                {
                    self.error = Some(format!("{error:#}"));
                }
            }
        }
    }
}

fn self_busy_view(app: &crate::app::App) -> bool {
    matches!(
        app.current_view,
        View::Loading | View::Installing | View::Preflighting | View::SourceSyncing
    )
}

fn source_input_view(view: View) -> bool {
    matches!(
        view,
        View::SourceAddUrl | View::SourceAddBranch | View::SourceAddPath | View::SourceAddRoot
    )
}

fn select_row(app: &mut crate::app::App, index: usize) -> bool {
    if app.current_view == View::Sources && index <= app.source_entries.len() {
        app.source_list_index = index;
        return true;
    }
    if app.current_view != View::List {
        return false;
    }
    let valid = match app.tab {
        Tab::McpServers
            if app
                .visible_mcp_servers()
                .iter()
                .any(|(visible, _)| *visible == index) =>
        {
            app.mcp_index = index;
            true
        }
        Tab::Plugins
            if app
                .visible_plugins()
                .iter()
                .any(|(visible, _)| *visible == index) =>
        {
            app.plugin_index = index;
            true
        }
        _ => app.tree_views.get_mut(&app.tab).is_some_and(|tree| {
            if index < tree.visible_indices.len() {
                tree.cursor = index;
                true
            } else {
                false
            }
        }),
    };
    if valid {
        app.focus_content();
    }
    valid
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::test_support::{fresh_app, make_mcp_server};

    fn runtime_with_app() -> Runtime {
        let mut runtime = Runtime::empty();
        runtime.app = Some(fresh_app());
        runtime
    }

    #[test]
    fn invalid_row_does_not_toggle_previously_focused_server() {
        let mut runtime = runtime_with_app();
        let app = runtime.app.as_mut().unwrap();
        app.tab = Tab::McpServers;
        app.mcp_servers.push(make_mcp_server());
        runtime.dispatch(Action::ToggleRow(99));
        assert!(!runtime.app.as_ref().unwrap().mcp_servers[0].selected);
        runtime.dispatch(Action::ToggleRow(0));
        assert!(runtime.app.as_ref().unwrap().mcp_servers[0].selected);
    }

    #[test]
    fn stale_row_action_cannot_change_selection_behind_prompt() {
        let mut runtime = runtime_with_app();
        let app = runtime.app.as_mut().unwrap();
        app.tab = Tab::McpServers;
        app.mcp_servers.push(make_mcp_server());
        app.current_view = View::EnvInput;
        runtime.dispatch(Action::ToggleRow(0));
        assert!(!runtime.app.as_ref().unwrap().mcp_servers[0].selected);
    }

    #[test]
    fn unavailable_codex_tab_is_ignored() {
        let mut runtime = runtime_with_app();
        let app = runtime.app.as_mut().unwrap();
        app.available_tabs = Tab::for_cli(TargetCli::Codex);
        runtime.dispatch(Action::SelectTab(Tab::Plugins));
        assert_eq!(runtime.app.as_ref().unwrap().tab, Tab::Skills);
    }

    #[test]
    fn unicode_and_pasted_input_is_preserved_and_late_input_ignored() {
        let mut runtime = runtime_with_app();
        runtime.app.as_mut().unwrap().current_view = View::SourceAddPath;
        runtime.dispatch(Action::SetInput("/tmp/한국어 폴더".into()));
        assert_eq!(
            runtime.app.as_ref().unwrap().source_input_buffer,
            "/tmp/한국어 폴더"
        );
        runtime.app.as_mut().unwrap().current_view = View::List;
        runtime.dispatch(Action::SetInput("late event".into()));
        assert_eq!(
            runtime.app.as_ref().unwrap().source_input_buffer,
            "/tmp/한국어 폴더"
        );
    }

    #[test]
    fn invalid_env_prompt_reports_error_instead_of_panicking() {
        let mut runtime = runtime_with_app();
        let app = runtime.app.as_mut().unwrap();
        app.current_view = View::EnvInput;
        app.env_input_buffer = "secret".into();
        runtime.dispatch(Action::Key(KeyCode::Enter));
        assert!(
            runtime
                .error
                .as_deref()
                .unwrap()
                .contains("No environment variable")
        );
        assert_eq!(runtime.app.as_ref().unwrap().current_view, View::List);
        assert!(runtime.app.as_ref().unwrap().env_input_buffer.is_empty());
    }

    #[test]
    fn empty_install_does_not_start_worker() {
        let mut runtime = runtime_with_app();
        runtime.dispatch(Action::Key(KeyCode::Char('i')));
        assert!(!runtime.busy());
        assert_eq!(runtime.app.as_ref().unwrap().current_view, View::List);
    }

    #[test]
    fn cancelling_sync_waits_for_worker_and_blocks_new_source_actions() {
        let mut runtime = runtime_with_app();
        let (result_tx, result_rx) = std::sync::mpsc::channel();
        let (cancel_tx, cancel_rx) = std::sync::mpsc::channel();
        let app = runtime.app.as_mut().unwrap();
        app.current_view = View::SourceSyncing;
        app.source_sync_rx = Some(result_rx);
        app.source_sync_cancel_tx = Some(cancel_tx);
        runtime.dispatch(Action::Cancel);
        assert!(cancel_rx.try_recv().is_ok());
        assert!(runtime.busy());
        runtime.dispatch(Action::Key(KeyCode::Char('a')));
        assert_eq!(
            runtime.app.as_ref().unwrap().current_view,
            View::SourceSyncing
        );
        assert!(runtime.app.as_ref().unwrap().source_sync_rx.is_some());
        drop(result_tx);
        runtime.tick();
        assert!(!runtime.busy());
        assert_eq!(runtime.app.as_ref().unwrap().current_view, View::Sources);
    }

    #[test]
    fn cancelling_before_first_install_tick_clears_queue_and_keeps_refresh() {
        let mut runtime = runtime_with_app();
        let app = runtime.app.as_mut().unwrap();
        app.current_view = View::Installing;
        app.processing_queue = vec![0, 1];
        runtime.dispatch(Action::Cancel);
        let app = runtime.app.as_ref().unwrap();
        assert!(app.processing_queue.is_empty());
        assert!(app.needs_refresh);
        assert!(!app.processing_complete);
    }

    #[test]
    fn dismissing_close_preserves_selections_and_does_not_exit() {
        let mut runtime = runtime_with_app();
        let mut server = make_mcp_server();
        server.selected = true;
        runtime.app.as_mut().unwrap().mcp_servers.push(server);
        runtime.dispatch(Action::RequestClose);
        assert!(runtime.close_requested);
        runtime.dispatch(Action::Key(KeyCode::Char('a')));
        runtime.dispatch(Action::DismissClose);
        assert!(!runtime.should_close);
        assert!(!runtime.close_requested);
        assert!(runtime.app.as_ref().unwrap().mcp_servers[0].selected);
    }

    #[test]
    fn completed_processing_back_button_returns_to_list() {
        let mut runtime = runtime_with_app();
        let app = runtime.app.as_mut().unwrap();
        app.current_view = View::Installing;
        app.processing_complete = true;
        runtime.dispatch(Action::Back);
        assert_eq!(runtime.app.as_ref().unwrap().current_view, View::List);
    }

    #[test]
    fn mouse_command_works_after_keyboard_focused_tabs() {
        let mut runtime = runtime_with_app();
        let app = runtime.app.as_mut().unwrap();
        app.tab = Tab::McpServers;
        app.focus_tabs();
        app.mcp_servers.push(make_mcp_server());
        runtime.dispatch(Action::Command(KeyCode::Char('a')));
        assert!(runtime.app.as_ref().unwrap().mcp_servers[0].selected);
        assert_eq!(
            runtime.app.as_ref().unwrap().focus,
            crate::app::FocusArea::Content
        );
    }

    #[test]
    fn physical_key_preserves_tab_navigation_mode() {
        let mut runtime = runtime_with_app();
        let app = runtime.app.as_mut().unwrap();
        app.tab = Tab::McpServers;
        app.focus_tabs();
        app.mcp_servers.push(make_mcp_server());
        runtime.dispatch(Action::Key(KeyCode::Char('a')));
        assert!(!runtime.app.as_ref().unwrap().mcp_servers[0].selected);
        assert_eq!(
            runtime.app.as_ref().unwrap().focus,
            crate::app::FocusArea::Tabs
        );
    }

    #[test]
    fn escape_dismisses_close_dialog_without_discarding_selection() {
        let mut runtime = runtime_with_app();
        let mut server = make_mcp_server();
        server.selected = true;
        runtime.app.as_mut().unwrap().mcp_servers.push(server);
        runtime.dispatch(Action::RequestClose);
        runtime.dispatch(Action::Key(KeyCode::Esc));
        assert!(!runtime.close_requested);
        assert!(!runtime.should_close);
        assert!(runtime.app.as_ref().unwrap().mcp_servers[0].selected);
    }

    #[test]
    fn unsolicited_close_confirmation_is_ignored() {
        let mut runtime = runtime_with_app();
        runtime.dispatch(Action::ConfirmClose);
        assert!(!runtime.should_close);
    }
    #[test]
    fn codex_scope_keyboard_and_button_commands_cannot_enable_local_scope() {
        for action in [
            Action::Key(KeyCode::Char('o')),
            Action::Command(KeyCode::Char('o')),
        ] {
            let mut runtime = runtime_with_app();
            let app = runtime.app.as_mut().unwrap();
            app.target_cli = Some(TargetCli::Codex);
            app.tab = Tab::McpServers;
            app.mcp_scope = crate::mcp::McpScope::Local;
            runtime.dispatch(action);
            let app = runtime.app.as_ref().unwrap();
            assert_eq!(app.current_view, View::List);
            assert_eq!(app.mcp_scope, crate::mcp::McpScope::User);
            assert!(
                app.status_message
                    .as_deref()
                    .unwrap()
                    .contains("Codex MCP uses user scope")
            );
        }
    }

    #[test]
    fn selecting_codex_clears_scope_left_from_claude_without_starting_a_scan() {
        let mut app = fresh_app();
        app.current_view = View::CliSelection;
        app.target_cli = Some(TargetCli::Claude);
        app.mcp_scope = crate::mcp::McpScope::Local;
        app.select_cli(TargetCli::Codex).unwrap();
        assert_eq!(app.mcp_scope, crate::mcp::McpScope::User);
        assert_eq!(app.target_cli, Some(TargetCli::Codex));
        assert_eq!(app.current_view, View::Loading);
    }

    #[test]
    fn claude_scope_command_still_opens_project_path_form() {
        let mut runtime = runtime_with_app();
        let app = runtime.app.as_mut().unwrap();
        app.target_cli = Some(TargetCli::Claude);
        app.tab = Tab::McpServers;
        runtime.dispatch(Action::Command(KeyCode::Char('o')));
        let app = runtime.app.as_ref().unwrap();
        assert_eq!(app.current_view, View::ProjectPath);
        assert_eq!(app.mcp_scope, crate::mcp::McpScope::Local);
    }
}

#[cfg(test)]
mod search_tests {
    use super::*;
    use crate::app::test_support::{fresh_app, make_mcp_server};
    use crate::tree::test_support::make_component;

    fn runtime() -> Runtime {
        let mut state = Runtime::empty();
        state.app = Some(fresh_app());
        state
    }

    #[test]
    fn escape_clears_query_before_navigation_and_preserves_pending_selections() {
        let mut state = runtime();
        state
            .app
            .as_mut()
            .unwrap()
            .components
            .push(make_component("hidden-component"));
        state.dispatch(Action::SetSearch("target".into()));
        state.dispatch(Action::Key(KeyCode::Esc));
        let app = state.app.as_ref().unwrap();
        assert!(app.search_query.is_empty());
        assert_eq!(app.current_view, View::List);
        assert!(app.components[0].selected);
        state.dispatch(Action::Key(KeyCode::Esc));
        assert_eq!(state.app.as_ref().unwrap().current_view, View::ConfirmExit);
    }

    #[test]
    fn modal_input_and_back_controls_preserve_search_query() {
        for view in [View::Diff, View::EnvInput, View::ProjectPath] {
            let mut state = runtime();
            state.dispatch(Action::SetSearch("한국어 query".into()));
            state.app.as_mut().unwrap().current_view = view;
            state.dispatch(Action::SetSearch("late text".into()));
            state.dispatch(Action::ClearSearch);
            state.dispatch(Action::Back);
            let app = state.app.as_ref().unwrap();
            assert_eq!(app.search_query, "한국어 query");
            assert_eq!(app.current_view, View::List);
        }
    }

    #[test]
    fn stale_hidden_mcp_row_cannot_toggle_selection_after_query_changes() {
        let mut state = runtime();
        let app = state.app.as_mut().unwrap();
        app.tab = Tab::McpServers;
        app.mcp_servers = ["hidden", "target"]
            .map(|name| {
                let mut server = make_mcp_server();
                server.def.name = name.into();
                server
            })
            .into();
        state.dispatch(Action::SetSearch("target".into()));
        state.dispatch(Action::ToggleRow(0));
        let app = state.app.as_ref().unwrap();
        assert_eq!(app.mcp_index, 1);
        assert!(!app.mcp_servers[0].selected);
        state.dispatch(Action::ToggleRow(1));
        assert!(state.app.as_ref().unwrap().mcp_servers[1].selected);
        state.dispatch(Action::SetSearch("no-match".into()));
        state.dispatch(Action::Key(KeyCode::Char(' ')));
        assert!(!state.app.as_ref().unwrap().mcp_servers[0].selected);
        assert!(state.app.as_ref().unwrap().mcp_servers[1].selected);
    }

    #[test]
    fn removal_confirmation_counts_only_matching_selection_and_blocks_query_changes() {
        let mut state = runtime();
        let app = state.app.as_mut().unwrap();
        app.tab = Tab::McpServers;
        app.mcp_servers = ["hidden", "target"]
            .map(|name| {
                let mut server = make_mcp_server();
                server.def.name = name.into();
                server.selected = true;
                server
            })
            .into();
        state.dispatch(Action::SetSearch("target".into()));
        state.dispatch(Action::Command(KeyCode::Char('r')));
        assert!(state.remove_requested);
        assert_eq!(state.remove_count, 1);
        state.dispatch(Action::SetSearch("hidden".into()));
        state.dispatch(Action::ClearSearch);
        assert_eq!(state.app.as_ref().unwrap().search_query, "target");
        state.dispatch(Action::CancelRemove);
        assert_eq!(state.app.as_ref().unwrap().current_view, View::List);
        assert!(
            state
                .app
                .as_ref()
                .unwrap()
                .mcp_servers
                .iter()
                .all(|server| server.selected)
        );
        state.dispatch(Action::Command(KeyCode::Char('r')));
        state.dispatch(Action::ConfirmRemove);
        assert_eq!(state.app.as_ref().unwrap().processing_queue, vec![1]);
        assert_eq!(state.app.as_ref().unwrap().current_view, View::Preflighting);
    }
    #[test]
    fn search_and_clear_return_logical_keyboard_focus_from_tabs_to_content() {
        let mut state = runtime();
        state.app.as_mut().unwrap().focus_tabs();
        state.dispatch(Action::SetSearch("demo".into()));
        assert_eq!(
            state.app.as_ref().unwrap().focus,
            crate::app::FocusArea::Content
        );
        state.app.as_mut().unwrap().focus_tabs();
        state.dispatch(Action::ClearSearch);
        let app = state.app.as_ref().unwrap();
        assert_eq!(app.focus, crate::app::FocusArea::Content);
        assert!(app.search_query.is_empty());
    }
}
