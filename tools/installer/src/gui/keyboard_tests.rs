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

fn render_page(state: &Runtime, tree: &mut Tree, renderer: &mut Renderer) -> Vec<u8> {
    use iced::Color;
    use iced::advanced::renderer::{Headless, Renderer as _};

    let viewport = Rectangle::with_size(state.window_size);
    let theme = super::theme(state);
    let mut element = super::view(state);
    tree.diff(element.as_widget());
    let node = element.as_widget_mut().layout(
        tree,
        renderer,
        &layout::Limits::new(Size::ZERO, state.window_size),
    );
    renderer.reset(viewport);
    element.as_widget().draw(
        tree,
        renderer,
        &theme,
        &iced::advanced::renderer::Style {
            text_color: theme.palette().text,
        },
        Layout::new(&node),
        mouse::Cursor::Unavailable,
        &viewport,
    );
    renderer.screenshot(
        Size::new(
            state.window_size.width as u32,
            state.window_size.height as u32,
        ),
        1.0,
        Color::from_rgb(1.0, 0.0, 1.0),
    )
}

#[test]
fn version_stays_in_the_bottom_right_when_status_appears() {
    let mut state = runtime();
    let mut renderer = Renderer::new(Font::default(), Pixels(16.0));
    let mut tree = Tree::new(super::view(&state).as_widget());
    for (width, height) in [(760, 560), (1120, 780)] {
        state.window_size = Size::new(width as f32, height as f32);
        state.updater = super::updates::State::Idle;
        state.app.as_mut().unwrap().status_message = None;
        let idle = render_page(&state, &mut tree, &mut renderer);
        state.app.as_mut().unwrap().status_message = Some("Refreshing components".into());
        let busy = render_page(&state, &mut tree, &mut renderer);
        state.updater = super::updates::State::Checking;
        let checking = render_page(&state, &mut tree, &mut renderer);
        let mut has_version = false;
        for y in height - 36..height {
            let start = (y * width + width - 80) * 4;
            let end = (y + 1) * width * 4;
            assert_eq!(&idle[start..end], &busy[start..end]);
            has_version |= idle[start..end] != checking[start..end];
        }
        assert!(has_version);
    }
}

#[test]
fn theme_changes_repaint_the_entire_page_background() {
    let mut state = runtime();
    let mut renderer = Renderer::new(Font::default(), Pixels(16.0));
    let mut tree = Tree::new(super::view(&state).as_widget());
    for dark in [true, false, true] {
        state.theme_choice = if dark {
            crate::fs::installer::preferences::Appearance::Dark
        } else {
            crate::fs::installer::preferences::Appearance::Light
        };
        let pixels = render_page(&state, &mut tree, &mut renderer);
        let corners = [(0, 0), (1119, 0), (0, 779), (1119, 779)].map(|(x, y)| {
            let offset = (y * 1120 + x) * 4;
            &pixels[offset..offset + 4]
        });
        for pixel in corners {
            assert_eq!(pixel[3], 255);
            assert!(pixel[..3].iter().all(|&channel| if dark {
                channel < 100
            } else {
                channel > 200
            }));
        }
        let top_edge = &pixels[..1120 * 4];
        assert!(top_edge.chunks_exact(4).any(|pixel| pixel != corners[0]));
    }
}

#[test]
fn system_changes_follow_the_os_until_a_palette_is_selected() {
    use crate::fs::installer::preferences::Appearance;
    use iced::theme::Mode;
    let mut state = runtime();
    for mode in [Mode::Dark, Mode::Light] {
        let _ = super::update(&mut state, Message::SystemTheme(mode));
        assert_eq!(
            super::theme(&state).extended_palette().is_dark,
            mode == Mode::Dark
        );
        assert!(!state.appearance.busy());
    }
    state.dispatch(Action::SetAppearance(Appearance::Palette("Nord".into())));
    let nord = super::theme(&state);
    let _ = super::update(&mut state, Message::SystemTheme(Mode::Light));
    assert_eq!(super::theme(&state), nord);
    assert_eq!(nord.to_string(), "Nord");
    state.dispatch(Action::SetAppearance(Appearance::System));
    assert!(!super::theme(&state).extended_palette().is_dark);
}

