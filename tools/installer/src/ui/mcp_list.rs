use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use crate::app::App;
use crate::mcp::McpStatus;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    if app.mcp_servers.is_empty() {
        let message = "No MCP servers found. Create mcps/mcps.yaml to add servers.";

        let empty = List::new(vec![ListItem::new(Line::from(vec![Span::styled(
            message,
            Style::default().fg(app.theme.text_muted()),
        )]))])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(app.theme.border()))
                .title(" MCP Servers ")
                .title_style(Style::default().fg(app.theme.text_primary())),
        );
        f.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = app
        .mcp_servers
        .iter()
        .map(|m| {
            let checkbox = if m.selected {
                "[x]"
            } else if m.status == McpStatus::Installed {
                "[*]"
            } else {
                "[ ]"
            };

            let status_style = match m.status {
                McpStatus::Installed => Style::default().fg(app.theme.success()),
                McpStatus::NotInstalled => Style::default().fg(app.theme.text_muted()),
            };

            // First line: checkbox, name, status, category, env warning
            let line1 = Line::from(vec![
                Span::raw(format!("{} ", checkbox)),
                Span::styled(
                    format!("{:<24}", m.def.name),
                    Style::default().fg(app.theme.text_primary()),
                ),
                Span::styled(
                    format!("({:^13})", m.status.display()),
                    status_style,
                ),
                Span::styled(
                    format!(" [{}]", m.def.category),
                    Style::default().fg(app.theme.accent_primary()),
                ),
                if !m.def.env.is_empty() {
                    Span::styled(" ⚠ env", Style::default().fg(app.theme.warning()))
                } else {
                    Span::raw("")
                },
            ]);

            // Second line: description (indented)
            let line2 = Line::from(vec![
                Span::raw("    "),
                Span::styled(
                    m.def.description.clone(),
                    Style::default().fg(app.theme.text_secondary()),
                ),
            ]);

            ListItem::new(vec![line1, line2])
        })
        .collect();

    let title = format!(" MCP Servers (scope: {}) ", app.mcp_scope.display());
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
    state.select(Some(app.mcp_index));

    f.render_stateful_widget(list, area, &mut state);
}
