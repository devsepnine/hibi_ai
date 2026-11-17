use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;

use anyhow::Result;

use crate::app::{App, View};
use crate::loading::{self, ProcessingChannels};

pub struct Runtime {
    pub app: Option<App>,
    pub error: Option<String>,
    pub close_requested: bool,
    pub remove_requested: bool,
    pub remove_count: usize,
    pub should_close: bool,
    pub window_size: iced::Size,
    pub(super) channels: ProcessingChannels,
    pub(super) init_rx: Option<Receiver<Result<App>>>,
    pub(super) closing: bool,
    pub(super) source_cancelling: bool,
    pub(super) appearance: super::preferences::Writer,
}

impl Runtime {
    pub fn new() -> Self {
        let mut runtime = Self::empty();
        runtime.start_initialization();
        runtime
    }

    pub(super) fn empty() -> Self {
        Self {
            app: None,
            error: None,
            close_requested: false,
            remove_requested: false,
            remove_count: 0,
            should_close: false,
            window_size: iced::Size::new(1120.0, 780.0),
            channels: ProcessingChannels::new(),
            init_rx: None,
            closing: false,
            source_cancelling: false,
            appearance: Default::default(),
        }
    }

    pub(super) fn start_initialization(&mut self) {
        let (tx, rx) = mpsc::channel();
        self.init_rx = Some(rx);
        self.error = None;
        thread::spawn(move || {
            let result = App::new().map(|mut app| {
                super::preferences::restore(&mut app);
                app
            });
            let _ = tx.send(result);
        });
    }

    pub fn busy(&self) -> bool {
        self.appearance.busy() || self.operation_active()
    }

    pub(super) fn operation_active(&self) -> bool {
        self.init_rx.is_some()
            || self.app.as_ref().is_some_and(|app| {
                matches!(
                    app.current_view,
                    View::Loading | View::Preflighting | View::SourceSyncing
                ) || (app.current_view == View::Installing && !app.processing_complete)
            })
    }

    pub fn tick(&mut self) {
        self.poll_initialization();
        if let Some(Err(error)) = self.appearance.tick() {
            self.error = Some(format!("Could not save appearance settings: {error:#}"));
        }
        let Some(app) = self.app.as_mut() else {
            if self.closing && !self.busy() {
                self.should_close = true;
            }
            return;
        };
        let result = match app.current_view {
            View::Loading => {
                loading::tick_loading(app, &self.channels.refresh_rx);
                Ok(())
            }
            View::Preflighting => loading::tick_preflighting(app, &mut self.channels),
            View::Installing => {
                loading::tick_installing(app, &mut self.channels);
                Ok(())
            }
            View::SourceSyncing => {
                app.tick();
                if self.source_cancelling {
                    self.source_cancelling = !poll_cancelled_sync(app);
                } else {
                    app.check_source_sync();
                }
                Ok(())
            }
            _ => Ok(()),
        };
        if let Err(error) = result {
            self.error = Some(format!("{error:#}"));
        }
        if self.closing && !self.busy() {
            self.should_close = true;
        }
    }

    pub(super) fn request_removal(&mut self) {
        let Some(app) = self.app.as_mut() else {
            return;
        };
        self.remove_count = removal_count(app);
        self.remove_requested = self.remove_count > 0;
        if !self.remove_requested {
            app.status_message = Some("Select items in this category to remove.".into());
        }
    }

    pub(super) fn confirm_removal(&mut self) {
        if !self.remove_requested {
            return;
        }
        self.remove_requested = false;
        let Some(app) = self.app.as_mut() else {
            return;
        };
        if app.current_view != View::List || removal_count(app) != self.remove_count {
            self.error =
                Some("Selection changed. Review your selection before removing items.".into());
            self.remove_count = 0;
            return;
        }
        self.remove_count = 0;
        if let Err(error) = app.remove_selected() {
            let message = format!("{error:#}");
            app.status_message = Some(message.clone());
            self.error = Some(message);
        }
    }

    fn poll_initialization(&mut self) {
        let Some(rx) = self.init_rx.as_ref() else {
            return;
        };
        match rx.try_recv() {
            Ok(Ok(app)) => {
                self.app = Some(app);
                self.init_rx = None;
            }
            Ok(Err(error)) => {
                self.error = Some(format!("{error:#}"));
                self.init_rx = None;
            }
            Err(TryRecvError::Disconnected) => {
                self.error =
                    Some("Initialization worker stopped unexpectedly. Retry to reload.".into());
                self.init_rx = None;
            }
            Err(TryRecvError::Empty) => {}
        }
    }
}

