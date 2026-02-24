use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use crate::app::{App, Tab};
use crate::component::InstallStatus;
use crate::tree::TreeNode;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    // Use tree-based rendering for component tabs
    if let Some(tree) = app.get_tree_view() {
        render_tree(f, app, tree, area);
    } else {
        // Fallback to flat list (shouldn't happen for component tabs)
        render_flat(f, app, area);
    }
}

fn render_tree(f: &mut Frame, app: &App, tree: &crate::tree::TreeView, area: Rect) {
    let items: Vec<ListItem> = tree.visible_indices
        .iter()
        .map(|&node_idx| {
            let node = &tree.nodes[node_idx];
            render_tree_node(app, tree, node, node_idx)
        })
        .collect();

    // Show current default status in title
    let mut title = format!(" {} ", app.tab.display_name());
    if app.tab == Tab::OutputStyles {
        if let Some(ref style) = app.current_output_style {
            title = format!("{} [Default: {}] ", title.trim(), style);
        } else {
            title = format!("{} [No default set] ", title.trim());
        }
    } else if app.tab == Tab::Statusline {
        if let Some(ref statusline) = app.current_statusline {
            title = format!("{} [Default: {}] ", title.trim(), statusline);
        } else {
            title = format!("{} [No default set] ", title.trim());
        }
    }

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(app.theme.border()))
                .title(title)
                .title_style(Style::default().fg(app.theme.text_primary()))
                .style(Style::default().bg(app.theme.bg_primary())),
        )
        .style(Style::default().fg(app.theme.text_primary()).bg(app.theme.bg_primary()))
        .highlight_style(
            Style::default()
                .bg(app.theme.selection_bg())
                .fg(app.theme.selection_fg())
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    let mut state = ListState::default();
    if !tree.visible_indices.is_empty() {
        state.select(Some(tree.cursor));
    }

    f.render_stateful_widget(list, area, &mut state);
}

fn render_tree_node(app: &App, tree: &crate::tree::TreeView, node: &TreeNode, node_idx: usize) -> ListItem<'static> {
    let depth = node.depth();
    let indent = "  ".repeat(depth);

    match node {
        TreeNode::Folder { name, expanded, .. } => {
            // Folder icon
            let icon = if *expanded { "v " } else { "> " };

            // Check folder selection state
            let (checkbox, checkbox_style) = if tree.is_folder_all_selected(node_idx, &app.components) {
                ("[x]", Style::default().fg(app.theme.success()))
            } else if tree.is_folder_any_selected(node_idx, &app.components) {
                ("[-]", Style::default().fg(app.theme.warning()))
            } else {
                ("[ ]", Style::default().fg(app.theme.text_muted()))
            };

            let line = Line::from(vec![
                Span::raw(format!("{}{}", indent, checkbox)),
                Span::styled(" ", checkbox_style),
                Span::styled(
                    icon,
                    Style::default().fg(app.theme.accent_primary()),
                ),
                Span::styled(
                    format!("{}/", name),
                    Style::default().fg(app.theme.accent_primary()).add_modifier(Modifier::BOLD),
                ),
            ]);

            ListItem::new(line)
        }
        TreeNode::File { component_idx, .. } => {
            let c = &app.components[*component_idx];
            let checkbox = if c.selected { "[x]" } else { "[ ]" };

            let status_style = match c.status {
                InstallStatus::New => Style::default().fg(app.theme.success()),
                InstallStatus::Modified => Style::default().fg(app.theme.warning()),
                InstallStatus::Unchanged => Style::default().fg(app.theme.text_secondary()),
                InstallStatus::Managed => Style::default().fg(app.theme.accent_primary()),
            };

            // Check if this is the default item
            let is_default = match app.tab {
                Tab::OutputStyles => {
                    // Compare style name without extension
                    let style_name = c.name.strip_suffix(".md").unwrap_or(&c.name);
                    app.current_output_style.as_ref().map(|s| s.as_str()) == Some(style_name)
                }
                Tab::Statusline => {
                    app.current_statusline.as_ref().map(|s| s.as_str()) == Some(&c.name)
                }
                _ => false,
            };

            let default_marker = match app.tab {
                Tab::OutputStyles | Tab::Statusline => {
                    if is_default {
                        " ★ DEFAULT"
                    } else if c.status == InstallStatus::New {
                        " (not installed)"
                    } else {
                        ""
                    }
                }
                _ => "",
            };

            // Extract just the filename (last part of path)
            let filename = c.name.rsplit('/').next().unwrap_or(&c.name);

            // For hooks, show description and event
            let mut spans = vec![
                Span::raw(format!("{}{} ", indent, checkbox)),
                Span::styled(
                    format!("{:<20}", filename),
                    Style::default().fg(app.theme.text_primary()),
                ),
                Span::styled(format!("({:^9})", c.status.display()), status_style),
                Span::styled(default_marker, Style::default().fg(app.theme.peach()).add_modifier(Modifier::BOLD)),
            ];

            if app.tab == Tab::Hooks {
                if let Some(ref config) = c.hook_config {
                    // Add event info
                    spans.push(Span::styled(
                        format!(" [{}]", config.event),
                        Style::default().fg(app.theme.highlight()),
                    ));
                    // Add description if available
                    if let Some(ref desc) = config.description {
                        spans.push(Span::styled(
                            format!(" - {}", desc),
                            Style::default().fg(app.theme.text_muted()),
                        ));
                    }
                }
            }

            let line = Line::from(spans);
            ListItem::new(line)
        }
    }
}

