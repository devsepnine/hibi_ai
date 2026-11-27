use super::Language;
use crossterm::event::KeyCode;
use iced::widget::{
    Space, button, column, container, row, rule, scrollable, text, text_input, tooltip,
};
use iced::{Element, Fill};

use super::{Message, actions::Action, filters, key_button, rows, sidebar, styles};
use crate::app::{App, FocusArea, Tab};

pub fn view(
    lang: Language,
    app: &App,
    compact: bool,
    filter_choice: Option<filters::Choice>,
) -> Element<'_, Message> {
    let selection = app.work_selection();
    let total = selection.visible;
    let focus = filter_choice.is_none().then_some(app.focus);
    let selected_folders = app.selected_folders().len();
    let expand = app.get_tree_view().is_some_and(|tree| {
        tree.nodes
            .iter()
            .any(|node| node.is_folder() && !node.is_expanded())
    });
    let mut heading =
        column![text(lang.text(app.tab.display_name())).size(if compact { 22 } else { 24 })]
            .spacing(5);
    if !compact {
        heading = heading.push(
            text(lang.text(description(app.tab)))
                .size(12)
                .style(styles::muted),
        );
    }
    let mut main = column![
        row![
            heading,
            Space::new().width(Fill),
            container(text(lang.result_count(total, !app.search_query.trim().is_empty())).size(12))
                .padding([5, 9])
                .style(styles::badge),
        ]
        .align_y(iced::Center),
    ]
    .spacing(if compact { 8 } else { 12 });
    if app.tab.to_component_type().is_some() {
        main = main
            .push(
                row![
                    search(lang, app, filter_choice.is_none()),
                    filters::view(lang, app, filter_choice)
                ]
                .spacing(8)
                .align_y(iced::Top),
            )
            .push(
                row![
                    key_button(
                        if expand {
                            lang.text("Expand all")
                        } else {
                            lang.text("Collapse all")
                        },
                        KeyCode::Char('E')
                    )
                    .style(styles::secondary),
                    button(
                        text(localized_format!(
                            lang,
                            "Hide selected · {selected_folders}",
                            "선택한 폴더 숨기기 · {selected_folders}",
                            "選択したフォルダーを隠す · {selected_folders}"
                        ))
                        .size(12)
                    )
                    .padding([7, 10])
                    .style(styles::secondary)
                    .on_press_maybe(
                        (selected_folders > 0)
                            .then_some(Message::Action(Action::HideSelectedFolders))
                    ),
                    button(
                        text(localized_format!(
                            lang,
                            "Unhide selected · {selected_folders}",
                            "선택한 폴더 표시 · {selected_folders}",
                            "選択したフォルダーを表示 · {selected_folders}"
                        ))
                        .size(12)
                    )
                    .padding([7, 10])
                    .style(styles::secondary)
                    .on_press_maybe(
                        (selected_folders > 0)
                            .then_some(Message::Action(Action::UnhideSelectedFolders))
                    ),
                ]
                .spacing(6)
                .wrap(),
            );
    } else {
        main = main.push(search(lang, app, true));
    }
    if app.tab == Tab::McpServers {
        main = main.push(
            container(
                row![
                    column![
                        text(localized_format!(
                            lang,
                            "Installation scope: {}",
                            "설치 범위: {}",
                            "インストール範囲: {}",
                            if app.target_cli == Some(crate::app::TargetCli::Codex) {
                                lang.text("user")
                            } else {
                                lang.text(app.mcp_scope.display())
                            }
                        ))
                        .size(13),
                        text(&app.mcp_project_path).size(11).style(styles::muted)
                    ]
                    .spacing(3),
                    Space::new().width(Fill),
                    key_button(lang.text("Change scope"), KeyCode::Char('o'))
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
                text(localized_format!(
                    lang,
                    "Default: {}",
                    "기본값: {}",
                    "既定値: {}",
                    default.unwrap_or(lang.text("Not set"))
                ))
                .size(12)
                .style(styles::muted),
                key_button(lang.text("Set default"), KeyCode::Char('s')).style(styles::subtle),
                key_button(lang.text("Clear default"), KeyCode::Char('u')).style(styles::subtle),
            ]
            .spacing(8)
            .wrap(),
        );
    }
    let table = column![
        row![
            text(lang.text("Component"))
                .size(11)
                .style(if focus == Some(FocusArea::Content) {
                    text::primary
                } else {
                    styles::muted
                }),
            Space::new().width(Fill),
            text(lang.text("Status")).size(11).style(styles::muted)
        ]
        .padding([10, 16]),
        rule::horizontal(1).style(styles::divider),
        scrollable(rows::items(lang, app, focus == Some(FocusArea::Content)))
            .id("component-list")
            .height(Fill),
    ];
    main = main.push(container(table).style(styles::surface).height(Fill));
    row![
        sidebar::view(lang, app, focus == Some(FocusArea::Tabs)),
        main.push(work_bar(lang, app, &selection)).height(Fill)
    ]
    .spacing(18)
    .height(Fill)
    .into()
}

