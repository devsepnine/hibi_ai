use iced::widget::{Space, button, column, container, row, text};
use iced::{Element, Fill};

use super::{Message, actions::Action, styles};
use crate::app::{App, TargetCli};

pub fn view(app: &App) -> Element<'_, Message> {
    let intro = column![
        text("Your AI workspace").size(11).style(styles::muted),
        text("Your workspace, configured.").size(28),
        text("Manage the instructions, tools and workflows your AI assistants use.")
            .size(13)
            .style(styles::muted),
    ]
    .spacing(10);
    let choices = column![
        target(
            "C",
            "Claude Code",
            "Agents, skills, hooks, MCP servers and plugins",
            TargetCli::Claude
        ),
        target(
            ">",
            "Codex CLI",
            "Skills, shared configuration and MCP servers",
            TargetCli::Codex
        ),
    ]
    .spacing(12);
    let sources = container(
        row![
            column![
                text("Connected sources").size(14),
                text(format!(
                    "{} available · bundled configuration works offline",
                    app.sources.len()
                ))
                .size(12)
                .style(styles::muted),
            ]
            .spacing(5),
            Space::new().width(Fill),
            button("Manage sources")
                .padding([9, 12])
                .style(styles::secondary)
                .on_press(Message::Action(Action::OpenSources)),
        ]
        .align_y(iced::Center)
        .spacing(12),
    )
    .padding(16)
    .style(styles::sidebar);
    container(container(column![intro, choices, sources].spacing(24)).max_width(680))
        .center_x(Fill)
        .center_y(Fill)
        .into()
}

fn target<'a>(
    icon: &'a str,
    name: &'a str,
    description: &'a str,
    target: TargetCli,
) -> Element<'a, Message> {
    button(
        row![
            container(text(icon).size(20))
                .width(40)
                .height(40)
                .center_x(40)
                .center_y(40)
                .style(styles::badge),
            column![
                text(name).size(17),
                text(description).size(12).style(styles::muted)
            ]
            .spacing(6)
            .width(Fill),
            text("›").size(24).style(styles::muted),
        ]
        .spacing(16)
        .align_y(iced::Center),
    )
    .padding(18)
    .width(Fill)
    .style(|theme, status| {
        let mut style = styles::subtle(theme, status);
        let surface = styles::surface(theme);
        style.border = surface.border;
        if matches!(status, button::Status::Active | button::Status::Disabled) {
            style.background = surface.background;
        }
        style
    })
    .on_press(Message::Action(Action::SelectTarget(target)))
    .into()
}
