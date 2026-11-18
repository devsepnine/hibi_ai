use super::{Language, Message, Runtime, styles};
use crate::fs::update::{self, Offer, Route};
use iced::{
    Element, Fill, Task,
    widget::{button, column, container, row, text},
};
use std::time::{Duration, Instant};

#[derive(Debug, Default)]
pub(super) enum State {
    #[default]
    Idle,
    Checking,
    Current(Instant),
    Offer(Offer),
    Installing,
    Failed(&'static str, String),
}

impl State {
    pub fn modal(&self) -> bool {
        matches!(self, Self::Offer(_) | Self::Installing | Self::Failed(..))
    }
}

#[derive(Clone, Debug)]
pub enum Event {
    Check,
    Checked(Result<Option<Offer>, String>),
    Confirm,
    Finished(Result<(), String>),
    Dismiss,
}

fn work<T: Send + 'static>(
    f: impl FnOnce() -> anyhow::Result<T> + Send + 'static,
    map: fn(Result<T, String>) -> Event,
) -> Task<Message> {
    let (tx, rx) = iced::futures::channel::oneshot::channel();
    std::thread::spawn(move || {
        let _ = tx.send(f().map_err(|e| format!("{e:#}")));
    });
    Task::perform(
        async move {
            rx.await
                .unwrap_or_else(|_| Err("Update worker stopped".into()))
        },
        move |result| Message::Update(map(result)),
    )
}

pub(super) fn handle(state: &mut Runtime, event: Event) -> Task<Message> {
    if state.closing || state.close_requested || state.remove_requested {
        if matches!(event, Event::Checked(_)) {
            state.updater = State::Idle;
        }
        return Task::none();
    }
    match event {
        Event::Check if matches!(state.updater, State::Idle | State::Current(_)) => {
            state.updater = State::Checking;
            return work(update::check, Event::Checked);
        }
        Event::Checked(result) if matches!(state.updater, State::Checking) => {
            state.updater = match result {
                Ok(Some(offer)) => State::Offer(offer),
                Ok(None) => State::Current(Instant::now()),
                Err(error) => State::Failed("Could not check for updates", error),
            };
        }
        Event::Confirm if !state.busy() => {
            if let State::Offer(offer) = &state.updater {
                if matches!(offer.route, Route::Manual) {
                    state.updater = match update::open_release(&offer.version) {
                        Ok(()) => State::Idle,
                        Err(e) => State::Failed("Could not open release page", format!("{e:#}")),
                    };
                } else if !matches!(offer.route, Route::Pending) {
                    let offer = offer.clone();
                    state.updater = State::Installing;
                    return work(move || update::apply(offer), Event::Finished);
                }
            }
        }
        Event::Finished(result) if matches!(state.updater, State::Installing) => match result {
            Ok(()) => return iced::exit(),
            Err(error) => state.updater = State::Failed("Update failed", error),
        },
        Event::Dismiss if !matches!(state.updater, State::Installing) => {
            state.updater = State::Idle
        }
        _ => {}
    }
    Task::none()
}

pub(super) fn footer(state: &Runtime) -> Element<'_, Message> {
    let lang = state.language;
    let label = match state.updater {
        State::Checking => lang.text("Checking for updates…").into(),
        State::Current(at) if at.elapsed() < Duration::from_secs(4) => {
            lang.text("Up to date").into()
        }
        _ => crate::fs::VERSION.to_owned(),
    };
    button(text(label).size(11))
        .style(styles::subtle)
        .on_press_maybe(
            (!state.closing
                && !state.close_requested
                && !state.remove_requested
                && matches!(state.updater, State::Idle | State::Current(_)))
            .then_some(Message::Update(Event::Check)),
        )
        .into()
}

pub(super) fn dialog(state: &Runtime) -> Element<'_, Message> {
    let lang: Language = state.language;
    let mut body = column![].spacing(14);
    match &state.updater {
        State::Offer(offer) => {
            body = body.push(text(format!("{} → v{}", crate::fs::VERSION, offer.version)).size(24));
            let (detail, action) = match &offer.route {
                Route::Pending => ("Waiting for the package update", None),
                Route::Manual => (
                    "Download the package for your installation.",
                    Some("Open release page"),
                ),
                _ => ("hibi will restart after updating.", Some("Update")),
            };
            body = body.push(text(lang.text(if state.busy() {
                "Wait for the current operation to finish."
            } else {
                detail
            })));
            let mut buttons =
                row![button(lang.text("Cancel")).on_press(Message::Update(Event::Dismiss))]
                    .spacing(10);
            if let Some(action) = action {
                buttons = buttons.push(
                    button(lang.text(action))
                        .style(styles::primary)
                        .on_press_maybe((!state.busy()).then_some(Message::Update(Event::Confirm))),
                );
            }
            body = body.push(buttons);
        }
        State::Installing => body = body.push(text(lang.text("Updating hibi…"))),
        State::Failed(title, error) => {
            body = body
                .push(text(lang.text(title)).size(22))
                .push(text(error))
                .push(button(lang.text("Dismiss")).on_press(Message::Update(Event::Dismiss)))
        }
        _ => {}
    }
    container(container(body).padding(24).style(styles::surface))
        .center_x(Fill)
        .center_y(Fill)
        .into()
}