fn work_bar<'a>(
    lang: Language,
    app: &'a App,
    selection: &crate::app::WorkSelection,
) -> Element<'a, Message> {
    let selected = selection.selected;
    let install = selection.install.len();
    let remove = selection.remove.len();
    container(
        row![
            selection_label(lang, app, selected),
            Space::new().width(Fill),
            key_button(
                if selected > 0 && selected == selection.visible {
                    lang.text("Clear selection")
                } else {
                    lang.text("Select all")
                },
                KeyCode::Char('a')
            )
            .style(styles::subtle)
            .on_press_maybe((selection.visible > 0).then_some(command('a'))),
            key_button(lang.text("Diff"), KeyCode::Char('d'))
                .style(styles::secondary)
                .on_press_maybe(
                    app.selected_component_index()
                        .is_some()
                        .then_some(command('d'))
                ),
            button(
                text(localized_format!(
                    lang,
                    "Remove {remove}",
                    "{remove}개 제거",
                    "{remove} 件削除"
                ))
                .size(12)
            )
            .padding([7, 10])
            .style(styles::danger)
            .on_press_maybe((remove > 0).then_some(Message::Action(Action::RequestRemove))),
            button(
                text(localized_format!(
                    lang,
                    "Install {install}",
                    "{install}개 설치",
                    "{install} 件インストール"
                ))
                .size(12)
            )
            .padding([9, 18])
            .style(styles::primary)
            .on_press_maybe((install > 0).then_some(Message::Action(Action::InstallSelected))),
        ]
        .spacing(8)
        .align_y(iced::Center),
    )
    .padding([8, 12])
    .style(styles::glass)
    .into()
}

fn search(lang: Language, app: &App, search_shortcut: bool) -> Element<'_, Message> {
    let mut field = row![super::text_input::search_input(
        text_input(lang.text("Search this category…"), &app.search_query)
            .id("component-search")
            .size(13)
            .padding([10, 12])
            .style(styles::input)
            .on_input(|query| Message::Action(Action::SetSearch(query)))
            .on_submit(Message::SearchSubmitted),
        search_shortcut,
    )]
    .spacing(4)
    .align_y(iced::Center);
    if !app.search_query.is_empty() {
        field = field.push(
            button(text(lang.text("Clear")).size(12))
                .style(styles::subtle)
                .on_press(Message::Action(Action::ClearSearch)),
        );
    }
    field.width(Fill).into()
}

fn selection_label(lang: Language, app: &App, selected: usize) -> Element<'_, Message> {
    let label = column![
        text(localized_format!(
            lang,
            "{selected} selected",
            "{selected}개 선택됨",
            "{selected} 件選択中"
        ))
        .size(13),
        text(lang.text("In this view"))
            .size(10)
            .style(styles::muted),
    ]
    .spacing(3);
    if let Some(details) = focused_details(app) {
        tooltip(
            label,
            container(text(details).size(12)).max_width(600).padding(10),
            tooltip::Position::Top,
        )
        .style(styles::popup)
        .into()
    } else {
        label.into()
    }
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
    use crate::app::{ComponentFilter, test_support::fresh_app};
    use crate::component::{Component, ComponentType, InstallStatus};
    #[test]
    fn empty_search_results_do_not_expose_hidden_mcp_or_plugin_details() {
        let mut app = fresh_app();
        app.mcp_servers = vec![crate::app::test_support::make_mcp_server()];
        app.plugins = vec![crate::app::test_support::make_plugin()];
        app.set_search_query("no-such-result".into());
        for tab in [Tab::McpServers, Tab::Plugins] {
            app.tab = tab;
            assert_eq!(app.work_selection().visible, 0);
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
        assert_eq!(app.work_selection().visible, 0);
        app.set_component_filter(ComponentFilter::External, true);
        let selection = app.work_selection();
        assert_eq!(selection.selected, 1);
        assert!(selection.install.is_empty());
        assert_eq!(selection.remove, [0]);
        assert_eq!(selection.visible, 1);
        app.component_filter = ComponentFilter::New.into();
        assert_eq!(app.work_selection().visible, 0);
    }

    #[test]
    fn installed_codex_mcp_is_excluded_from_the_install_count() {
        let mut app = fresh_app();
        app.target_cli = Some(crate::app::TargetCli::Codex);
        app.tab = Tab::McpServers;
        let mut server = crate::app::test_support::make_mcp_server();
        server.status = crate::mcp::McpStatus::Installed;
        server.selected = true;
        app.mcp_servers.push(server);

        let selection = app.work_selection();
        assert_eq!(selection.selected, 1);
        assert!(selection.install.is_empty());
        assert_eq!(selection.remove, [0]);
    }
}
