use super::Language;
use crossterm::event::KeyCode;
use iced::widget::{Column, Space, button, column, container, row, scrollable, text, text_input};
use iced::{Element, Fill};

use super::{
    Message,
    actions::Action,
    activity::{diff, progress},
    key_button, styles,
};
use crate::app::{App, View};

pub fn view(lang: Language, app: &App, follow_log: bool) -> Element<'_, Message> {
    match app.current_view {
        View::Loading => card(
            column![
                text(lang.text("Finding your components")).size(24),
                text(lang.text("Scanning connected libraries and checking installed configuration."))
                    .style(styles::muted),
                text(lang.text("This may take a moment."))
                    .size(13)
                    .style(styles::muted),
            ]
            .spacing(18),
        ),
        View::Preflighting | View::Installing => progress(lang, app, follow_log),
        View::Diff => diff(lang, app),
        View::EnvInput => {
            let server = app.current_env_server_name().unwrap_or(lang.text("MCP server"));
            let label = localized_format!(lang, "{} · {} of {}", "{0} · {2}개 중 {1}번째", "{0} · 全{2}件中{1}件目",
                app.current_env_var().unwrap_or(lang.text("Value")),
                app.env_input_current
                    .saturating_add(1)
                    .min(app.env_input_vars.len()),
                app.env_input_vars.len()
            );
            form(lang,
                &localized_format!(lang, "Configure {server}", "{server} 설정", "{server} の設定"),
                app.current_setup_input().map_or(
                    lang.text("Enter the required value to continue. It is hidden while you type."),
                    |setup| lang.text(setup.hint()),
                ),
                &label,
                text_input(lang.text("Enter value"), &app.env_input_buffer)
                    .style(styles::input)
                    .secure(app.current_setup_input().is_none())
                    .on_input(|value| Message::Action(Action::SetEnvInput(value)))
                    .on_submit(Message::Action(Action::Key(KeyCode::Enter)))
                    .padding(14)
                    .size(15)
                    .into(),
            )
        }
        View::ProjectPath => form(lang,
            lang.text("Choose a project directory"),
            lang.text("The MCP server will be available in this project's local configuration."),
            lang.text("Project directory"),
            text_input("/path/to/project", &app.project_path_buffer)
                .style(styles::input)
                .on_input(|value| Message::Action(Action::SetProjectPath(value)))
                .on_submit(Message::Action(Action::Key(KeyCode::Enter)))
                .padding(14)
                .size(15)
                .into(),
        ),
        View::ConfirmExit => confirm(lang,
            lang.text("Leave this environment?"),
            lang.text("Your pending selections in every category will be cleared. Installed components stay in place."),
            lang.text("Discard selections"),
        ),
        View::ConfirmRemove => remove(lang, app),
        View::ConfirmHide => {
            let mut description = String::from(
                lang.text("These folders will be hidden from the component list and excluded from install and remove actions. Their files stay in place."),
            );
            if let Some(pending) = &app.hide_confirmation {
                description.push_str("\n\n");
                description.push_str(&pending.folders.join("\n"));
                description.push_str(&localized_format!(lang, "\n\n{} selected files will be unchecked.", "\n\n선택한 파일 {}개의 선택을 해제합니다.", "\n\n選択した {} 件のファイルの選択を解除します。",
                    pending.selected_files
                ));
            }
            confirm(lang, lang.text("Hide folders?"), &description, lang.text("Hide folders"))
        }
        View::Help => help(lang),
        _ => text(lang.text("Select an environment to get started."))
            .style(styles::muted)
            .into(),
    }
}

fn card<'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    container(
        container(scrollable(container(content).padding(30).width(Fill)))
            .max_width(620)
            .width(Fill)
            .style(styles::glass),
    )
    .center_x(Fill)
    .center_y(Fill)
    .into()
}

pub(super) fn neutral<'a>(label: &'a str, action: Action) -> iced::widget::Button<'a, Message> {
    button(label)
        .style(styles::secondary)
        .padding([10, 16])
        .on_press(Message::Action(action))
}

fn form<'a>(
    lang: Language,
    title: &str,
    hint: &str,
    label: &str,
    input: Element<'a, Message>,
) -> Element<'a, Message> {
    card(
        column![
            text(title.to_owned()).size(24),
            text(hint.to_owned()).style(styles::muted),
            column![text(label.to_owned()).size(13), input].spacing(8),
            row![
                neutral(lang.text("Cancel"), Action::Cancel),
                Space::new().width(Fill),
                key_button(lang.text("Continue"), KeyCode::Enter).padding([10, 20])
            ]
            .spacing(12),
        ]
        .spacing(20),
    )
}

fn confirm(
    lang: Language,
    title: &str,
    description: &str,
    label: &str,
) -> Element<'static, Message> {
    card(
        column![
            text(title.to_owned()).size(24),
            text(description.to_owned()).size(14).style(styles::muted),
            row![
                neutral(lang.text("Keep working"), Action::Cancel),
                Space::new().width(Fill),
                button(text(label.to_owned()))
                    .style(styles::primary)
                    .padding([10, 16])
                    .on_press(Message::Action(Action::Key(KeyCode::Char('y'))))
            ]
            .spacing(12),
        ]
        .spacing(20),
    )
}

