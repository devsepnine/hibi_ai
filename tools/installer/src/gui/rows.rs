use super::Language;
use super::{Message, actions::Action, styles};
use crate::app::{App, HiddenVisibility, Tab};
use crate::tree::TreeNode;
use iced::widget::{Column, Space, button, checkbox, column, container, row, svg, text, tooltip};
use iced::{Element, Fill};

const ROW_HEIGHT: f32 = 40.0;

pub fn items(lang: Language, app: &App, has_focus: bool) -> Column<'_, Message> {
    let mut rows = Column::new().spacing(2).padding(6).width(Fill);
    let mut count = 0;
    if app.tab == Tab::McpServers {
        for (index, item) in app.visible_mcp_servers() {
            rows = rows.push(file(
                lang,
                index,
                item.selected,
                has_focus && index == app.mcp_index,
                &item.def.name,
                item.status.display(),
                &item.source_name,
                format!(
                    "{} · {}{}",
                    item.def.category,
                    item.def.description,
                    if item.def.inputs().next().is_some() {
                        localized_format!(
                            lang,
                            " · Requires {}",
                            " · 필요: {}",
                            " · 必須: {}",
                            item.def.inputs().collect::<Vec<_>>().join(", ")
                        )
                    } else {
                        String::new()
                    }
                ),
            ));
            count += 1;
        }
    } else if app.tab == Tab::Plugins {
        for (index, item) in app.visible_plugins() {
            rows = rows.push(file(
                lang,
                index,
                item.selected,
                has_focus && index == app.plugin_index,
                &item.def.name,
                item.status.display(),
                &item.source_name,
                format!(
                    "{} · {} · {}",
                    item.def.marketplace,
                    item.short_repo(),
                    item.def.comment.as_deref().unwrap_or("")
                ),
            ));
            count += 1;
        }
    } else if let Some(tree) = app.tree_views.get(&app.tab) {
        for (index, &node_index) in tree.visible_indices.iter().enumerate() {
            let Some(node) = tree.nodes.get(node_index) else {
                continue;
            };
            let element: Element<'_, Message> = match node {
                TreeNode::Folder {
                    name,
                    expanded,
                    path,
                    ..
                } => row![
                    button(container(chevron(*expanded)).center_x(28).center_y(28))
                        .style(styles::subtle)
                        .padding(0)
                        .width(28)
                        .height(28)
                        .on_press(Message::Action(Action::ExpandRow(index))),
                    checkbox(tree.is_folder_all_selected(node_index, &app.components))
                        .size(15)
                        .style(styles::check)
                        .on_toggle(move |_| Message::Action(Action::ToggleRow(index))),
                    button(
                        row![
                            text(name).size(13),
                            Space::new().width(Fill),
                            text(if app.is_folder_hidden(path) {
                                lang.text("hidden")
                            } else if tree.is_folder_any_selected(node_index, &app.components)
                                && !tree.is_folder_all_selected(node_index, &app.components)
                            {
                                lang.text("partial selection")
                            } else {
                                lang.text("folder")
                            })
                            .size(11)
                            .style(styles::muted)
                        ]
                        .align_y(iced::Center)
                    )
                    .style(if has_focus && tree.cursor == index {
                        styles::focused_row
                    } else {
                        styles::subtle
                    })
                    .padding([7, 8])
                    .width(Fill)
                    .on_press(Message::Action(Action::SelectRow(index))),
                ]
                .spacing(8)
                .align_y(iced::Center)
                .height(ROW_HEIGHT)
                .into(),
                TreeNode::File { component_idx, .. } => {
                    let Some(item) = app.components.get(*component_idx) else {
                        continue;
                    };
                    let details = if let Some(reason) = item.status.blocking_reason() {
                        reason.to_owned()
                    } else {
                        item.hook_config
                            .as_ref()
                            .map(|hook| {
                                format!(
                                    "{} · {}{}",
                                    hook.event,
                                    hook.description.as_deref().unwrap_or(""),
                                    if hook.is_deprecated() {
                                        lang.text(" · deprecated")
                                    } else {
                                        ""
                                    }
                                )
                            })
                            .unwrap_or_else(|| item.name.clone())
                    };
                    file(
                        lang,
                        index,
                        item.selected,
                        has_focus && tree.cursor == index,
                        item.name.rsplit('/').next().unwrap_or(&item.name),
                        item.status.display(),
                        &item.source_name,
                        details,
                    )
                }
            };
            rows = rows.push(
                container(element)
                    .height(ROW_HEIGHT)
                    .center_y(ROW_HEIGHT)
                    .padding(iced::Padding {
                        left: (node.depth().min(8) * 14) as f32,
                        ..Default::default()
                    }),
            );
            count += 1;
        }
    }
    if count == 0 {
        let mut empty = column![
            text(if !app.search_query.trim().is_empty() {
                lang.text("No matching items")
            } else if app.hidden_visibility == HiddenVisibility::Only
                && app.tab.to_component_type().is_some()
            {
                lang.text("No hidden items in this view")
            } else {
                lang.text("Nothing to show here yet")
            })
            .size(20),
            text(lang.text(
                "Try another search, change the status filter, or choose another visibility mode."
            ))
            .size(13)
            .style(styles::muted),
        ]
        .spacing(8);
        if !app.search_query.is_empty() {
            empty = empty.push(
                button(lang.text("Clear search"))
                    .style(styles::subtle)
                    .on_press(Message::Action(Action::ClearSearch)),
            );
        }
        rows = rows.push(container(empty).padding(24).center_x(Fill));
    }
    rows
}

