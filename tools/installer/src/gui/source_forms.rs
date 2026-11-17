use crossterm::event::KeyCode;
use iced::widget::{Space, button, column, container, row, scrollable, text, text_input};
use iced::{Element, Fill};

use super::{Message, actions::Action, styles};
use crate::app::{App, SyncStatus, View};
use crate::source::SourceEntry;

fn key(value: char) -> Message {
    Message::Action(Action::Key(KeyCode::Char(value)))
}

fn cancel<'a>() -> iced::widget::Button<'a, Message> {
    button("Cancel")
        .padding([10, 16])
        .style(styles::subtle)
        .on_press(Message::Action(Action::Cancel))
}

fn card<'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    container(
        container(scrollable(
            container(content)
                .padding(28)
                .width(Fill)
                .style(styles::surface),
        ))
        .max_width(620)
        .width(Fill),
    )
    .center_x(Fill)
    .center_y(Fill)
    .into()
}

pub(super) fn source_type() -> Element<'static, Message> {
    card(
        column![
            text("ADD SOURCE  /  STEP 1").size(11).style(styles::muted),
            text("Where are your components?").size(24),
            text("Connect a repository or a folder on this computer.").style(styles::muted),
            button(
                column![
                    text("Git repository").size(16),
                    text("Sync components from an HTTPS repository")
                        .size(13)
                        .style(styles::muted)
                ]
                .spacing(5)
            )
            .width(Fill)
            .padding(18)
            .style(styles::subtle)
            .on_press(key('1')),
            button(
                column![
                    text("Local folder").size(16),
                    text("Use an existing component directory")
                        .size(13)
                        .style(styles::muted)
                ]
                .spacing(5)
            )
            .width(Fill)
            .padding(18)
            .style(styles::subtle)
            .on_press(key('2')),
            cancel(),
        ]
        .spacing(18),
    )
}

pub(super) fn syncing(app: &App) -> Element<'_, Message> {
    let mut content = column![
        text("SOURCE SYNC").size(11).style(styles::muted),
        text("Getting everything up to date").size(24),
        text("Resolving your libraries and refreshing the component inventory.")
            .style(styles::muted),
        text("Changes will appear when synchronization finishes.")
            .size(13)
            .style(styles::muted),
    ]
    .spacing(18);
    if let Some(status) = &app.source_sync_status {
        content = content.push(status_text(status));
    }
    card(content.push(cancel()))
}

pub(super) fn status_text(status: &SyncStatus) -> iced::widget::Text<'_> {
    match status {
        SyncStatus::Success(message) => text(message).size(13).style(text::success),
        SyncStatus::Error(message) => text(message).size(13).style(text::warning),
    }
}

pub(super) fn editable_selection(app: &App) -> Option<&SourceEntry> {
    app.source_list_index
        .checked_sub(1)
        .and_then(|index| app.source_entries.get(index))
}

pub(super) fn confirm_remove(app: &App) -> Element<'_, Message> {
    let entry = editable_selection(app);
    let label = entry.map(|entry| match entry {
        SourceEntry::Git { url, .. } => url.clone(),
        SourceEntry::Local { path, .. } => path.to_string_lossy().into_owned(),
    });
    card(column![
        text("REMOVE SOURCE").size(11).style(styles::muted),
        text("Disconnect this library?").size(24),
        container(text(label.unwrap_or_else(|| "This source is no longer available.".to_string())).size(14)).padding(16).style(styles::surface),
        text("This removes the library from your sources and clears its Git cache. Your installed components stay in place.").style(styles::muted),
        row![cancel(), Space::new().width(Fill), button("Remove source").on_press_maybe(entry.map(|_| key('y'))).style(button::danger).padding([10,16])].spacing(10),
    ].spacing(18))
}