fn render_flat(f: &mut Frame, app: &App, area: Rect) {
    let filtered = app.current_components();

    let items: Vec<ListItem> = filtered
        .iter()
        .map(|(_, c)| {
            let checkbox = if c.selected { "[x]" } else { "[ ]" };

            let status_style = match c.status {
                InstallStatus::New => Style::default().fg(app.theme.success()),
                InstallStatus::Modified => Style::default().fg(app.theme.warning()),
                InstallStatus::Unchanged => Style::default().fg(app.theme.text_secondary()),
                InstallStatus::Managed => Style::default().fg(app.theme.accent_primary()),
            };

            // Check if this is the default item
            let is_default = match app.tab {
                Tab::OutputStyles => {
                    // Compare style name without extension
                    let style_name = c.name.strip_suffix(".md").unwrap_or(&c.name);
                    app.current_output_style.as_ref().map(|s| s.as_str()) == Some(style_name)
                }
                Tab::Statusline => {
                    app.current_statusline.as_ref().map(|s| s.as_str()) == Some(&c.name)
                }
                _ => false,
            };

            let default_marker = match app.tab {
                Tab::OutputStyles | Tab::Statusline => {
                    if is_default {
                        " ★ DEFAULT"
                    } else if c.status == InstallStatus::New {
                        " (not installed)"
                    } else {
                        ""
                    }
                }
                _ => "",
            };

            let line = Line::from(vec![
                Span::raw(format!("{} ", checkbox)),
                Span::styled(
                    format!("{:<40}", c.name),
                    Style::default().fg(app.theme.text_primary()),
                ),
                Span::styled(format!("({:^9})", c.status.display()), status_style),
                Span::styled(default_marker, Style::default().fg(app.theme.peach()).add_modifier(Modifier::BOLD)),
            ]);

            ListItem::new(line)
        })
        .collect();

    // Show current default status in title
    let mut title = format!(" {} ", app.tab.display_name());
    if app.tab == Tab::OutputStyles {
        if let Some(ref style) = app.current_output_style {
            title = format!("{} [Default: {}] ", title.trim(), style);
        } else {
            title = format!("{} [No default set] ", title.trim());
        }
    } else if app.tab == Tab::Statusline {
        if let Some(ref statusline) = app.current_statusline {
            title = format!("{} [Default: {}] ", title.trim(), statusline);
        } else {
            title = format!("{} [No default set] ", title.trim());
        }
    }

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(app.theme.border()))
                .title(title)
                .title_style(Style::default().fg(app.theme.text_primary())),
        )
        .highlight_style(
            Style::default()
                .bg(app.theme.selection_bg())
                .fg(app.theme.selection_fg())
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    let mut state = ListState::default();
    if !filtered.is_empty() {
        state.select(Some(app.list_index));
    }

    f.render_stateful_widget(list, area, &mut state);
}
