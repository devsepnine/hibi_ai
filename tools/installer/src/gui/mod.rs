mod actions;
mod activity;
mod dialogs;
mod home;
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

use crossterm::event::KeyCode;
use iced::widget::{Space, button, column, container, row, scrollable, text};
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
}

pub fn run() -> iced::Result {
    iced::application(Runtime::new, update, view)
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
    let scroll = match &message {
        Message::Action(Action::Key(key)) => Some(*key),
        Message::Action(
            Action::SelectTab(_)
            | Action::SetFilter(_)
            | Action::SetSearch(_)
            | Action::ClearSearch,
        ) => Some(KeyCode::Home),
        _ => None,
    };
    match message {
        Message::Action(action) => state.dispatch(action),
        Message::Tick => state.tick(),
        Message::Resized(size) => state.window_size = size,
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
        iced::exit()
    } else if let (Some(key), Some(app)) = (scroll, state.app.as_ref()) {
        scroll_after_key(app, key)
    } else {
        Task::none()
    }
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
    let mut page = column![header(state), iced::widget::rule::horizontal(1)]
        .spacing(if compact { 10 } else { 16 })
        .padding(if compact { 14 } else { 20 });
    if state.close_requested {
        page = page.push(dialogs::close(state.busy()));
    } else if state.remove_requested {
        page = page.push(dialogs::remove(state.remove_count));
    } else if let Some(app) = &state.app {
        page = page.push(content(app, compact));
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
        .and_then(|app| app.status_message.as_ref())
    {
        page = page.push(scrollable(text(message).size(11).style(styles::muted)).height(30));
    }
    container(page.height(Fill)).width(Fill).height(Fill).into()
}

fn header(state: &Runtime) -> Element<'_, Message> {
    let dark = !matches!(
        state.app.as_ref().map(|app| app.theme.mode()),
        Some(ThemeMode::Latte)
    );
    row![
        container(text("h").size(23))
            .width(36)
            .height(36)
            .center_x(36)
            .center_y(36)
            .style(styles::badge),
        text("hibi").size(23),
        text(format!("/  Configuration manager · {}", crate::fs::VERSION))
            .size(12)
            .style(styles::muted),
        Space::new().width(Fill),
        text(if state.busy() {
            "Working…"
        } else {
            "Local workspace"
        })
        .size(11)
        .style(styles::muted),
        button(text(if dark { "Light theme" } else { "Dark theme" }).size(12))
            .padding([8, 12])
            .style(styles::subtle)
            .on_press_maybe(
                state
                    .app
                    .is_some()
                    .then_some(Message::Action(Action::ToggleTheme))
            ),
        button(text("Close").size(12))
            .style(styles::subtle)
            .on_press(Message::Action(Action::RequestClose)),
    ]
    .spacing(12)
    .align_y(iced::Center)
    .into()
}

fn content(app: &App, compact: bool) -> Element<'_, Message> {
    match app.current_view {
        View::CliSelection => home::view(app),
        View::List => list::view(app, compact),
        View::Sources
        | View::SourceAddType
        | View::SourceAddUrl
        | View::SourceAddBranch
        | View::SourceAddPath
        | View::SourceAddRoot
        | View::SourceAddMapTo
        | View::SourceConfirmRemove
        | View::SourceSyncing => sources::view(app),
        _ => dialogs::view(app),
    }
}

pub(super) fn key_button(label: &str, key: KeyCode) -> iced::widget::Button<'_, Message> {
    button(text(label).size(12))
        .padding([7, 10])
        .on_press(Message::Action(Action::Command(key)))
}
