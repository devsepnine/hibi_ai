use super::{actions::Action, runtime::Runtime};
use crate::app::test_support::fresh_app;
use crate::app::{FocusArea, Tab, TargetCli, View};
use crate::tree::{TreeView, test_support::make_component};
use crossterm::event::KeyCode;
use iced::advanced::widget::{Operation, Tree};
use iced::advanced::{Layout, Shell, clipboard, layout, mouse};
use iced::keyboard::{Key, Location, Modifiers, key};
use iced::{Event, Font, Pixels, Point, Rectangle, Renderer, Size};

use super::{Message, shortcuts};

fn runtime() -> Runtime {
    let mut app = fresh_app();
    app.available_tabs = Tab::for_cli(TargetCli::Codex);
    app.components = ["folder/first.md", "folder/second.md", "other.md"]
        .map(make_component)
        .into();
    let filtered = app.components.iter().enumerate().collect::<Vec<_>>();
    let tree = TreeView::build_from_components(&app.components, &filtered);
    app.tree_views.insert(Tab::Skills, tree);
    let mut state = Runtime::empty();
    state.app = Some(app);
    state
}

#[test]
fn clicking_category_keeps_keyboard_navigation_in_the_sidebar() {
    let mut state = runtime();
    state.dispatch(Action::SelectTab(Tab::Skills));
    assert_eq!(state.app.as_ref().unwrap().focus, FocusArea::Tabs);
    state.dispatch(Action::Key(KeyCode::Down));
    let app = state.app.as_ref().unwrap();
    assert_eq!(app.tab, Tab::Config);
    assert_eq!(app.focus, FocusArea::Tabs);
    state.dispatch(Action::Key(KeyCode::Up));
    assert_eq!(state.app.as_ref().unwrap().tab, Tab::Skills);
}

#[test]
fn sidebar_vertical_navigation_wraps_within_available_categories() {
    let mut state = runtime();
    state.dispatch(Action::Key(KeyCode::Char('1')));
    state.dispatch(Action::Key(KeyCode::Up));
    assert_eq!(state.app.as_ref().unwrap().tab, Tab::McpServers);
    state.dispatch(Action::Key(KeyCode::Down));
    assert_eq!(state.app.as_ref().unwrap().tab, Tab::Skills);
    assert_eq!(state.app.as_ref().unwrap().focus, FocusArea::Tabs);
}

#[test]
fn right_arrow_enters_list_and_row_click_restores_list_navigation() {
    let mut state = runtime();
    state.dispatch(Action::Key(KeyCode::Char('1')));
    state.dispatch(Action::Key(KeyCode::Right));
    let app = state.app.as_ref().unwrap();
    assert_eq!(app.tab, Tab::Skills);
    assert_eq!(app.focus, FocusArea::Content);
    state.dispatch(Action::Key(KeyCode::Down));
    assert_eq!(
        state.app.as_ref().unwrap().get_tree_view().unwrap().cursor,
        1
    );
    state.dispatch(Action::Key(KeyCode::Up));
    assert_eq!(
        state.app.as_ref().unwrap().get_tree_view().unwrap().cursor,
        0
    );
    state.dispatch(Action::Key(KeyCode::Char('1')));
    state.dispatch(Action::SelectRow(1));
    state.dispatch(Action::Key(KeyCode::Down));
    let app = state.app.as_ref().unwrap();
    assert_eq!(app.focus, FocusArea::Content);
    assert_eq!(app.get_tree_view().unwrap().cursor, 2);
}

#[test]
fn list_arrows_collapse_expand_and_return_to_parent_without_toggling_checks() {
    let mut state = runtime();
    state.dispatch(Action::SelectRow(0));
    state.dispatch(Action::Key(KeyCode::Left));
    let tree = state.app.as_ref().unwrap().get_tree_view().unwrap();
    assert!(!tree.is_current_folder_expanded());
    assert_eq!(tree.visible_indices.len(), 2);
    state.dispatch(Action::Key(KeyCode::Right));
    assert_eq!(
        state
            .app
            .as_ref()
            .unwrap()
            .get_tree_view()
            .unwrap()
            .visible_indices
            .len(),
        4
    );
    state.dispatch(Action::Key(KeyCode::Down));
    state.dispatch(Action::Key(KeyCode::Left));
    let app = state.app.as_ref().unwrap();
    let tree = app.get_tree_view().unwrap();
    assert_eq!(tree.cursor, 0);
    assert!(!tree.is_current_folder_expanded());
    assert!(app.components.iter().all(|component| component.selected));
}

#[test]
fn sidebar_directions_cannot_change_categories_behind_a_confirmation() {
    let mut state = runtime();
    state.dispatch(Action::Key(KeyCode::Char('1')));
    state.dispatch(Action::RequestClose);
    assert!(state.close_requested);
    state.dispatch(Action::Key(KeyCode::Down));
    assert_eq!(state.app.as_ref().unwrap().tab, Tab::Skills);
    state.dispatch(Action::DismissClose);
    state.dispatch(Action::Key(KeyCode::Down));
    assert_eq!(state.app.as_ref().unwrap().tab, Tab::Config);
    assert_eq!(state.app.as_ref().unwrap().current_view, View::List);
}

