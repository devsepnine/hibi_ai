use crossterm::event::KeyCode;
use iced::widget::{Column, Space, button, column, container, row, scrollable, text};
use iced::{Element, Fill};

use super::source_forms::{
    confirm_remove, editable_selection, map_to, source_input, source_type, status_text, syncing,
};
use super::{Message, actions::Action, styles};
use crate::app::{App, View};
use crate::source::SourceEntry;

pub fn view(app: &App) -> Element<'_, Message> {
    let content = match app.current_view {
        View::Sources => source_list(app),
        View::SourceAddType => source_type(),
        View::SourceAddMapTo => map_to(app),
        View::SourceConfirmRemove => confirm_remove(app),
        View::SourceSyncing => syncing(app),
        _ => source_input(app),
    };
    column![
        column![
            text("Sources").size(28),
            text("Your component libraries, all in one place.")
                .size(14)
                .style(styles::muted)
        ]
        .spacing(6),
        content,
    ]
    .spacing(24)
    .height(Fill)
    .into()
}

fn key(value: char) -> Message {
    Message::Action(Action::Key(KeyCode::Char(value)))
}

fn source_list(app: &App) -> Element<'_, Message> {
    let mut entries = Column::new().spacing(6).width(Fill);
    entries = entries.push(source_row(
        0,
        "Bundled library".into(),
        "Included with HIBI AI · Read only".into(),
        app.source_list_index == 0,
        false,
    ));
    for (index, entry) in app.source_entries.iter().enumerate() {
        let (label, details) = entry_description(entry);
        let stale = match entry {
            SourceEntry::Git { url, .. } => {
                app.sources.iter().any(|s| s.label == *url && s.is_stale)
            }
            SourceEntry::Local { .. } => false,
        };
        entries = entries.push(source_row(
            index + 1,
            label,
            details,
            app.source_list_index == index + 1,
            stale,
        ));
    }
    let can_edit = editable_selection(app).is_some();
    let actions = row![
        button("Add source").on_press(key('a')).padding([10, 16]),
        button("Sync all")
            .on_press(key('f'))
            .style(styles::subtle)
            .padding([10, 16]),
        Space::new().width(Fill),
        text(format!("{} libraries", app.source_entries.len() + 1))
            .size(13)
            .style(styles::muted),
    ]
    .spacing(8)
    .align_y(iced::Center);
    let mut panel = column![
        row![
            text("Connected libraries").size(17),
            Space::new().width(Fill),
            text("Selected sources provide your components")
                .size(12)
                .style(styles::muted)
        ]
        .spacing(10),
        scrollable(entries).height(Fill),
        row![
            button("Edit source")
                .on_press_maybe(can_edit.then(|| key('e')))
                .style(styles::subtle)
                .padding([9, 14]),
            button("Remove source")
                .on_press_maybe(can_edit.then(|| key('r')))
                .style(styles::danger)
                .padding([9, 14]),
            Space::new().width(Fill),
            button("Back")
                .on_press(Message::Action(Action::Back))
                .style(styles::subtle)
                .padding([9, 14]),
        ]
        .spacing(8)
        .align_y(iced::Center),
    ]
    .spacing(18)
    .height(Fill);
    if let Some(status) = &app.source_sync_status {
        panel = panel.push(scrollable(status_text(status)).height(60));
    }
    column![
        actions,
        container(panel)
            .padding(22)
            .style(styles::surface)
            .height(Fill)
    ]
    .spacing(18)
    .height(Fill)
    .into()
}

fn source_row(
    index: usize,
    label: String,
    details: String,
    selected: bool,
    stale: bool,
) -> Element<'static, Message> {
    let mut description = column![
        text(label).size(15),
        text(details).size(12).style(styles::muted)
    ]
    .spacing(5);
    if stale {
        description = description.push(
            text("Cached version · Sync to retry")
                .size(12)
                .style(text::warning),
        );
    }
    button(
        row![
            container(text(if index == 0 { "H" } else { "+" }).size(18))
                .padding(12)
                .style(styles::surface),
            description.width(Fill),
            text(if selected { "Selected" } else { "" })
                .size(11)
                .style(styles::muted)
        ]
        .spacing(14)
        .align_y(iced::Center),
    )
    .on_press(Message::Action(Action::SelectSource(index)))
    .style(if selected {
        styles::navigation
    } else {
        styles::subtle
    })
    .padding(14)
    .width(Fill)
    .into()
}

fn entry_description(entry: &SourceEntry) -> (String, String) {
    let (label, kind, branch, root, map_to) = match entry {
        SourceEntry::Git {
            url,
            branch,
            root,
            map_to,
        } => (url.clone(), "Git", branch.as_deref(), root, map_to),
        SourceEntry::Local { path, root, map_to } => (
            path.to_string_lossy().into_owned(),
            "Local",
            None,
            root,
            map_to,
        ),
    };
    let mut details = vec![kind.to_string()];
    if let Some(branch) = branch {
        details.push(format!("Branch: {branch}"));
    }
    if let Some(root) = root {
        details.push(format!("Root: {root}"));
    }
    if let Some(map_to) = map_to {
        details.push(format!("Map to: {map_to}"));
    }
    (label, details.join(" · "))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{SyncStatus, test_support::fresh_app};

    #[test]
    fn bundled_and_stale_selection_cannot_remove_a_user_source() {
        let mut app = fresh_app();
        app.source_entries.push(SourceEntry::Local {
            path: "/tmp/설정".into(),
            root: None,
            map_to: None,
        });
        assert!(editable_selection(&app).is_none());
        app.source_list_index = usize::MAX;
        assert!(editable_selection(&app).is_none());
        app.current_view = View::SourceConfirmRemove;
        let _ = view(&app);
    }

    #[test]
    fn wizard_renders_empty_inputs_and_multiline_failure_in_every_step() {
        let mut app = fresh_app();
        app.source_edit_index = Some(usize::MAX);
        app.source_input_error = Some("Cannot save configuration\nPermission denied".into());
        app.source_sync_status = Some(SyncStatus::Error("Offline\nUsing stale cache".into()));
        for state in [
            View::Sources,
            View::SourceAddType,
            View::SourceAddUrl,
            View::SourceAddBranch,
            View::SourceAddPath,
            View::SourceAddRoot,
            View::SourceAddMapTo,
            View::SourceConfirmRemove,
            View::SourceSyncing,
        ] {
            app.current_view = state;
            let _ = view(&app);
        }
    }
}
