use super::Language;
use iced::widget::{Space, button, column, container, row, text};
use iced::{Element, Fill, Font, font};

use super::{Message, actions::Action, styles};
use crate::app::{App, CliChoice, TargetCli};

pub fn view(lang: Language, app: &App) -> Element<'_, Message> {
    let intro = column![
        text(lang.text("Your workspace, configured."))
            .size(38)
            .font(Font {
                weight: font::Weight::Semibold,
                ..super::fonts::interface()
            })
            .align_x(iced::Center),
        text(lang.text("Manage the instructions, tools and workflows your AI assistants use."))
            .size(14)
            .align_x(iced::Center)
            .style(styles::muted),
    ]
    .spacing(10)
    .align_x(iced::Center)
    .width(Fill);
    let choices = row![
        target(
            "C",
            "Claude Code",
            lang.text("Agents, skills, hooks, MCP servers and plugins"),
            TargetCli::Claude,
            app.cli_choice == CliChoice::Claude,
        ),
        target(
            ">_",
            "Codex CLI",
            lang.text("Skills, shared configuration and MCP servers"),
            TargetCli::Codex,
            app.cli_choice == CliChoice::Codex,
        ),
    ]
    .spacing(14);
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
                .style(if app.cli_choice == CliChoice::Sources {
                    styles::focused_secondary
                } else {
                    styles::secondary
                })
                .on_press(Message::Action(Action::OpenSources)),
        ]
        .align_y(iced::Center)
        .spacing(12),
    )
    .padding(16)
    .style(styles::surface);
    container(
        container(
            column![
                intro,
                choices,
                sources,
                text(lang.text("Tab or arrow keys to move · Enter to select"))
                    .size(12)
                    .style(styles::muted)
            ]
            .spacing(22)
            .align_x(iced::Center),
        )
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
    focused: bool,
) -> Element<'a, Message> {
    button(
        column![
            row![
                container(text(icon).size(19).style(text::primary))
                    .center_x(38)
                    .center_y(38)
                    .style(styles::badge),
                Space::new().width(Fill),
                text("›").size(24).style(styles::muted),
            ]
            .align_y(iced::Center),
            column![
                text(name).size(19),
                text(description).size(12).style(styles::muted),
            ]
            .spacing(5),
        ]
        .spacing(16)
        .width(Fill),
    )
    .padding(20)
    .width(Fill)
    .style(if focused {
        styles::focused_glass_button
    } else {
        styles::glass_button
    })
    .on_press(Message::Action(Action::SelectTarget(target)))
    .into()
}
