use iced::widget::{button, column, container, row, rule, text};
use iced::{Element, Fill};

use super::{Language, Message, actions::Action, fonts, styles};
use crate::app::{App, CliChoice, TargetCli};

pub fn view(lang: Language, app: &App) -> Element<'_, Message> {
    let choices = column![
        destination(
            "C",
            "Claude Code",
            lang.text("Agents, skills, hooks, MCP servers and plugins"),
            Action::SelectTarget(TargetCli::Claude),
            app.cli_choice == CliChoice::Claude
        ),
        rule::horizontal(1).style(styles::divider),
        destination(
            ">_",
            "Codex CLI",
            lang.text("Skills, shared configuration and MCP servers"),
            Action::SelectTarget(TargetCli::Codex),
            app.cli_choice == CliChoice::Codex
        ),
        rule::horizontal(1).style(styles::divider),
        destination(
            "+",
            lang.text("Manage sources"),
            &localized_format!(
                lang,
                "{} available · bundled configuration works offline",
                "{}개 사용 가능 · 기본 설정은 오프라인에서도 사용 가능",
                "{} 件利用可能 · 同梱設定はオフラインでも使用可能",
                app.sources.len()
            ),
            Action::OpenSources,
            app.cli_choice == CliChoice::Sources
        ),
    ]
    .spacing(4)
    .padding(8);
    let launcher = column![
        text(lang.text("Choose a workspace"))
            .font(fonts::display())
            .size(38),
        container(choices).width(Fill).style(styles::surface),
        text(lang.text("Tab or arrow keys to move · Enter to select"))
            .size(12)
            .style(styles::muted),
    ]
    .spacing(16);
    container(
        container(launcher)
            .width(Fill)
            .max_width(640)
            .id("home-panel"),
    )
    .center_x(Fill)
    .center_y(Fill)
    .into()
}

fn destination(
    mark: &'static str,
    name: &'static str,
    description: &str,
    action: Action,
    focused: bool,
) -> Element<'static, Message> {
    button(
        row![
            container(text(mark).font(fonts::monospace()).size(17))
                .center_x(40)
                .center_y(40)
                .style(styles::badge),
            column![
                text(name).size(19),
                text(description.to_owned()).size(12).style(styles::muted)
            ]
            .spacing(3)
            .width(Fill),
            text("›").size(22).style(styles::muted),
        ]
        .spacing(16)
        .align_y(iced::Center),
    )
    .padding([12, 16])
    .width(Fill)
    .style(if focused {
        styles::focused_row
    } else {
        styles::subtle
    })
    .on_press(Message::Action(action))
    .into()
}