pub(super) fn source_input(app: &App) -> Element<'_, Message> {
    let (title, label, hint) = match app.current_view {
        View::SourceAddUrl => (
            "Connect a Git repository",
            "https://github.com/owner/repository.git",
            "Use an HTTPS URL without credentials.",
        ),
        View::SourceAddBranch => (
            "Choose a branch",
            "Default branch",
            "Leave empty to follow the repository's default branch.",
        ),
        View::SourceAddPath => (
            "Choose a local folder",
            "/path/to/components",
            "Enter a component folder. You can use ~ for your home directory.",
        ),
        View::SourceAddRoot => (
            "Set the source root",
            "Repository root",
            "Optional: a subdirectory such as config/claude.",
        ),
        _ => ("Source", "", ""),
    };
    let step = step_label(app);
    let mut content = column![
        text(format!(
            "{}  /  {step}",
            if app.source_edit_index.is_some() {
                "EDIT SOURCE"
            } else {
                "ADD SOURCE"
            }
        ))
        .size(11)
        .style(styles::muted),
        text(title).size(24),
        text(hint).style(styles::muted),
        text_input(label, &app.source_input_buffer)
            .on_input(|value| Message::Action(Action::SetInput(value)))
            .on_submit(Message::Action(Action::Key(KeyCode::Enter)))
            .padding(14)
            .size(15),
    ]
    .spacing(18);
    if let Some(error) = &app.source_input_error {
        content = content.push(text(error).size(13).style(text::danger));
    }
    card(
        content.push(
            row![
                cancel(),
                Space::new().width(Fill),
                button("Continue")
                    .on_press(Message::Action(Action::Key(KeyCode::Enter)))
                    .padding([10, 20])
            ]
            .spacing(12),
        ),
    )
}

fn step_label(app: &App) -> String {
    let git = app.source_add_kind == Some(crate::source::SourceKind::Git);
    let (step, total) = match app.current_view {
        View::SourceAddUrl => (2, 5),
        View::SourceAddBranch => (3, 5),
        View::SourceAddPath => (2, 4),
        View::SourceAddRoot if git => (4, 5),
        View::SourceAddRoot => (3, 4),
        View::SourceAddMapTo if git => (5, 5),
        _ => (4, 4),
    };
    let skipped = usize::from(app.source_edit_index.is_some());
    format!("STEP {} OF {}", step - skipped, total - skipped)
}

pub(super) fn map_to(app: &App) -> Element<'_, Message> {
    let step = step_label(app);
    let mut choices = column![
        text(format!("SOURCE MAPPING  /  {step}"))
            .size(11)
            .style(styles::muted),
        text("Organize your components").size(24),
        text("Map every file to one component type, or keep the source's folder structure.")
            .style(styles::muted),
    ]
    .spacing(16);
    if let Some(index) = app.source_edit_index
        && let Some(
            SourceEntry::Git {
                map_to: Some(value),
                ..
            }
            | SourceEntry::Local {
                map_to: Some(value),
                ..
            },
        ) = app.source_entries.get(index)
    {
        choices = choices.push(
            text(format!("Current mapping: {value}"))
                .size(13)
                .style(styles::muted),
        );
    }
    let options = [
        ('1', "Agents"),
        ('2', "Commands"),
        ('3', "Contexts"),
        ('4', "Rules"),
        ('5', "Skills"),
        ('6', "Hooks"),
        ('7', "Output styles"),
    ];
    for chunk in options.chunks(2) {
        let mut options = row![].spacing(8);
        for &(shortcut, label) in chunk {
            options = options.push(
                button(label)
                    .on_press(key(shortcut))
                    .width(Fill)
                    .style(styles::subtle)
                    .padding(12),
            );
        }
        choices = choices.push(options);
    }
    choices = choices.push(
        button("Keep folder structure")
            .on_press(Message::Action(Action::Key(KeyCode::Enter)))
            .width(Fill)
            .padding(12),
    );
    choices = choices.push(cancel());
    if let Some(error) = &app.source_input_error {
        choices = choices.push(text(error).size(13).style(text::danger));
    }
    card(choices)
}