fn file<'a>(
    lang: Language,
    index: usize,
    selected: bool,
    focused: bool,
    name: &'a str,
    status: &'a str,
    source: &'a str,
    details: String,
) -> Element<'a, Message> {
    let name_column = container(
        column![
            text(name).size(13).wrapping(text::Wrapping::None),
            text(source).size(10).style(styles::muted)
        ]
        .spacing(1),
    )
    .width(Fill)
    .clip(true);
    let status_badge = container(text(lang.text(status)).size(11))
        .padding([3, 7])
        .style(move |theme: &iced::Theme| {
            let mut style = styles::badge(theme);
            style.text_color = Some(match status {
                "new" | "not installed" => theme.palette().primary,
                "modified" => theme.palette().warning,
                "installed" | "managed" => theme.palette().success,
                "blocked" => theme.palette().danger,
                _ => styles::muted(theme).color.unwrap_or(theme.palette().text),
            });
            style
        });
    row![
        checkbox(selected)
            .size(15)
            .style(styles::check)
            .on_toggle(move |_| Message::Action(Action::ToggleRow(index))),
        tooltip(
            button(
                row![name_column, status_badge]
                    .spacing(12)
                    .align_y(iced::Center)
            )
            .style(if focused {
                styles::focused_row
            } else {
                styles::subtle
            })
            .height(ROW_HEIGHT - 4.0)
            .width(Fill)
            .padding([2, 8])
            .on_press(Message::Action(Action::SelectRow(index))),
            container(
                column![
                    text(name).size(13),
                    text(details).size(12).style(styles::muted)
                ]
                .spacing(5)
            )
            .max_width(420)
            .padding(12),
            tooltip::Position::Top
        )
        .style(styles::popup),
    ]
    .spacing(12)
    .padding([0, 6])
    .height(ROW_HEIGHT)
    .align_y(iced::Center)
    .into()
}

fn chevron(expanded: bool) -> iced::widget::Svg<'static> {
    static HANDLES: std::sync::OnceLock<[svg::Handle; 2]> = std::sync::OnceLock::new();
    let handles = HANDLES.get_or_init(|| [
        svg::Handle::from_memory(
            br#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 16 16"><path d="M6 4L10 8L6 12" fill="none" stroke="black" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/></svg>"#,
        ),
        svg::Handle::from_memory(
            br#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 16 16"><path d="M4 6L8 10L12 6" fill="none" stroke="black" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/></svg>"#,
        ),
    ]);
    svg(handles[usize::from(expanded)].clone())
        .width(16)
        .height(16)
        .style(|theme: &iced::Theme, _| svg::Style {
            color: Some(theme.palette().text),
        })
}