struct Desktop {
    state: Runtime,
    tree: Tree,
    renderer: Renderer,
}

impl Desktop {
    fn app(&self) -> &crate::app::App {
        self.state.app.as_ref().unwrap()
    }

    fn new() -> Self {
        let state = runtime();
        let tree = Tree::new(super::view(&state).as_widget());
        Self {
            state,
            tree,
            renderer: Renderer::new(Font::default(), Pixels(16.0)),
        }
    }

    fn operate(&mut self, operation: &mut dyn Operation<Message>) {
        let mut root = super::view(&self.state);
        self.tree.diff(root.as_widget());
        let node = root.as_widget_mut().layout(
            &mut self.tree,
            &self.renderer,
            &layout::Limits::new(Size::ZERO, self.state.window_size),
        );
        root.as_widget_mut().operate(
            &mut self.tree,
            Layout::new(&node),
            &self.renderer,
            &mut iced::advanced::widget::operation::black_box(operation),
        );
    }

    fn focus_search(&mut self) {
        self.operate(&mut iced::advanced::widget::operation::focusable::focus(
            "component-search".into(),
        ));
    }

    fn message(&mut self, message: Message) {
        let focus = shortcuts::navigation_focus(&message);
        let _ = super::update(&mut self.state, message);
        if super::search_available(&self.state)
            && let Some(mut focus) = focus
        {
            self.operate(&mut focus);
        }
    }

    fn key(&mut self, name: key::Named) -> iced::event::Status {
        let event = Event::Keyboard(iced::keyboard::Event::KeyPressed {
            key: Key::Named(name),
            modified_key: Key::Named(name),
            physical_key: key::Physical::Code(match name {
                key::Named::ArrowUp => key::Code::ArrowUp,
                key::Named::ArrowDown => key::Code::ArrowDown,
                key::Named::ArrowLeft => key::Code::ArrowLeft,
                key::Named::ArrowRight => key::Code::ArrowRight,
                _ => key::Code::Enter,
            }),
            location: Location::Standard,
            modifiers: Modifiers::NONE,
            text: None,
            repeat: false,
        });
        let mut root = super::view(&self.state);
        self.tree.diff(root.as_widget());
        let node = root.as_widget_mut().layout(
            &mut self.tree,
            &self.renderer,
            &layout::Limits::new(Size::ZERO, self.state.window_size),
        );
        let mut messages = Vec::new();
        let mut shell = Shell::new(&mut messages);
        root.as_widget_mut().update(
            &mut self.tree,
            &event,
            Layout::new(&node),
            mouse::Cursor::Unavailable,
            &self.renderer,
            &mut clipboard::Null,
            &mut shell,
            &Rectangle::new(Point::ORIGIN, self.state.window_size),
        );
        let status = shell.event_status();
        drop(root);
        for message in messages {
            self.message(message);
        }
        if let Some(message) = shortcuts::event_message(event, status) {
            self.message(message);
        }
        status
    }
}

#[test]
fn desktop_widgets_deliver_all_four_arrows_to_the_focused_list() {
    let mut desktop = Desktop::new();
    assert_eq!(
        desktop.key(key::Named::ArrowDown),
        iced::event::Status::Ignored
    );
    assert_eq!(desktop.app().get_tree_view().unwrap().cursor, 1);
    desktop.key(key::Named::ArrowUp);
    desktop.key(key::Named::ArrowLeft);
    assert!(!desktop.app().is_current_folder_expanded());
    desktop.key(key::Named::ArrowRight);
    assert!(desktop.app().is_current_folder_expanded());
}

#[test]
fn baseline_logical_focus_alone_leaves_search_capturing_folder_arrows() {
    let mut desktop = Desktop::new();
    desktop.focus_search();
    desktop.state.dispatch(Action::SelectRow(0));
    assert_eq!(desktop.app().focus, FocusArea::Content);
    assert_eq!(
        desktop.key(key::Named::ArrowLeft),
        iced::event::Status::Captured
    );
    assert!(desktop.app().is_current_folder_expanded());
}

#[test]
fn clicking_navigation_releases_native_search_focus_before_handling_arrows() {
    let mut desktop = Desktop::new();
    desktop.focus_search();
    desktop.message(Message::Action(Action::SelectTab(Tab::Skills)));
    desktop.key(key::Named::ArrowDown);
    assert_eq!(desktop.app().tab, Tab::Config);
    desktop.message(Message::Action(Action::SelectTab(Tab::Skills)));
    desktop.focus_search();
    desktop.message(Message::Action(Action::SelectRow(0)));
    assert_eq!(
        desktop.key(key::Named::ArrowLeft),
        iced::event::Status::Ignored
    );
    assert!(!desktop.app().is_current_folder_expanded());
}

#[test]
fn editing_search_keeps_native_cursor_arrows_and_does_not_collapse_folders() {
    let mut desktop = Desktop::new();
    desktop.focus_search();
    desktop.message(Message::Action(Action::SetSearch("folder".into())));
    assert_eq!(
        desktop.key(key::Named::ArrowLeft),
        iced::event::Status::Captured
    );
    assert!(desktop.app().is_current_folder_expanded());
}
