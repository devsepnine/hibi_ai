mod actions;
mod activity;
mod dialogs;
mod filters;
mod fonts;
mod home;
#[cfg(test)]
mod keyboard_tests;
mod list;
mod preferences;
mod rows;
mod runtime;
mod shortcuts;
mod sidebar;
mod source_forms;
mod sources;
mod styles;
mod text_input;
#[cfg(test)]
#[path = "visibility/tests.rs"]
mod visibility_tests;

use crossterm::event::KeyCode;
use iced::widget::{Space, button, column, container, row, scrollable, svg, text, tooltip};
use iced::{Element, Fill, Task, Theme};

use crate::app::{App, View};
use crate::theme::ThemeMode;
use actions::Action;
use runtime::Runtime;
use shortcuts::{scroll_after_key, subscription};

#[derive(Debug, Clone)]
pub enum Message {
    Action(Action),
    Tick,
    Resized(iced::Size),
    FocusSearch,
    FocusResults,
    SearchEscape,
    OperationLogScrolled(scrollable::Viewport),
}

pub fn run() -> iced::Result {
    iced::application(Runtime::new, update, view)
        .default_font(fonts::interface())
        .title("HIBI AI")
        .theme(theme)
        .subscription(subscription)
        .window(iced::window::Settings {
            size: iced::Size::new(1120.0, 780.0),
            min_size: Some(iced::Size::new(760.0, 560.0)),
            exit_on_close_request: false,
            ..Default::default()
        })
        .centered()
        .run()
}

fn update(state: &mut Runtime, message: Message) -> Task<Message> {
    let focus_navigation = shortcuts::navigation_focus(&message);
    let scroll = match &message {
        Message::Action(Action::Key(key)) => Some(*key),
        Message::Action(
            Action::SelectTab(_)
            | Action::SetFilter(..)
            | Action::SetHiddenVisibility(_)
            | Action::SetSearch(_)
            | Action::ClearSearch,
        ) => Some(KeyCode::Home),
        _ => None,
    };
    match message {
        Message::Action(action) => state.dispatch(action),
        Message::Tick => state.tick(),
        Message::Resized(size) => state.window_size = size,
        Message::OperationLogScrolled(viewport) => {
            if state.app.as_ref().is_some_and(|app| {
                matches!(app.current_view, View::Preflighting | View::Installing)
            }) && let Some(operation) = state.operation_log.update(viewport)
            {
                return iced::advanced::widget::operate(operation);
            }
        }
        Message::FocusSearch if search_available(state) => {
            return iced::widget::operation::focus("component-search");
        }
        Message::FocusResults if search_available(state) => {
            if let Some(app) = state.app.as_mut() {
                app.focus_content();
            }
            return iced::widget::operation::focus("component-list");
        }
        Message::SearchEscape if search_available(state) => {
            state.dispatch(Action::ClearSearch);
            if let Some(app) = state.app.as_mut() {
                app.focus_content();
            }
        }
        Message::FocusSearch | Message::FocusResults | Message::SearchEscape => {}
    }
    if state.should_close {
        return iced::exit();
    }
    if !state
        .app
        .as_ref()
        .is_some_and(|app| matches!(app.current_view, View::Preflighting | View::Installing))
    {
        state.operation_log = Default::default();
    }
    let scroll = match (scroll, state.app.as_ref()) {
        (Some(key), Some(app)) => scroll_after_key(app, key),
        _ => Task::none(),
    };
    let focus = if search_available(state) {
        focus_navigation.map_or_else(Task::none, iced::advanced::widget::operate)
    } else {
        Task::none()
    };
    Task::batch([focus, scroll])
}

fn search_available(state: &Runtime) -> bool {
    !state.close_requested
        && !state.remove_requested
        && state
            .app
            .as_ref()
            .is_some_and(|app| app.current_view == View::List)
}

fn theme(state: &Runtime) -> Theme {
    styles::theme(!matches!(
        state.app.as_ref().map(|app| app.theme.mode()),
        Some(ThemeMode::Latte)
    ))
}