fn poll_cancelled_sync(app: &mut App) -> bool {
    match app.source_sync_rx.as_ref().map(Receiver::try_recv) {
        Some(Err(TryRecvError::Empty)) => false,
        _ => {
            app.source_sync_rx = None;
            app.source_sync_cancel_tx = None;
            app.source_sync_status = Some(crate::app::SyncStatus::Error("Cancelled".into()));
            app.current_view = View::Sources;
            true
        }
    }
}

fn removal_count(app: &App) -> usize {
    match app.tab {
        crate::app::Tab::McpServers => app
            .visible_mcp_servers()
            .iter()
            .filter(|(_, server)| server.selected)
            .count(),
        crate::app::Tab::Plugins => app
            .visible_plugins()
            .iter()
            .filter(|(_, plugin)| plugin.selected)
            .count(),
        _ => app
            .visible_components()
            .iter()
            .filter(|(_, component)| component.selected)
            .count(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gui::actions::Action;

    #[test]
    fn initialization_failure_is_visible_and_does_not_stay_busy() {
        let mut runtime = Runtime::empty();
        let (tx, rx) = mpsc::channel();
        runtime.init_rx = Some(rx);
        tx.send(Err(anyhow::anyhow!("Permission denied"))).unwrap();
        runtime.tick();
        assert_eq!(runtime.error.as_deref(), Some("Permission denied"));
        assert!(!runtime.busy());
        assert!(runtime.app.is_none());
    }

    #[test]
    fn initialization_panic_disconnect_is_reported() {
        let mut runtime = Runtime::empty();
        let (tx, rx) = mpsc::channel();
        runtime.init_rx = Some(rx);
        drop(tx);
        runtime.tick();
        assert!(runtime.error.as_deref().unwrap().contains("unexpectedly"));
        assert!(!runtime.busy());
    }

    #[test]
    fn closing_waits_for_initialization_to_settle() {
        let mut runtime = Runtime::empty();
        let (tx, rx) = mpsc::channel();
        runtime.init_rx = Some(rx);
        runtime.dispatch(Action::RequestClose);
        assert!(runtime.close_requested);
        runtime.dispatch(Action::ConfirmClose);
        runtime.tick();
        assert!(!runtime.should_close);
        tx.send(Err(anyhow::anyhow!("failed"))).unwrap();
        runtime.tick();
        assert!(runtime.should_close);
    }
}

#[cfg(test)]
mod removal_tests {
    use super::*;
    use crate::app::test_support::{fresh_app, make_mcp_server};
    use crate::app::{ComponentFilter, Tab};
    use crate::component::InstallStatus;
    use crate::gui::actions::Action;
    use crate::tree::test_support::make_component;
    use crossterm::event::KeyCode;

    fn runtime() -> Runtime {
        let mut state = Runtime::empty();
        state.app = Some(fresh_app());
        state
    }

    fn selected_component(state: &mut Runtime) {
        state
            .app
            .as_mut()
            .unwrap()
            .components
            .push(make_component("demo"));
    }

    #[test]
    fn no_selection_in_current_category_does_not_open_removal_confirmation() {
        let mut state = runtime();
        let mut server = make_mcp_server();
        server.selected = true;
        state.app.as_mut().unwrap().mcp_servers.push(server);
        state.dispatch(Action::Command(KeyCode::Char('r')));
        assert!(!state.remove_requested);
        assert_eq!(state.remove_count, 0);
        assert_eq!(state.app.as_ref().unwrap().current_view, View::List);
        assert!(state.app.as_ref().unwrap().processing_queue.is_empty());
    }

    #[test]
    fn confirmation_counts_and_removes_only_selected_visible_scope() {
        let mut state = runtime();
        let app = state.app.as_mut().unwrap();
        app.dest_dir = std::path::PathBuf::from("/tmp/hibi-removal-fixture");
        let mut visible = make_component("visible/item");
        visible.status = InstallStatus::External;
        visible.selected = true;
        let mut hidden = make_component("hidden/item");
        hidden.status = InstallStatus::External;
        hidden.selected = true;
        let mut other_status = make_component("modified/item");
        other_status.status = InstallStatus::Modified;
        app.components = vec![visible, hidden, other_status];
        app.hidden_folders
            .insert(app.dest_dir.join("skills/hidden"));
        app.component_filter = ComponentFilter::External;
        state.dispatch(Action::Key(KeyCode::Char('r')));
        assert!(state.remove_requested);
        assert_eq!(state.remove_count, 1);
        assert!(state.app.as_ref().unwrap().processing_queue.is_empty());
        state.dispatch(Action::ConfirmRemove);
        let app = state.app.as_ref().unwrap();
        assert_eq!(app.processing_queue, vec![0]);
        assert_eq!(app.current_view, View::Installing);
        assert!(app.components[1].selected);
        assert!(app.components[2].selected);
    }

    #[test]
    fn cancelling_removal_preserves_selection_and_does_not_start_work() {
        for cancel in [
            Action::CancelRemove,
            Action::Cancel,
            Action::Back,
            Action::Key(KeyCode::Esc),
        ] {
            let mut state = runtime();
            selected_component(&mut state);
            state.dispatch(Action::Command(KeyCode::Char('r')));
            state.dispatch(cancel);
            assert!(!state.remove_requested);
            assert_eq!(state.app.as_ref().unwrap().current_view, View::List);
            assert!(state.app.as_ref().unwrap().components[0].selected);
            assert!(state.app.as_ref().unwrap().processing_queue.is_empty());
        }
    }

    #[test]
    fn confirmation_runs_once_and_unsolicited_confirmation_is_ignored() {
        let mut state = runtime();
        selected_component(&mut state);
        state.dispatch(Action::ConfirmRemove);
        assert_eq!(state.app.as_ref().unwrap().current_view, View::List);
        state.dispatch(Action::Command(KeyCode::Char('r')));
        state.dispatch(Action::ConfirmRemove);
        let first_log = state.app.as_ref().unwrap().processing_log.clone();
        state.dispatch(Action::ConfirmRemove);
        assert!(!state.remove_requested);
        assert_eq!(state.app.as_ref().unwrap().processing_queue, vec![0]);
        assert_eq!(state.app.as_ref().unwrap().processing_log, first_log);
    }

    #[test]
    fn removal_confirmation_blocks_selection_navigation_install_and_enter() {
        let mut state = runtime();
        selected_component(&mut state);
        state.dispatch(Action::Key(KeyCode::Char('r')));
        for action in [
            Action::ToggleRow(0),
            Action::SelectTab(Tab::McpServers),
            Action::SetFilter(ComponentFilter::New),
            Action::Command(KeyCode::Char('a')),
            Action::Key(KeyCode::Char('i')),
            Action::Key(KeyCode::Enter),
            Action::ToggleTheme,
            Action::OpenSources,
        ] {
            state.dispatch(action);
        }
        assert!(state.remove_requested);
        let app = state.app.as_ref().unwrap();
        assert_eq!(app.tab, Tab::Skills);
        assert_eq!(app.component_filter, ComponentFilter::All);
        assert!(app.components[0].selected);
        assert_eq!(app.current_view, View::List);
        assert!(app.processing_queue.is_empty());
    }

    #[test]
    fn closing_supersedes_removal_and_dismiss_does_not_remove_items() {
        let mut state = runtime();
        selected_component(&mut state);
        state.dispatch(Action::Key(KeyCode::Char('r')));
        state.dispatch(Action::RequestClose);
        assert!(state.close_requested);
        assert!(!state.remove_requested);
        state.dispatch(Action::ConfirmRemove);
        state.dispatch(Action::DismissClose);
        state.dispatch(Action::ConfirmRemove);
        assert!(!state.should_close);
        assert!(state.app.as_ref().unwrap().components[0].selected);
        assert!(state.app.as_ref().unwrap().processing_queue.is_empty());
    }

    #[test]
    fn stale_confirmation_does_not_remove_changed_selection() {
        let mut state = runtime();
        selected_component(&mut state);
        state.dispatch(Action::Key(KeyCode::Char('r')));
        state.app.as_mut().unwrap().components[0].selected = false;
        state.dispatch(Action::ConfirmRemove);
        assert!(!state.remove_requested);
        assert!(
            state
                .error
                .as_deref()
                .unwrap()
                .contains("Selection changed")
        );
        assert!(state.app.as_ref().unwrap().processing_queue.is_empty());
    }
}

#[cfg(test)]
mod cancelled_source_tests {
    use super::*;
    use crate::app::sources::{RescanResult, SyncPayload};
    use crate::app::test_support::{fresh_app, make_mcp_server, make_plugin};
    use crate::gui::actions::Action;
    use crate::source::{ResolvedSource, SourceKind};
    use crate::tree::test_support::make_component;
    use std::path::Path;

    fn runtime_with_inventory() -> Runtime {
        let mut state = Runtime::empty();
        let mut app = fresh_app();
        app.current_view = View::SourceSyncing;
        app.sources = vec![ResolvedSource {
            label: "existing user source".into(),
            kind: SourceKind::Local,
            path: "/test/existing-source".into(),
            is_stale: false,
            branch: None,
            map_to: None,
        }];
        app.components.push(make_component("existing-component"));
        app.mcp_servers.push(make_mcp_server());
        app.plugins.push(make_plugin());
        state.app = Some(app);
        state
    }

    fn replacement_payload() -> SyncPayload {
        SyncPayload {
            resolved: vec![ResolvedSource::bundled(Path::new("/replacement-only"))],
            summaries: vec!["cancelled".into()],
            had_error: true,
            rescan: Some(RescanResult {
                components: Vec::new(),
                mcp_servers: Vec::new(),
                plugins: Vec::new(),
            }),
        }
    }

    fn assert_inventory_preserved(state: &Runtime) {
        let app = state.app.as_ref().unwrap();
        assert_eq!(app.sources.len(), 1);
        assert_eq!(app.sources[0].label, "existing user source");
        assert_eq!(app.sources[0].kind, SourceKind::Local);
        assert_eq!(app.components.len(), 1);
        assert_eq!(app.components[0].name, "existing-component");
        assert!(app.components[0].selected);
        assert_eq!(app.mcp_servers.len(), 1);
        assert_eq!(app.mcp_servers[0].def.name, "demo");
        assert_eq!(app.plugins.len(), 1);
        assert_eq!(app.plugins[0].def.name, "demo");
    }

    #[test]
    fn cancelled_sync_discards_payload_and_rescan_preserving_existing_inventory() {
        let mut state = runtime_with_inventory();
        let (tx, rx) = mpsc::channel();
        state.app.as_mut().unwrap().source_sync_rx = Some(rx);
        state.dispatch(Action::Cancel);
        tx.send(replacement_payload()).unwrap();
        state.tick();
        assert_inventory_preserved(&state);
        assert_eq!(state.app.as_ref().unwrap().current_view, View::Sources);
        assert!(matches!(state.app.as_ref().unwrap().source_sync_status,
            Some(crate::app::SyncStatus::Error(ref message)) if message == "Cancelled"));
        assert!(!state.source_cancelling);
        assert!(!state.busy());
    }

    #[test]
    fn closing_after_cancel_waits_for_worker_completion_without_applying_its_result() {
        let mut state = runtime_with_inventory();
        let (tx, rx) = mpsc::channel();
        state.app.as_mut().unwrap().source_sync_rx = Some(rx);
        state.dispatch(Action::RequestClose);
        state.dispatch(Action::ConfirmClose);
        state.tick();
        assert!(!state.should_close);
        assert!(state.source_cancelling);
        assert!(state.busy());
        assert_inventory_preserved(&state);
        tx.send(replacement_payload()).unwrap();
        state.tick();
        assert!(state.should_close);
        assert!(!state.source_cancelling);
        assert_inventory_preserved(&state);
    }

    #[test]
    fn cancelled_sync_without_receiver_settles_defensively() {
        let mut state = runtime_with_inventory();
        state.source_cancelling = true;
        state.tick();
        assert!(!state.source_cancelling);
        assert!(!state.busy());
        assert_eq!(state.app.as_ref().unwrap().current_view, View::Sources);
        assert_inventory_preserved(&state);
    }

    #[test]
    fn cancelled_sync_with_disconnected_worker_settles_without_replacing_inventory() {
        let mut state = runtime_with_inventory();
        let (tx, rx) = mpsc::channel();
        state.app.as_mut().unwrap().source_sync_rx = Some(rx);
        state.dispatch(Action::Cancel);
        drop(tx);
        state.tick();
        assert!(!state.source_cancelling);
        assert!(!state.busy());
        assert_eq!(state.app.as_ref().unwrap().current_view, View::Sources);
        assert_inventory_preserved(&state);
    }
}
