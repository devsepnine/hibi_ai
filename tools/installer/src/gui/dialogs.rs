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

pub fn view(app: &App, follow_log: bool) -> Element<'_, Message> {
    match app.current_view {
        View::Loading => card(
            column![
                text("PREPARING YOUR ENVIRONMENT")
                    .size(11)
                    .style(styles::muted),
                text("Finding your components").size(24),
                text("Scanning connected libraries and checking installed configuration.")
                    .style(styles::muted),
                text("This may take a moment.")
                    .size(13)
                    .style(styles::muted),
            ]
            .spacing(18),
        ),
        View::Preflighting | View::Installing => progress(app, follow_log),
        View::Diff => diff(app),
        View::EnvInput => {
            let server = app.current_env_server_name().unwrap_or("MCP server");
            let step = format!(
                "{}  /  VALUE {} OF {}",
                server,
                app.env_input_current
                    .saturating_add(1)
                    .min(app.env_input_vars.len()),
                app.env_input_vars.len()
            );
            form(
                "Configure your MCP server",
                &step,
                app.current_setup_input().map_or(
                    "Enter the required value to continue. It is hidden while you type.",
                    |setup| setup.hint(),
                ),
                app.current_env_var().unwrap_or("Value"),
                text_input("Enter value", &app.env_input_buffer)
                    .style(styles::input)
                    .secure(app.current_setup_input().is_none())
                    .on_input(|value| Message::Action(Action::SetEnvInput(value)))
                    .on_submit(Message::Action(Action::Key(KeyCode::Enter)))
                    .padding(14)
                    .size(15)
                    .into(),
            )
        }
        View::ProjectPath => form(
            "Choose a project directory",
            "MCP  /  LOCAL SCOPE",
            "The MCP server will be available in this project's local configuration.",
            "Project directory",
            text_input("/path/to/project", &app.project_path_buffer)
                .style(styles::input)
                .on_input(|value| Message::Action(Action::SetProjectPath(value)))
                .on_submit(Message::Action(Action::Key(KeyCode::Enter)))
                .padding(14)
                .size(15)
                .into(),
        ),
        View::ConfirmExit => confirm(
            "Leave this environment?",
            "Your pending selections in every category will be cleared. Installed components stay in place.",
            "Discard selections",
        ),
        View::ConfirmHide => {
            let mut description = String::from(
                "These folders will be hidden from the component list and excluded from install and remove actions. Their files stay in place.\n\n",
            );
            if let Some(pending) = &app.hide_confirmation {
                description.push_str(&pending.folders.join("\n"));
                description.push_str(&format!(
                    "\n\n{} selected files will be unchecked.",
                    pending.selected_files
                ));
            }
            confirm("Hide folders?", &description, "Hide folders")
        }
        View::Help => help(),
        _ => text("Select an environment to get started.")
            .style(styles::muted)
            .into(),
    }
}

fn card<'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    container(
        container(scrollable(
            container(content)
                .padding(30)
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

pub(super) fn neutral<'a>(label: &'a str, action: Action) -> iced::widget::Button<'a, Message> {
    button(label)
        .style(styles::secondary)
        .padding([10, 16])
        .on_press(Message::Action(action))
}

fn form<'a>(
    title: &str,
    step: &str,
    hint: &str,
    label: &str,
    input: Element<'a, Message>,
) -> Element<'a, Message> {
    card(
        column![
            text(step.to_owned()).size(11).style(styles::muted),
            text(title.to_owned()).size(24),
            text(hint.to_owned()).style(styles::muted),
            column![text(label.to_owned()).size(13), input].spacing(8),
            row![
                neutral("Cancel", Action::Cancel),
                Space::new().width(Fill),
                key_button("Continue", KeyCode::Enter).padding([10, 20])
            ]
            .spacing(12),
        ]
        .spacing(20),
    )
}

fn confirm(title: &str, description: &str, label: &str) -> Element<'static, Message> {
    card(
        column![
            text("CONFIRM ACTION").size(11).style(styles::muted),
            text(title.to_owned()).size(24),
            text(description.to_owned()).size(14).style(styles::muted),
            row![
                neutral("Keep working", Action::Cancel),
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

pub fn close(busy: bool) -> Element<'static, Message> {
    card(
        column![
            text("CLOSE HIBI AI").size(11).style(styles::muted),
            text(if busy {
                "Stop the current operation?"
            } else {
                "Ready to close?"
            })
            .size(24),
            text(if busy {
                "The window closes when the active operation stops."
            } else {
                "Pending selections will be discarded."
            })
            .style(styles::muted),
            row![
                neutral("Keep working", Action::DismissClose),
                Space::new().width(Fill),
                button("Close application")
                    .style(styles::destructive)
                    .padding([10, 16])
                    .on_press(Message::Action(Action::ConfirmClose))
            ]
            .spacing(12),
        ]
        .spacing(20),
    )
}

pub fn remove(count: usize) -> Element<'static, Message> {
    card(column![
        text("REMOVE COMPONENTS").size(11).style(styles::muted),
        text(format!("Remove {count} selected items?")).size(24),
        text("This removes the selected installed items in the current category from this environment, including selected external files.").style(styles::muted),
        text("This action cannot be undone.").size(13).style(text::danger),
        row![neutral("Cancel", Action::CancelRemove), Space::new().width(Fill), button("Remove items").style(styles::destructive).padding([10,16]).on_press(Message::Action(Action::ConfirmRemove))].spacing(12),
    ].spacing(20))
}

fn help() -> Element<'static, Message> {
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
                ("f", "Cycle the status filter"),
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
        let mut section = column![text(heading).size(16)].spacing(10);
        for &(key, description) in shortcuts {
            section = section.push(
                row![
                    container(text(key).size(12).font(iced::Font::MONOSPACE))
                        .padding([5, 9])
                        .width(110)
                        .style(styles::surface),
                    text(description).size(14).width(Fill)
                ]
                .spacing(16)
                .align_y(iced::Center),
            );
        }
        contents = contents.push(section);
    }
    column![
        row![text("Keyboard shortcuts").size(28), Space::new().width(Fill), neutral("Back", Action::Back)].spacing(12).align_y(iced::Center),
        text("Click to focus a row. Use its checkbox to select it. Input fields support selection and paste; shortcuts pause while typing.").size(14).style(styles::muted),
        container(scrollable(contents).id("help").height(Fill)).padding(24).style(styles::surface).height(Fill),
    ].spacing(18).height(Fill).into()
}