#[test]
fn saving_appearance_does_not_change_the_visible_page() {
    let mut state = runtime();
    let mut renderer = Renderer::new(Font::default(), Pixels(16.0));
    let mut tree = Tree::new(super::view(&state).as_widget());
    let before = render_page(&state, &mut tree, &mut renderer);
    state.appearance.request_theme(state.theme_choice.clone());
    assert!(state.busy());
    let saving = render_page(&state, &mut tree, &mut renderer);
    let changed = before.iter().zip(&saving).filter(|(a, b)| a != b).count();
    assert_eq!(changed, 0);
}

#[test]
fn toggling_theme_preserves_status_messages_and_footer_visibility() {
    for status in [None, Some("Installation complete".to_string())] {
        let mut state = runtime();
        state.app.as_mut().unwrap().status_message = status.clone();
        let previous = super::theme(&state).extended_palette().is_dark;
        let _ = super::update(&mut state, Message::Action(Action::ToggleTheme));
        let app = state.app.as_ref().unwrap();
        assert_ne!(super::theme(&state).extended_palette().is_dark, previous);
        assert_eq!(app.status_message, status);
        assert!(state.appearance.busy());
    }
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
fn switch_cli_button_opens_the_picker_or_confirmation_from_either_focus_area() {
    for focus in [FocusArea::Tabs, FocusArea::Content] {
        for selected in [false, true] {
            let mut state = runtime();
            let app = state.app.as_mut().unwrap();
            app.focus = focus;
            for component in &mut app.components {
                component.selected = selected;
            }
            state.dispatch(Action::Back);
            assert_eq!(
                state.app.as_ref().unwrap().current_view,
                if selected {
                    View::ConfirmExit
                } else {
                    View::CliSelection
                },
                "focus={focus:?}, selected={selected}",
            );
            if selected {
                state.dispatch(Action::Cancel);
                let app = state.app.as_ref().unwrap();
                assert_eq!(app.current_view, View::List);
                assert!(app.components.iter().all(|component| component.selected));
            }
        }
    }
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
        let _ = super::update(&mut self.state, message);
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
        self.event(event, mouse::Cursor::Unavailable)
    }

    fn click_label(&mut self, label: &str) {
        struct Label<'a>(&'a str, Option<Rectangle>);
        impl Operation<Message> for Label<'_> {
            fn traverse(&mut self, visit: &mut dyn FnMut(&mut dyn Operation<Message>)) {
                visit(self);
            }

            fn text(&mut self, _: Option<&iced::widget::Id>, bounds: Rectangle, value: &str) {
                if value == self.0 {
                    self.1 = Some(bounds);
                }
            }
        }
        let mut label = Label(label, None);
        self.operate(&mut label);
        let cursor = mouse::Cursor::Available(label.1.expect("visible label").center());
        self.event(
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
            cursor,
        );
        self.event(
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
            cursor,
        );
    }

    fn event(&mut self, event: Event, cursor: mouse::Cursor) -> iced::event::Status {
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
            cursor,
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
    desktop.click_label("Config");
    assert_eq!(desktop.app().focus, FocusArea::Tabs);
    desktop.key(key::Named::ArrowDown);
    assert_eq!(desktop.app().tab, Tab::McpServers);
    desktop.click_label("Skills");
    desktop.focus_search();
    desktop.click_label("folder");
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

#[test]
fn home_panel_stays_centered_and_title_aligns_with_list_in_each_language() {
    #[derive(Default)]
    struct Bounds {
        panel: Option<Rectangle>,
        labels: Vec<(String, Rectangle)>,
    }
    impl Operation<Message> for Bounds {
        fn traverse(&mut self, visit: &mut dyn FnMut(&mut dyn Operation<Message>)) {
            visit(self);
        }
        fn container(&mut self, id: Option<&iced::widget::Id>, bounds: Rectangle) {
            if id == Some(&iced::widget::Id::new("home-panel")) {
                self.panel = Some(bounds);
            }
        }
        fn text(&mut self, _: Option<&iced::widget::Id>, bounds: Rectangle, value: &str) {
            self.labels.push((value.into(), bounds));
        }
    }
    let mut desktop = Desktop::new();
    desktop.state.app.as_mut().unwrap().current_view = View::CliSelection;
    for size in [Size::new(760.0, 560.0), Size::new(1120.0, 780.0)] {
        desktop.state.window_size = size;
        for (lang, title) in [
            (super::Language::Japanese, "ワークスペースを選択"),
            (super::Language::Korean, "작업 공간 선택"),
            (super::Language::English, "Choose a workspace"),
        ] {
            desktop.message(Message::Action(Action::SetLanguage(lang)));
            let mut bounds = Bounds::default();
            desktop.operate(&mut bounds);
            let panel = bounds.panel.unwrap();
            assert!(
                (panel.center_x() - size.width / 2.0).abs() <= 1.0,
                "{panel:?}"
            );
            assert!(
                (panel.center_y() - size.height / 2.0).abs() <= 1.0,
                "{panel:?}"
            );
            let (_, title_bounds) = bounds
                .labels
                .iter()
                .find(|(label, _)| label == title)
                .unwrap();
            assert!((title_bounds.x - panel.x).abs() <= 1.0);
        }
    }
}

#[test]
fn version_button_emits_check_and_update_confirmation_blocks_background_actions() {
    use super::updates::{Event as Update, State};
    let mut state = runtime();
    let area = Rectangle::with_size(state.window_size);
    let renderer = Renderer::new(Font::default(), Pixels(16.0));
    let mut element = super::view(&state);
    let widget = element.as_widget_mut();
    let mut tree = Tree::new(&*widget);
    let node = widget.layout(
        &mut tree,
        &renderer,
        &layout::Limits::new(Size::ZERO, area.size()),
    );
    let mut messages = Vec::new();
    for event in [
        mouse::Event::ButtonPressed(mouse::Button::Left),
        mouse::Event::ButtonReleased(mouse::Button::Left),
    ] {
        widget.update(
            &mut tree,
            &Event::Mouse(event),
            Layout::new(&node),
            mouse::Cursor::Available(Point::new(area.width - 45.0, area.height - 18.0)),
            &renderer,
            &mut clipboard::Null,
            &mut Shell::new(&mut messages),
            &area,
        );
    }
    assert!(matches!(
        messages.as_slice(),
        [Message::Update(Update::Check)]
    ));
    drop(element);
    state.updater = State::Checking;
    let offer = crate::fs::update::Offer {
        version: "2.0.0-beta.1".parse().unwrap(),
        route: crate::fs::update::Route::Pending,
    };
    let _ = super::update(
        &mut state,
        Message::Update(Update::Checked(Ok(Some(offer)))),
    );
    let _ = super::update(&mut state, Message::Action(Action::Key(KeyCode::Down)));
    let _ = super::update(&mut state, Message::Update(Update::Confirm));
    assert_eq!(state.app.as_ref().unwrap().tab, Tab::Skills);
    assert!(matches!(state.updater, State::Offer(_)));
    let _ = super::update(&mut state, Message::Action(Action::Key(KeyCode::Esc)));
    assert!(matches!(state.updater, State::Idle));
}

#[test]
fn home_right_arrow_moves_visible_focus_to_codex() {
    let mut desktop = Desktop::new();
    desktop.state.app.as_mut().unwrap().current_view = View::CliSelection;
    let before = render_page(&desktop.state, &mut desktop.tree, &mut desktop.renderer);
    desktop.key(key::Named::ArrowRight);
    assert_eq!(desktop.app().cli_choice, crate::app::CliChoice::Codex);
    let after = render_page(&desktop.state, &mut desktop.tree, &mut desktop.renderer);
    assert!(before != after, "the focused card must visibly change");
    use crate::app::CliChoice;
    for (arrow, target) in [
        (key::Named::ArrowUp, CliChoice::Claude),
        (key::Named::ArrowDown, CliChoice::Codex),
        (key::Named::ArrowDown, CliChoice::Sources),
        (key::Named::ArrowUp, CliChoice::Codex),
    ] {
        desktop.key(arrow);
        assert_eq!(desktop.app().cli_choice, target);
    }
}

#[test]
fn keyboard_can_open_filter_move_apply_and_dismiss_without_moving_between_panes() {
    let mut desktop = Desktop::new();
    desktop.message(Message::Action(Action::Key(KeyCode::Char('f'))));
    assert!(desktop.state.filter_choice.is_some());
    desktop.key(key::Named::Tab);
    assert_eq!(desktop.app().focus, FocusArea::Content);
    desktop.key(key::Named::Enter);
    assert!(
        desktop
            .app()
            .component_filter
            .contains(crate::app::ComponentFilter::Modified)
    );
    assert!(desktop.state.filter_choice.is_some());
    desktop.key(key::Named::Escape);
    assert!(desktop.state.filter_choice.is_none());
    assert_eq!(desktop.app().current_view, View::List);
}
