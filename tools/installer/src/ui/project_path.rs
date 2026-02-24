use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::App;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    // Center the dialog
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(35),
            Constraint::Length(9),
            Constraint::Percentage(35),
        ])
        .split(area);

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(15),
            Constraint::Percentage(70),
            Constraint::Percentage(15),
        ])
        .split(vertical[1]);

    let dialog_area = horizontal[1];

    // Clear background
    f.render_widget(Clear, dialog_area);

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Set project path for local MCP installation:", Style::default().fg(app.theme.text_secondary())),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Path: ", Style::default().fg(app.theme.text_secondary())),
            Span::styled(&app.project_path_buffer, Style::default().fg(app.theme.text_primary())),
            Span::styled("_", Style::default().fg(app.theme.accent_secondary()).add_modifier(Modifier::SLOW_BLINK)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  [Enter] Confirm  [Esc] Cancel (revert to user scope)", Style::default().fg(app.theme.text_muted())),
        ]),
    ];

    let paragraph = Paragraph::new(lines)
        .style(Style::default().bg(app.theme.bg_secondary()))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(app.theme.border_focused()))
                .title(" Local Scope - Project Path ")
                .title_style(Style::default().fg(app.theme.text_primary()))
                .style(Style::default().bg(app.theme.bg_secondary())),
        )
        .alignment(Alignment::Left);

    f.render_widget(paragraph, dialog_area);
}
