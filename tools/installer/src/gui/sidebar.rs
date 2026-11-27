use super::Language;
use crossterm::event::KeyCode;
use iced::widget::{Space, button, column, container, row, rule, scrollable, svg, text};
use iced::{Element, Fill};

use super::{Message, actions::Action, key_button, styles};
use crate::app::{App, Tab, TargetCli};

pub fn view(lang: Language, app: &App, focused: bool) -> Element<'_, Message> {
    let mut navigation = column![].spacing(4);
    for tab in &app.available_tabs {
        let count = count(app, *tab);
        navigation = navigation.push(
            button(
                row![
                    category_icon(*tab),
                    text(lang.text(tab.display_name())).size(13),
                    Space::new().width(Fill),
                    text(count).size(11).style(styles::muted)
                ]
                .spacing(10)
                .align_y(iced::Center),
            )
            .width(Fill)
            .padding([8, 10])
            .style(match (*tab == app.tab, focused) {
                (true, true) => styles::focused_row,
                (true, _) => styles::navigation,
                _ => styles::subtle,
            })
            .on_press(Message::Action(Action::SelectTab(*tab))),
        );
    }
    container(
        column![
            text(lang.text("Workspace")).size(11).style(styles::muted),
            workspace(lang, app),
            rule::horizontal(1).style(styles::divider),
            text(lang.text("Components")).size(11).style(if focused {
                text::primary
            } else {
                styles::muted
            }),
            scrollable(navigation)
                .id("component-sidebar")
                .direction(scrollable::Direction::Vertical(
                    scrollable::Scrollbar::new().spacing(6),
                ))
                .height(Fill),
            rule::horizontal(1).style(styles::divider),
            key_button(lang.text("Keyboard shortcuts"), KeyCode::Char('?'))
                .style(styles::subtle)
                .width(Fill),
            text(match lang {
                Language::English => format!(
                    "{} source{}",
                    app.sources.len(),
                    if app.sources.len() == 1 { "" } else { "s" }
                ),
                Language::Korean => format!("소스 {}개", app.sources.len()),
                Language::Japanese => format!("ソース {} 件", app.sources.len()),
            })
            .size(11)
            .style(styles::muted),
        ]
        .spacing(10)
        .height(Fill),
    )
    .padding(16)
    .width(208)
    .height(Fill)
    .style(styles::glass)
    .into()
}

fn workspace(lang: Language, app: &App) -> Element<'_, Message> {
    let environment = app
        .target_cli
        .as_ref()
        .map(|target| target.display_name())
        .unwrap_or(lang.text("Workspace"));
    let mark = if app.target_cli == Some(TargetCli::Claude) {
        "C"
    } else {
        ">_"
    };
    button(
        row![
            container(text(mark).size(14))
                .width(30)
                .height(30)
                .center_x(30)
                .center_y(30)
                .style(styles::badge),
            column![
                text(environment).size(14),
                text(lang.text("Switch CLI")).size(10).style(styles::muted),
            ]
            .spacing(3)
            .width(Fill),
            text("›").size(18).style(styles::muted),
        ]
        .spacing(10)
        .align_y(iced::Center),
    )
    .padding(10)
    .width(Fill)
    .style(styles::secondary)
    .on_press(Message::Action(Action::Back))
    .into()
}

fn category_icon(tab: Tab) -> iced::widget::Svg<'static> {
    let shape = match tab {
        Tab::Agents => r#"<circle cx="10" cy="6" r="3"/><path d="M4 18v-2a6 6 0 0 1 12 0v2"/>"#,
        Tab::Commands => r#"<path d="M4 5l5 5-5 5m7 0h5"/>"#,
        Tab::Contexts => {
            r#"<rect x="3" y="5" width="12" height="12" rx="2"/><path d="M7 2h10v11"/>"#
        }
        Tab::Rules => r#"<path d="M10 2l7 3v5c0 4-7 8-7 8S3 14 3 10V5zM7 10l2 2 4-4"/>"#,
        Tab::Skills => {
            r#"<rect x="3" y="3" width="5" height="5" rx="1"/><rect x="12" y="3" width="5" height="5" rx="1"/><rect x="3" y="12" width="5" height="5" rx="1"/><rect x="12" y="12" width="5" height="5" rx="1"/>"#
        }
        Tab::Hooks => r#"<path d="M7 3v9a5 5 0 0 0 10 0V9m-3 3 3-3 2 3M4 3h6"/>"#,
        Tab::OutputStyles => r#"<path d="M5 3h10M10 3v14M7 17h6M3 7V3m14 4V3"/>"#,
        Tab::Statusline => r#"<path d="M2 10h4l2-6 4 12 2-6h4"/>"#,
        Tab::Config => {
            r#"<path d="M3 5h14M3 10h14M3 15h14"/><circle cx="7" cy="5" r="2"/><circle cx="13" cy="10" r="2"/><circle cx="8" cy="15" r="2"/>"#
        }
        Tab::McpServers => r#"<path d="M7 2v4m6-4v4M5 6h10v3a5 5 0 0 1-10 0zm5 8v4"/>"#,
        Tab::Plugins => r#"<path d="M3 8h4V5a3 3 0 0 1 6 0v3h4v9H3z"/>"#,
    };
    let image = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="none" stroke="black" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round">{shape}</svg>"#
    );
    svg(svg::Handle::from_memory(image.into_bytes()))
        .width(16)
        .height(16)
        .style(|theme: &iced::Theme, _| svg::Style {
            color: styles::muted(theme).color,
        })
}

fn count(app: &App, tab: Tab) -> usize {
    match tab {
        Tab::McpServers => app.mcp_servers.len(),
        Tab::Plugins => app.plugins.len(),
        _ => app
            .components
            .iter()
            .filter(|item| {
                tab.to_component_type().as_ref() == Some(&item.component_type)
                    && app.component_filter.matches(&item.status)
            })
            .count(),
    }
}
