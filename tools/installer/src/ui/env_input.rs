use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::App;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let server_name = app.current_env_server_name().unwrap_or("Unknown");
    let current_var = app.current_env_var().unwrap_or("Unknown");
    let total_vars = app.env_input_vars.len();
    let current_idx = app.env_input_current + 1;

    // Center the dialog
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Length(12),
            Constraint::Percentage(30),
        ])
        .split(area);

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(60),
            Constraint::Percentage(20),
        ])
        .split(vertical[1]);

    let dialog_area = horizontal[1];

    // Clear background
    f.render_widget(Clear, dialog_area);

    // Build content
    let title = format!(" Environment Variables for {} ({}/{}) ", server_name, current_idx, total_vars);

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Variable: ", Style::default().fg(app.theme.text_secondary())),
            Span::styled(current_var, Style::default().fg(app.theme.warning()).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Value: ", Style::default().fg(app.theme.text_secondary())),
            Span::styled(&app.env_input_buffer, Style::default().fg(app.theme.text_primary())),
            Span::styled("_", Style::default().fg(app.theme.accent_secondary()).add_modifier(Modifier::SLOW_BLINK)),
        ]),
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::styled("  [Enter] Submit  [Esc] Cancel", Style::default().fg(app.theme.text_muted())),
        ]),
    ];

    let paragraph = Paragraph::new(lines)
        .style(Style::default().bg(app.theme.bg_secondary()))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(app.theme.border_focused()))
                .title(title)
                .title_style(Style::default().fg(app.theme.text_primary()))
                .style(Style::default().bg(app.theme.bg_secondary())),
        )
        .alignment(Alignment::Left);

    f.render_widget(paragraph, dialog_area);

    // Show already collected values below the dialog
    if !app.env_input_values.is_empty() {
        let collected_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(12),
                Constraint::Length(app.env_input_values.len() as u16 + 2),
            ])
            .split(vertical[1])[1];

        let collected_horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Percentage(60),
                Constraint::Percentage(20),
            ])
            .split(collected_area);

        let mut collected_lines: Vec<Line> = app.env_input_values.iter()
            .map(|(name, value)| {
                let masked = if value.len() > 4 {
                    format!("{}...", &value[..4])
                } else {
                    "****".to_string()
                };
                Line::from(vec![
                    Span::styled(format!("  {} = ", name), Style::default().fg(app.theme.text_secondary())),
                    Span::styled(masked, Style::default().fg(app.theme.success())),
                ])
            })
            .collect();

        collected_lines.insert(0, Line::from(Span::styled("  Collected:", Style::default().fg(app.theme.text_muted()))));

        let collected = Paragraph::new(collected_lines);
        f.render_widget(collected, collected_horizontal[1]);
    }
}
