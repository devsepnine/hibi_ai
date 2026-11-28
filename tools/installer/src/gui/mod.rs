#[macro_use]
mod locale;
mod actions;
mod appearance;
use crate::fs::installer::preferences::Language;
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
mod updates;
#[cfg(test)]
#[path = "visibility/tests.rs"]
mod visibility_tests;
mod window_bar;

use crossterm::event::KeyCode;
use iced::widget::{button, column, container, row, scrollable, svg, text, tooltip};
use iced::{Element, Fill, Task, Theme};

use crate::app::{App, View};
use actions::Action;
use runtime::Runtime;
use shortcuts::{scroll_after_key, subscription};

#[derive(Debug, Clone)]
pub enum Message {
    Action(Action),
    Update(updates::Event),
    Window(window_bar::Command),
    #[cfg(target_os = "windows")]
    WindowMoved(iced::Point),
    #[cfg(target_os = "windows")]
    TouchWindowDrag(iced::Vector),
    Tick,
    SystemTheme(iced::theme::Mode),
    Resized(iced::Size),
    SearchSubmitted,
    SearchEscape,
    OperationLogScrolled(scrollable::Viewport),
}

pub fn run() -> iced::Result {
    #[cfg(target_os = "macos")]
    set_application_icon();
    iced::application(
        || {
            (
                Runtime::new(),
                iced::system::theme().map(Message::SystemTheme),
            )
        },
        update,
        view,
    )
    .default_font(fonts::interface())
    .title("HIBI AI")
    .theme(|state: &Runtime| appearance::window_theme(&state.theme_choice, state.accent))
    .subscription(subscription)
    .window(window_bar::settings())
    .centered()
    .run()
}

#[cfg(target_os = "macos")]
fn set_application_icon() {
    use objc2_app_kit::{NSApplication, NSImage, NSImageNameApplicationIcon};
    use objc2_foundation::{MainThreadMarker, NSData};

    let main = MainThreadMarker::new().expect("GUI startup runs on the main thread");
    let data = NSData::with_bytes(include_bytes!("../../assets/hibi.png"));
    let icon = NSImage::initWithData(main.alloc(), &data).expect("valid Hibi application icon");
    // The standard About panel looks up the named image even in an unbundled process.
    // SAFETY: AppKit is accessed on the main thread and retains the registered image.
    unsafe {
        NSApplication::sharedApplication(main).setApplicationIconImage(Some(&icon));
        if let Some(previous) = NSImage::imageNamed(NSImageNameApplicationIcon) {
            previous.setName(None);
        }
        icon.setName(Some(NSImageNameApplicationIcon));
    }
}

fn update(state: &mut Runtime, message: Message) -> Task<Message> {
    if state.updater.modal() {
        match &message {
            Message::Action(Action::Key(KeyCode::Esc) | Action::RequestClose)
            | Message::SearchEscape => return updates::handle(state, updates::Event::Dismiss),
            Message::Action(_) | Message::SearchSubmitted => {
                return Task::none();
            }
            _ => {}
        }
    }
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
        Message::Update(event) => return updates::handle(state, event),
        Message::Window(command) => return command.run(),
        #[cfg(target_os = "windows")]
        Message::WindowMoved(point) => state.window_position = Some(point),
        #[cfg(target_os = "windows")]
        Message::TouchWindowDrag(delta) => {
            if let Some(position) = state.window_position {
                return window_bar::Command::TouchMove(position + delta).run();
            }
        }
        Message::Action(action) => state.dispatch(action),
        Message::Tick => state.tick(),
        Message::SystemTheme(mode) => state.system_mode = mode,
        Message::Resized(size) => state.window_size = size,
        Message::OperationLogScrolled(viewport) => {
            if state.app.as_ref().is_some_and(|app| {
                matches!(app.current_view, View::Preflighting | View::Installing)
            }) && let Some(operation) = state.operation_log.update(viewport)
            {
                return iced::advanced::widget::operate(operation);
            }
        }
        Message::SearchSubmitted if search_available(state) && state.filter_choice.is_none() => {
            if let Some(app) = state.app.as_mut() {
                app.focus_content();
            }
        }
        Message::SearchEscape if search_available(state) && state.filter_choice.is_none() => {
            state.dispatch(Action::ClearSearch);
            if let Some(app) = state.app.as_mut() {
                app.focus_content();
            }
        }
        Message::SearchSubmitted | Message::SearchEscape => {}
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
    match (scroll, state.app.as_ref()) {
        (Some(key), Some(app)) if state.filter_choice.is_none() => scroll_after_key(app, key),
        _ => Task::none(),
    }
}

fn search_available(state: &Runtime) -> bool {
    !state.close_requested
        && state
            .app
            .as_ref()
            .is_some_and(|app| app.current_view == View::List)
}

fn theme(state: &Runtime) -> Theme {
    appearance::resolve(&state.theme_choice, state.system_mode, state.accent)
}

