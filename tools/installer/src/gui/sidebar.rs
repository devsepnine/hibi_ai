use crossterm::event::KeyCode;
use iced::widget::{Space, button, column, container, row, rule, scrollable, text};
use iced::{Element, Fill};

use super::{Message, actions::Action, key_button, styles};
use crate::app::{App, FocusArea, Tab};

pub fn view(app: &App) -> Element<'_, Message> {
    let environment = app
        .target_cli
        .as_ref()
        .map(|target| target.display_name())
        .unwrap_or("Workspace");
    let mut navigation = column![].spacing(3);
    for tab in &app.available_tabs {
        let count = count(app, *tab);
        navigation = navigation.push(
            button(
                row![
                    text(tab.display_name()).size(14),
                    Space::new().width(Fill),
                    text(count).size(12)
                ]
                .align_y(iced::Center),
            )
            .width(Fill)
            .padding([9, 12])
            .style(if *tab == app.tab {
                styles::navigation
            } else {
                styles::subtle
            })
            .on_press(Message::Action(Action::SelectTab(*tab))),
        );
    }
    container(
        column![
            text("WORKSPACE").size(10).style(styles::muted),
            text(environment).size(19),
            button(text("Switch environment").size(12))
                .style(styles::subtle)
                .on_press(Message::Action(Action::Back)),
            rule::horizontal(1),
            text("COMPONENTS · 1")
                .size(10)
                .style(if app.focus == FocusArea::Tabs {
                    text::primary
                } else {
                    styles::muted
                }),
            scrollable(navigation).id("component-sidebar").height(Fill),
            rule::horizontal(1),
            key_button("Keyboard shortcuts", KeyCode::Char('?'))
                .style(styles::subtle)
                .width(Fill),
            text(format!("{} sources connected", app.sources.len()))
                .size(11)
                .style(styles::muted),
        ]
        .spacing(12)
        .height(Fill),
    )
    .padding(16)
    .width(194)
    .height(Fill)
    .style(styles::sidebar)
    .into()
}

fn count(app: &App, tab: Tab) -> usize {
    match tab {
        Tab::McpServers => app.mcp_servers.len(),
        Tab::Plugins => app.plugins.len(),
        _ => app
            .components
            .iter()
            .filter(|item| tab.to_component_type().as_ref() == Some(&item.component_type))
            .count(),
    }
}
