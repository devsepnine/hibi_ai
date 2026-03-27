use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::app::App;
use crate::source::{SourceEntry, SourceKind};

/// Render the full-screen Sources management view.
pub fn render(f: &mut Frame, app: &App, area: Rect) {
    // Dynamic footer height: base 3 + extra lines for sync status
    let status_lines = app.source_sync_status.as_ref()
        .map(|s| s.split("; ").count())
        .unwrap_or(0);
    let footer_height = 3 + status_lines as u16;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(footer_height),
        ])
        .split(area);

    render_list(f, app, chunks[0]);
    render_footer(f, app, chunks[1]);
}

fn render_list(f: &mut Frame, app: &App, area: Rect) {
    let mut items = Vec::new();

    // Index 0: bundled source (always present, read-only)
    let bundled_style = if app.source_list_index == 0 {
        Style::default().fg(app.theme.selection_fg()).bg(app.theme.selection_bg())
    } else {
        Style::default().fg(app.theme.text_muted())
    };
    items.push(ListItem::new(Line::from(vec![
        Span::styled("  bundled", bundled_style),
        Span::styled("  [Built-in]", Style::default().fg(app.theme.text_muted())),
    ])));

    // User-configured entries
    for (i, entry) in app.source_entries.iter().enumerate() {
        let display_idx = i + 1; // +1 because bundled is index 0
        let is_selected = app.source_list_index == display_idx;

        let (label, kind_tag, extra) = match entry {
            SourceEntry::Git { url, branch, .. } => {
                let branch_str = branch.as_deref().unwrap_or("");
                let extra = if branch_str.is_empty() {
                    String::new()
                } else {
                    format!(" {}", branch_str)
                };
                (url.as_str(), "[Git]", extra)
            }
            SourceEntry::Local { path, .. } => {
                (path.to_str().unwrap_or("?"), "[Local]", String::new())
            }
        };

        let base_style = if is_selected {
            Style::default().fg(app.theme.selection_fg()).bg(app.theme.selection_bg())
        } else {
            Style::default().fg(app.theme.text_primary())
        };

        let kind_color = match entry {
            SourceEntry::Git { .. } => app.theme.accent_primary(),
            SourceEntry::Local { .. } => app.theme.success(),
        };

        // Check if this source is stale (match by label, not positional index)
        let stale = match entry {
            SourceEntry::Git { url, .. } => app.sources.iter()
                .any(|s| s.label == *url && s.is_stale),
            _ => false,
        };

        let mut spans = vec![
            Span::styled(format!("  {}", label), base_style),
            Span::styled(format!("  {}", kind_tag), Style::default().fg(kind_color)),
        ];
        if !extra.is_empty() {
            spans.push(Span::styled(extra, Style::default().fg(app.theme.text_muted())));
        }
        if stale {
            spans.push(Span::styled(" (stale)", Style::default().fg(app.theme.warning())));
        }

        items.push(ListItem::new(Line::from(spans)));
    }

    let list = List::new(items)
        .block(
            Block::default()
                .title(" Sources ")
                .title_style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(app.theme.border())),
        );

    f.render_widget(list, area);
}

fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    let on_bundled = app.source_list_index == 0;
    let has_git = app.sources.iter().any(|s| s.kind == SourceKind::Git);

    let mut help_parts = vec!["[a]Add"];
    if !on_bundled {
        help_parts.push("[e]Edit  [r]Remove");
    }
    if has_git {
        help_parts.push("[f]Sync");
    }
    help_parts.push("[Esc]Back");

    let help_line = help_parts.join("  ");

    let mut lines = vec![
        Line::from(Span::styled(help_line, Style::default().fg(app.theme.text_muted()))),
    ];

    if let Some(status) = &app.source_sync_status {
        let is_error = status.contains("failed")
            || status.contains("Failed")
            || status.contains("crashed")
            || status.contains("error");
        let color = if is_error { app.theme.error() } else { app.theme.accent_secondary() };
        // Split long status into multiple lines to prevent truncation
        for part in status.split("; ") {
            lines.push(Line::from(Span::styled(
                part.to_string(),
                Style::default().fg(color),
            )));
        }
    }

    let footer = Paragraph::new(lines)
        .wrap(ratatui::widgets::Wrap { trim: false })
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(app.theme.border())),
        );

    f.render_widget(footer, area);
}