fn view(state: &Runtime) -> Element<'_, Message> {
    let lang = state.language;
    let compact = state.window_size.height < 680.0;
    let padding = if compact { 14.0 } else { 20.0 };
    let spacing = if compact { 10 } else { 16 };
    let mut page = column![].spacing(spacing).padding([0, padding as u16]);
    if state.updater.modal() {
        page = page.push(updates::dialog(state));
    } else if state.close_requested {
        page = page.push(dialogs::close(lang, state.busy()));
    } else if let Some(app) = &state.app {
        page = page.push(content(
            lang,
            app,
            compact,
            state.filter_choice,
            state.operation_log.following,
        ));
    } else if state.error.is_none() {
        page = page
            .push(container(text(lang.text("Loading sources and configuration…"))).center_y(Fill));
    }
    if let Some(error) = &state.error {
        page = page.push(
            container(
                column![
                    text(lang.notice(error)).style(text::danger),
                    button(if state.app.is_some() {
                        lang.text("Dismiss")
                    } else {
                        lang.text("Retry")
                    })
                    .style(styles::secondary)
                    .on_press(Message::Action(if state.app.is_some() {
                        Action::DismissError
                    } else {
                        Action::Retry
                    })),
                ]
                .spacing(8),
            )
            .padding(12)
            .style(styles::surface),
        );
    }
    let message = state
        .app
        .as_ref()
        .filter(|app| !(app.current_view == View::Installing && app.processing_complete))
        .and_then(|app| app.status_message.as_deref());
    let mut footer = row![].spacing(12).align_y(iced::Center);
    if let Some(message) = message {
        footer = footer.push(
            container(scrollable(
                text(lang.notice(message)).size(11).style(styles::muted),
            ))
            .max_height(30)
            .width(Fill),
        );
    }
    footer = footer.push(
        iced::widget::pick_list(
            [Language::English, Language::Korean, Language::Japanese],
            Some(lang),
            |language| Message::Action(Action::SetLanguage(language)),
        )
        .text_size(12)
        .padding([4, 10])
        .style(styles::selector)
        .menu_style(styles::selector_menu),
    );
    let version = updates::footer(state);
    let footer = container(footer.push(version))
        .padding([0, padding as u16])
        .align_right(Fill)
        .center_y(window_bar::HEIGHT);
    let page = column![window_bar::view(state), page.height(Fill), footer].spacing(spacing);
    #[cfg(not(target_os = "macos"))]
    let page = window_bar::resizable(page.height(Fill).into());
    // Include the background in damage tracking when presentation buffers are reused.
    iced::widget::themer(
        Some(theme(state)),
        container(page)
            .width(Fill)
            .height(Fill)
            .style(styles::backdrop),
    )
    .text_color(|theme| theme.palette().text)
    .into()
}

fn theme_button(state: &Runtime) -> Element<'static, Message> {
    const DIAMETER: f32 = 36.0;
    let lang = state.language;
    let dark = theme(state).extended_palette().is_dark;
    let theme_icon: &[u8] = if dark {
        br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><circle cx="12" cy="12" r="4" fill="none" stroke="black" stroke-width="1.7"/><path d="M12 2v2m0 16v2M2 12h2m16 0h2M5 5l1.5 1.5m11 11L19 19M5 19l1.5-1.5m11-11L19 5" fill="none" stroke="black" stroke-width="1.7" stroke-linecap="round"/></svg>"#
    } else {
        br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M20.5 13A8.5 8.5 0 0 1 11 3.5 8.5 8.5 0 1 0 20.5 13Z" fill="none" stroke="black" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round"/></svg>"#
    };
    let enabled = state.app.is_some();
    let icon = svg(svg::Handle::from_memory(theme_icon))
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
            .width(DIAMETER)
            .height(DIAMETER)
            .style(|theme, status| {
                let mut style = styles::glass_button(theme, status);
                style.border.radius = (DIAMETER / 2.0).into();
                style
            })
            .on_press_maybe(enabled.then_some(Message::Action(Action::ToggleTheme))),
        text(if dark {
            lang.text("Light theme")
        } else {
            lang.text("Dark theme")
        })
        .size(12),
        tooltip::Position::Bottom,
    )
    .gap(6)
    .style(styles::popup)
    .into()
}

fn content(
    lang: Language,
    app: &App,
    compact: bool,
    filter_choice: Option<filters::Choice>,
    follow_log: bool,
) -> Element<'_, Message> {
    match app.current_view {
        View::CliSelection => home::view(lang, app),
        View::List => list::view(lang, app, compact, filter_choice),
        View::Sources
        | View::SourceAddType
        | View::SourceAddUrl
        | View::SourceAddBranch
        | View::SourceAddPath
        | View::SourceAddRoot
        | View::SourceAddMapTo
        | View::SourceConfirmRemove
        | View::SourceSyncing => sources::view(lang, app),
        _ => dialogs::view(lang, app, follow_log),
    }
}

pub(super) fn key_button(label: &str, key: KeyCode) -> iced::widget::Button<'_, Message> {
    button(text(label).size(12))
        .padding([7, 10])
        .style(styles::primary)
        .on_press(Message::Action(Action::Command(key)))
}
