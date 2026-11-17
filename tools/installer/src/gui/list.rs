use crossterm::event::KeyCode;
use iced::widget::{Space, button, column, container, row, rule, scrollable, text, text_input};
use iced::{Element, Fill};

use super::{Message, actions::Action, key_button, rows, sidebar, styles};
use crate::app::{App, ComponentFilter, Tab};

pub fn view(app: &App, compact: bool) -> Element<'_, Message> {
    let (selected, eligible, total) = selection_summary(app);
    let expand = app.get_tree_view().is_some_and(|tree| {
        tree.nodes
            .iter()
            .any(|node| node.is_folder() && !node.is_expanded())
    });
    let mut heading =
        column![text(app.tab.display_name()).size(if compact { 23 } else { 27 })].spacing(5);
    if !compact {
        heading = heading.push(text(description(app.tab)).size(13).style(styles::muted));
    }
    let mut main = column![
        row![
            heading,
            Space::new().width(Fill),
            container(
                text(format!(
                    "{total} {}",
                    if app.search_query.trim().is_empty() {
                        "items"
                    } else {
                        "results"
                    }
                ))
                .size(12)
            )
            .padding([5, 9])
            .style(styles::badge),
        ]
        .align_y(iced::Center),
    ]
    .spacing(if compact { 8 } else { 16 });
    if app.tab.to_component_type().is_some() {
        let filters = [
            ComponentFilter::All,
            ComponentFilter::Modified,
            ComponentFilter::External,
            ComponentFilter::New,
        ]
        .into_iter()
        .fold(row![].spacing(4), |row, filter| {
            row.push(
                button(text(filter.display()).size(12))
                    .padding([7, 12])
                    .style(if app.component_filter == filter {
                        styles::navigation
                    } else {
                        styles::subtle
                    })
                    .on_press(Message::Action(Action::SetFilter(filter))),
            )
        });
        main = main
            .push(row![search(app), filters].spacing(8).align_y(iced::Center))
            .push(
                row![
                    key_button(
                        if expand { "Expand all" } else { "Collapse all" },
                        KeyCode::Char('E')
                    )
                    .style(styles::subtle),
                    key_button("Hide / unhide", KeyCode::Char('x')).style(styles::subtle),
                    key_button("Hide checked", KeyCode::Char('X')).style(styles::subtle),
                    key_button(
                        if app.show_hidden {
                            "Conceal hidden"
                        } else {
                            "Show hidden"
                        },
                        KeyCode::Char('H')
                    )
                    .style(styles::subtle),
                ]
                .spacing(3)
                .wrap(),
            );
    } else {
        main = main.push(search(app));
    }
    if app.tab == Tab::McpServers {
        main = main.push(
            container(
                row![
                    column![
                        text(format!(
                            "Installation scope: {}",
                            if app.target_cli == Some(crate::app::TargetCli::Codex) {
                                "user"
                            } else {
                                app.mcp_scope.display()
                            }
                        ))
                        .size(13),
                        text(&app.mcp_project_path).size(11).style(styles::muted)
                    ]
                    .spacing(3),
                    Space::new().width(Fill),
                    key_button("Change scope", KeyCode::Char('o'))
                        .style(styles::subtle)
                        .on_press_maybe(
                            (app.target_cli != Some(crate::app::TargetCli::Codex))
                                .then_some(command('o'))
                        ),
                ]
                .align_y(iced::Center),
            )
            .padding(12)
            .style(styles::surface),
        );
    }
    if matches!(app.tab, Tab::OutputStyles | Tab::Statusline) {
        let default = if app.tab == Tab::OutputStyles {
            app.current_output_style.as_deref()
        } else {
            app.current_statusline.as_deref()
        };
        main = main.push(
            row![
                text(format!("Default: {}", default.unwrap_or("Not set")))
                    .size(12)
                    .style(styles::muted),
                key_button("Set default", KeyCode::Char('s')).style(styles::subtle),
                key_button("Clear default", KeyCode::Char('u')).style(styles::subtle),
            ]
            .spacing(8)
            .wrap(),
        );
    }
    let table = column![
        row![
            text("NAME / SOURCE").size(10).style(styles::muted),
            Space::new().width(Fill),
            text("STATUS").size(10).style(styles::muted)
        ]
        .padding([10, 16]),
        rule::horizontal(1),
        scrollable(rows::items(app))
            .id("component-list")
            .height(Fill),
    ];
    main = main.push(container(table).style(styles::surface).height(Fill));
    if !compact && let Some(details) = focused_details(app) {
        main = main.push(scrollable(text(details).size(11).style(styles::muted)).height(34));
    }
    let controls = column![
        rule::horizontal(1),
        row![
            column![
                text(format!("{selected} selected")).size(14),
                text("In this view").size(10).style(styles::muted)
            ]
            .spacing(2),
            Space::new().width(Fill),
            key_button(
                if selected > 0 && selected == total {
                    "Clear selection"
                } else {
                    "Select all"
                },
                KeyCode::Char('a')
            )
            .style(styles::subtle)
            .on_press_maybe((total > 0).then_some(command('a'))),
            key_button("Diff", KeyCode::Char('d'))
                .style(styles::subtle)
                .on_press_maybe(
                    app.selected_component_index()
                        .is_some()
                        .then_some(command('d'))
                ),
            key_button("Remove", KeyCode::Char('r'))
                .style(styles::danger)
                .on_press_maybe((selected > 0).then_some(command('r'))),
            key_button("Install", KeyCode::Char('i'))
                .padding([9, 16])
                .on_press_maybe((eligible > 0).then_some(command('i'))),
        ]
        .spacing(8)
        .align_y(iced::Center),
    ]
    .spacing(12);
    row![sidebar::view(app), main.push(controls).height(Fill)]
        .spacing(24)
        .height(Fill)
        .into()
}

