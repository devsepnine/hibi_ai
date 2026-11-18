use super::Language;
use iced::widget::{Space, button, column, container, row, text};
use iced::{Element, Fill};

use super::{Message, actions::Action, styles};
use crate::app::{App, TargetCli};

pub fn view(lang: Language, app: &App) -> Element<'_, Message> {
    let intro = column![
        text(lang.text("Your AI workspace"))
            .size(11)
            .style(styles::muted),
        text(lang.text("Your workspace, configured.")).size(28),
        text(lang.text("Manage the instructions, tools and workflows your AI assistants use."))
            .size(13)
            .align_x(iced::Center)
            .style(styles::muted),
    ]
    .spacing(10)
    .align_x(iced::Center)
    .width(Fill);
    let choices = column![
        target(
            "C",
            "Claude Code",
            lang.text("Agents, skills, hooks, MCP servers and plugins"),
            TargetCli::Claude
        ),
        target(
            ">",
            "Codex CLI",
            lang.text("Skills, shared configuration and MCP servers"),
            TargetCli::Codex
        ),
    ]
    .spacing(12);
    let sources = container(
        row![
            column![
                text(lang.text("Connected sources")).size(14),
                text(localized_format!(
                    lang,
                    "{} available · bundled configuration works offline",
                    "{}개 사용 가능 · 기본 설정은 오프라인에서도 사용 가능",
                    "{} 件利用可能 · 同梱設定はオフラインでも使用可能",
                    app.sources.len()
                ))
                .size(12)
                .style(styles::muted),
            ]
            .spacing(5),
            Space::new().width(Fill),
            button(lang.text("Manage sources"))
                .padding([9, 12])
                .style(styles::secondary)
                .on_press(Message::Action(Action::OpenSources)),
        ]
        .align_y(iced::Center)
        .spacing(12),
    )
    .padding(16)
    .style(styles::sidebar);
    container(
        container(column![intro, choices, sources].spacing(24))
            .width(Fill)
            .max_width(680)
            .id("home-panel"),
    )
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
