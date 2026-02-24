use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let content = app.diff_content.as_deref().unwrap_or("No diff available");

    let lines: Vec<Line> = content
        .lines()
        .map(|line| {
            let style = if line.starts_with('+') && !line.starts_with("+++") {
                Style::default().fg(app.theme.diff_added())
            } else if line.starts_with('-') && !line.starts_with("---") {
                Style::default().fg(app.theme.diff_removed())
            } else if line.starts_with("@@") {
                Style::default().fg(app.theme.accent_secondary())
            } else if line.starts_with("---") || line.starts_with("+++") {
                Style::default().fg(app.theme.warning())
            } else {
                Style::default().fg(app.theme.text_primary())
            };

            Line::from(Span::styled(line, style))
        })
        .collect();

    let title = if let Some(idx) = app.selected_component_index() {
        if let Some(c) = app.components.get(idx) {
            format!(" Diff: {} ", c.display_name())
        } else {
            " Diff ".to_string()
        }
    } else {
        " Diff ".to_string()
    };

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(app.theme.border()))
                .title(title)
                .title_style(Style::default().fg(app.theme.text_primary())),
        )
        .scroll((app.diff_scroll, 0));

    f.render_widget(paragraph, area);
}