pub fn close(lang: Language, busy: bool) -> Element<'static, Message> {
    card(
        column![
            text(if busy {
                lang.text("Stop the current operation?")
            } else {
                lang.text("Ready to close?")
            })
            .size(24),
            text(if busy {
                lang.text("The window closes when the active operation stops.")
            } else {
                lang.text("Pending selections will be discarded.")
            })
            .style(styles::muted),
            row![
                neutral(lang.text("Keep working"), Action::DismissClose),
                Space::new().width(Fill),
                button(lang.text("Close application"))
                    .style(styles::destructive)
                    .padding([10, 16])
                    .on_press(Message::Action(Action::ConfirmClose))
            ]
            .spacing(12),
        ]
        .spacing(20),
    )
}

fn remove(lang: Language, app: &App) -> Element<'_, Message> {
    use crate::component::InstallStatus;

    let items = app.pending_removal_items();
    let count = items.len();
    let question = if count == 1 {
        lang.text("Remove 1 selected item?").to_owned()
    } else {
        localized_format!(
            lang,
            "Remove {count} selected items?",
            "선택한 항목 {count}개를 제거할까요?",
            "選択した {count} 件を削除しますか？"
        )
    };
    let names = items.iter().fold(column![].spacing(8), |names, item| {
        let mut line = row![text(item.display_name()).size(14).width(Fill)].spacing(10);
        if let Some(status @ (InstallStatus::Modified | InstallStatus::External)) = &item.status {
            line = line.push(
                text(lang.text(status.display()))
                    .size(12)
                    .style(text::danger),
            );
        }
        names.push(line)
    });
    let panel = column![
        text(question).size(24),
        text(lang.text("This action cannot be undone."))
            .size(13)
            .style(text::danger),
        container(scrollable(names).id("remove-items").height(iced::Shrink))
            .padding(12)
            .max_height(188)
            .style(styles::surface),
        row![
            neutral(lang.text("Cancel"), Action::CancelRemove),
            Space::new().width(Fill),
            button(lang.text("Remove items"))
                .style(styles::destructive)
                .padding([10, 16])
                .on_press_maybe((count > 0).then_some(Message::Action(Action::ConfirmRemove)))
        ]
        .spacing(12),
        text(lang.text("Y to remove · Esc to cancel"))
            .size(12)
            .style(styles::muted),
    ]
    .spacing(12);
    container(
        container(panel)
            .padding(24)
            .width(Fill)
            .max_width(620)
            .max_height(390)
            .style(styles::glass),
    )
    .center_x(Fill)
    .center_y(Fill)
    .into()
}

fn help(lang: Language) -> Element<'static, Message> {
    let groups = [
        (
            "Search",
            &[
                (
                    if cfg!(target_os = "macos") {
                        "Cmd+F"
                    } else {
                        "Ctrl+F"
                    },
                    "Focus category search",
                ),
                (
                    "Ctrl+A / Cmd+A",
                    "Select all text in the focused search field",
                ),
                ("Enter", "Return from search to the component list"),
                ("Esc", "Clear the current search before going back"),
            ][..],
        ),
        (
            "Selection",
            &[
                ("Space", "Select or clear the focused item"),
                ("a", "Select or clear shown items"),
            ][..],
        ),
        (
            "Browse & review",
            &[
                ("Up / Down", "Move through the focused categories or list"),
                ("Left / Right", "Collapse or expand folders in the list"),
                ("Right / Enter", "Enter the list from categories"),
                ("1 / 2", "Focus categories or component list"),
                (
                    "Tab / Shift+Tab",
                    "Switch between categories and component list",
                ),
                ("d / Enter", "View changes or open a folder"),
                ("f", "Open or close filters"),
                ("↑/↓ · Enter", "Move within filters, then apply a choice"),
                ("E", "Expand or collapse all folders"),
            ][..],
        ),
        (
            "Manage components",
            &[
                ("i", "Install selected items"),
                ("r", "Remove selected items"),
                ("s / u", "Set or unset a default style or statusline"),
                ("o", "Change MCP scope"),
            ][..],
        ),
        (
            "Folder visibility",
            &[
                ("x", "Hide or unhide a folder"),
                ("X", "Hide selected folders"),
                ("H", "Show or conceal hidden folders"),
            ][..],
        ),
        (
            "Application",
            &[
                ("t", "Switch light and dark themes"),
                ("?", "Open this reference"),
                ("Esc", "Go back or cancel"),
                ("q", "Close the application"),
            ][..],
        ),
    ];

    let mut contents = Column::new().spacing(24);
    for (heading, shortcuts) in groups {
        let mut section = column![text(lang.text(heading)).size(16)].spacing(10);
        for &(key, description) in shortcuts {
            section = section.push(
                row![
                    container(text(key).size(12).font(iced::Font::MONOSPACE))
                        .padding([5, 9])
                        .width(110)
                        .style(styles::surface),
                    text(lang.text(description)).size(14).width(Fill)
                ]
                .spacing(16)
                .align_y(iced::Center),
            );
        }
        contents = contents.push(section);
    }
    column![
        row![text(lang.text("Keyboard shortcuts")).size(28), Space::new().width(Fill), neutral(lang.text("Back"), Action::Back)].spacing(12).align_y(iced::Center),
        text(lang.text("Click to focus a row. Use its checkbox to select it. Input fields support selection and paste; shortcuts pause while typing.")).size(14).style(styles::muted),
        container(scrollable(contents).id("help").height(Fill)).padding(24).style(styles::surface).height(Fill),
    ].spacing(18).height(Fill).into()
}