fn search(app: &App) -> Element<'_, Message> {
    let mut field = row![super::text_input::current_modifiers(
        text_input("Search this category…", &app.search_query)
            .id("component-search")
            .size(13)
            .padding([8, 10])
            .on_input(|query| Message::Action(Action::SetSearch(query)))
            .on_submit(Message::FocusResults)
    )]
    .spacing(4)
    .align_y(iced::Center);
    if !app.search_query.is_empty() {
        field = field.push(
            button(text("Clear").size(12))
                .style(styles::subtle)
                .on_press(Message::Action(Action::ClearSearch)),
        );
    }
    field.width(Fill).into()
}

fn command(value: char) -> Message {
    Message::Action(Action::Command(KeyCode::Char(value)))
}

fn description(tab: Tab) -> &'static str {
    match tab {
        Tab::Agents => "Specialist instructions for focused work.",
        Tab::Commands => "Reusable workflows for your everyday tasks.",
        Tab::Contexts => "Context that helps your assistant understand your work.",
        Tab::Rules => "Consistent conventions across your projects.",
        Tab::Skills => "Tools and knowledge for the way you build.",
        Tab::Hooks => "Automation that runs at specific events.",
        Tab::OutputStyles => "Choose how your assistant communicates.",
        Tab::Statusline => "Useful context below your Claude Code prompt.",
        Tab::Config => "Shared settings with your existing preferences preserved.",
        Tab::McpServers => "Connect your assistant to external tools and services.",
        Tab::Plugins => "Extend Claude Code with marketplace plugins.",
    }
}

pub(super) fn selection_summary(app: &App) -> (usize, usize, usize) {
    match app.tab {
        Tab::McpServers => {
            let shown = app.visible_mcp_servers();
            let selected = shown.iter().filter(|(_, item)| item.selected).count();
            (selected, selected, shown.len())
        }
        Tab::Plugins => {
            let shown = app.visible_plugins();
            let selected = shown.iter().filter(|(_, item)| item.selected).count();
            (selected, selected, shown.len())
        }
        _ => {
            let shown = app.visible_components();
            (
                shown.iter().filter(|(_, item)| item.selected).count(),
                shown
                    .iter()
                    .filter(|(_, item)| item.selected && item.is_install_eligible())
                    .count(),
                shown.len(),
            )
        }
    }
}

fn focused_details(app: &App) -> Option<String> {
    match app.tab {
        Tab::McpServers => app
            .visible_mcp_servers()
            .into_iter()
            .find(|(index, _)| *index == app.mcp_index)
            .map(|(_, item)| format!("{} · {}", item.def.name, item.def.description)),
        Tab::Plugins => app
            .visible_plugins()
            .into_iter()
            .find(|(index, _)| *index == app.plugin_index)
            .map(|(_, item)| {
                format!(
                    "{} · {}",
                    item.def.name,
                    item.def.comment.as_deref().unwrap_or(&item.def.source)
                )
            }),
        _ => app
            .selected_component_index()
            .and_then(|index| app.components.get(index))
            .map(|item| {
                format!(
                    "{}  →  {}",
                    item.source_path.display(),
                    item.dest_path.display()
                )
            }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::test_support::fresh_app;
    use crate::component::{Component, ComponentType, InstallStatus};
    #[test]
    fn empty_search_results_do_not_expose_hidden_mcp_or_plugin_details() {
        let mut app = fresh_app();
        app.mcp_servers = vec![crate::app::test_support::make_mcp_server()];
        app.plugins = vec![crate::app::test_support::make_plugin()];
        app.set_search_query("no-such-result".into());
        for tab in [Tab::McpServers, Tab::Plugins] {
            app.tab = tab;
            assert_eq!(selection_summary(&app), (0, 0, 0));
            assert!(focused_details(&app).is_none());
        }
    }

    #[test]
    fn action_counts_exclude_other_tabs_filtered_files_and_external_installation() {
        let mut app = fresh_app();
        let mut external = Component::new(
            ComponentType::Skills,
            "external".into(),
            "external".into(),
            "external".into(),
            InstallStatus::External,
        );
        external.selected = true;
        app.components = vec![
            external,
            Component::new(
                ComponentType::Agents,
                "agent".into(),
                "a".into(),
                "a".into(),
                InstallStatus::New,
            ),
        ];
        assert_eq!(selection_summary(&app), (1, 0, 1));
        app.component_filter = ComponentFilter::New;
        assert_eq!(selection_summary(&app), (0, 0, 0));
    }
}