fn view(state: &Runtime) -> Element<'_, Message> {
    let compact = state.window_size.height < 680.0;
    let padding = if compact { 14.0 } else { 20.0 };
    let mut page = column![
        header(state),
        iced::widget::rule::horizontal(1).style(styles::divider),
    ]
    .spacing(if compact { 10 } else { 16 })
    .padding(padding);
    if state.close_requested {
        page = page.push(dialogs::close(state.busy()));
    } else if state.remove_requested {
        page = page.push(dialogs::remove(state.remove_count));
    } else if let Some(app) = &state.app {
        page = page.push(content(
            app,
            compact,
            state.filter_menu_open,
            state.operation_log.following,
        ));
    } else if state.error.is_none() {
        page = page.push(container(text("Loading sources and configuration…")).center_y(Fill));
    }
    if let Some(error) = &state.error {
        page = page.push(
            container(
                column![
                    text(error).style(text::danger),
                    button(if state.app.is_some() {
                        "Dismiss"
                    } else {
                        "Retry"
                    })
                    .on_press(Message::Action(if state.app.is_some() {
                        Action::DismissError
                    } else {
                        Action::Retry
                    })),
                ]
                .spacing(8),
            )
            .padding(12)
            .style(container::rounded_box),
        );
    }
    if let Some(message) = state
        .app
        .as_ref()
        .filter(|app| !(app.current_view == View::Installing && app.processing_complete))
        .and_then(|app| app.status_message.as_ref())
    {
        page = page
            .push(container(scrollable(text(message).size(11).style(styles::muted))).max_height(30))
            .padding(iced::Padding::new(padding).bottom(8));
    }
    // Include the background in damage tracking when presentation buffers are reused.
    container(page.height(Fill))
        .width(Fill)
        .height(Fill)
        .style(|theme: &Theme| container::Style::default().background(theme.palette().background))
        .into()
}

fn header(state: &Runtime) -> Element<'_, Message> {
    let dark = !matches!(
        state.app.as_ref().map(|app| app.theme.mode()),
        Some(ThemeMode::Latte)
    );
    let theme_icon: &[u8] = if dark {
        br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><circle cx="12" cy="12" r="4" fill="none" stroke="black" stroke-width="1.7"/><path d="M12 2v2m0 16v2M2 12h2m16 0h2M5 5l1.5 1.5m11 11L19 19M5 19l1.5-1.5m11-11L19 5" fill="none" stroke="black" stroke-width="1.7" stroke-linecap="round"/></svg>"#
    } else {
        br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M20.5 13A8.5 8.5 0 0 1 11 3.5 8.5 8.5 0 1 0 20.5 13Z" fill="none" stroke="black" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round"/></svg>"#
    };
    row![
        container(text("h").size(23))
            .width(36)
            .height(36)
            .center_x(36)
            .center_y(36)
            .style(styles::badge),
        text("hibi").size(20),
        text("Configuration manager").size(11).style(styles::muted),
        Space::new().width(Fill),
        text(if state.operation_active() {
            "Working…".to_string()
        } else {
            crate::fs::VERSION.to_string()
        })
        .size(11)
        .style(styles::muted),
        header_button(
            theme_icon,
            if dark { "Light theme" } else { "Dark theme" },
            state.app.is_some().then_some(Action::ToggleTheme),
        ),
    ]
    .spacing(12)
    .align_y(iced::Center)
    .into()
}

fn header_button(
    icon: &'static [u8],
    label: &'static str,
    action: Option<Action>,
) -> Element<'static, Message> {
    let enabled = action.is_some();
    let icon = svg(svg::Handle::from_memory(icon))
        .width(18)
        .height(18)
        .style(move |theme: &Theme, _| svg::Style {
            color: Some(
                theme
                    .palette()
                    .text
                    .scale_alpha(if enabled { 1.0 } else { 0.55 }),
            ),
        });
    tooltip(
        button(icon)
            .padding(9)
            .width(36)
            .height(36)
            .style(styles::subtle)
            .on_press_maybe(action.map(Message::Action)),
        text(label).size(12),
        tooltip::Position::Bottom,
    )
    .gap(6)
    .style(styles::surface)
    .into()
}

fn content(
    app: &App,
    compact: bool,
    filter_menu_open: bool,
    follow_log: bool,
) -> Element<'_, Message> {
    match app.current_view {
        View::CliSelection => home::view(app),
        View::List => list::view(app, compact, filter_menu_open),
        View::Sources
        | View::SourceAddType
        | View::SourceAddUrl
        | View::SourceAddBranch
        | View::SourceAddPath
        | View::SourceAddRoot
        | View::SourceAddMapTo
        | View::SourceConfirmRemove
        | View::SourceSyncing => sources::view(app),
        _ => dialogs::view(app, follow_log),
    }
}

pub(super) fn key_button(label: &str, key: KeyCode) -> iced::widget::Button<'_, Message> {
    button(text(label).size(12))
        .padding([7, 10])
        .style(styles::primary)
        .on_press(Message::Action(Action::Command(key)))
}
