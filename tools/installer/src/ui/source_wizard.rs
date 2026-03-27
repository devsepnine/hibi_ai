use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::App;
use crate::source::SourceEntry;

/// Render the "Add Source" type selection dialog.
pub fn render_type_select(f: &mut Frame, app: &App, area: Rect) {
    let dialog = centered_rect(40, 8, area);
    f.render_widget(Clear, dialog);

    let text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  [1] ", Style::default().fg(app.theme.accent_primary())),
            Span::raw("Git repository"),
        ]),
        Line::from(vec![
            Span::styled("  [2] ", Style::default().fg(app.theme.accent_primary())),
            Span::raw("Local directory"),
        ]),
        Line::from(""),
        Line::from(Span::styled("  [Esc] Cancel", Style::default().fg(app.theme.text_muted()))),
    ];

    let block = Block::default()
        .title(" Add Source ")
        .title_style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(app.theme.accent_primary()));

    let paragraph = Paragraph::new(text).block(block);
    f.render_widget(paragraph, dialog);
}

/// Render a text input dialog for URL, path, or branch.
pub fn render_text_input(f: &mut Frame, app: &App, area: Rect, title: &str, label: &str) {
    let dialog = centered_rect(60, 9, area);
    f.render_widget(Clear, dialog);

    let cursor = Span::styled("_", Style::default()
        .fg(app.theme.accent_secondary())
        .add_modifier(Modifier::SLOW_BLINK));

    let mut lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(format!("  {}: ", label), Style::default().fg(app.theme.text_muted())),
            Span::raw(&app.source_input_buffer),
            cursor,
        ]),
    ];

    // Show validation error if present
    if let Some(err) = &app.source_input_error {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("  {}", err),
            Style::default().fg(app.theme.error()),
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  [Enter] Confirm  [Esc] Cancel",
        Style::default().fg(app.theme.text_muted()),
    )));

    let block = Block::default()
        .title(format!(" {} ", title))
        .title_style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(app.theme.accent_primary()));

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, dialog);
}

/// Render the removal confirmation dialog.
pub fn render_confirm_remove(f: &mut Frame, app: &App, area: Rect) {
    let dialog = centered_rect(50, 8, area);
    f.render_widget(Clear, dialog);

    let entry_idx = app.source_list_index.saturating_sub(1);
    let source_label = app.source_entries.get(entry_idx)
        .map(|e| match e {
            SourceEntry::Git { url, .. } => url.as_str(),
            SourceEntry::Local { path, .. } => path.to_str().unwrap_or("?"),
        })
        .unwrap_or("?");

    let text = vec![
        Line::from(""),
        Line::from(Span::styled("  Remove this source?", Style::default().fg(app.theme.warning()))),
        Line::from(Span::styled(format!("  {}", source_label), Style::default().fg(app.theme.text_primary()))),
        Line::from(""),
        Line::from(vec![
            Span::styled("  [y] ", Style::default().fg(app.theme.error())),
            Span::raw("Yes  "),
            Span::styled("[Esc] ", Style::default().fg(app.theme.text_muted())),
            Span::raw("No"),
        ]),
    ];

    let block = Block::default()
        .title(" Confirm ")
        .title_style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(app.theme.warning()));

    let paragraph = Paragraph::new(text).block(block);
    f.render_widget(paragraph, dialog);
}

/// Render the map_to component type selection dialog.
pub fn render_map_to_select(f: &mut Frame, app: &App, area: Rect) {
    let dialog = centered_rect(45, 14, area);
    f.render_widget(Clear, dialog);

    let accent = app.theme.accent_primary();
    let text = vec![
        Line::from(""),
        Line::from(Span::styled("  Map all files to (optional):", Style::default().fg(app.theme.text_primary()))),
        Line::from(""),
        Line::from(vec![Span::styled("  [1] ", Style::default().fg(accent)), Span::raw("agents")]),
        Line::from(vec![Span::styled("  [2] ", Style::default().fg(accent)), Span::raw("commands")]),
        Line::from(vec![Span::styled("  [3] ", Style::default().fg(accent)), Span::raw("contexts")]),
        Line::from(vec![Span::styled("  [4] ", Style::default().fg(accent)), Span::raw("rules")]),
        Line::from(vec![Span::styled("  [5] ", Style::default().fg(accent)), Span::raw("skills")]),
        Line::from(vec![Span::styled("  [6] ", Style::default().fg(accent)), Span::raw("hooks")]),
        Line::from(vec![Span::styled("  [7] ", Style::default().fg(accent)), Span::raw("output-styles")]),
        Line::from(""),
        Line::from(Span::styled("  [Enter] Skip  [Esc] Cancel", Style::default().fg(app.theme.text_muted()))),
    ];

    let block = Block::default()
        .title(" Map To ")
        .title_style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(app.theme.accent_primary()));

    let paragraph = Paragraph::new(text).block(block);
    f.render_widget(paragraph, dialog);
}

/// Render a syncing spinner overlay.
pub fn render_syncing(f: &mut Frame, app: &App, area: Rect) {
    let dialog = centered_rect(35, 5, area);
    f.render_widget(Clear, dialog);

    let spinner = super::get_spinner(app.animation_frame);
    let text = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("  {}  Syncing sources...", spinner),
            Style::default().fg(app.theme.accent_secondary()),
        )),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(app.theme.accent_primary()));

    let paragraph = Paragraph::new(text).alignment(Alignment::Center).block(block);
    f.render_widget(paragraph, dialog);
}

/// Create a centered rectangle within the given area.
fn centered_rect(width_pct: u16, height: u16, area: Rect) -> Rect {
    let v = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(height),
            Constraint::Min(0),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - width_pct) / 2),
            Constraint::Percentage(width_pct),
            Constraint::Percentage((100 - width_pct) / 2),
        ])
        .split(v[1])[1]
}
