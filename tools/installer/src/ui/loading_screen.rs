use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Render the startup loading screen with a centered spinner animation.
pub fn draw(f: &mut Frame, frame_idx: usize) {
    let area = f.area();

    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Length(7),
            Constraint::Percentage(40),
        ])
        .split(area);

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(40),
            Constraint::Percentage(30),
        ])
        .split(vertical[1]);

    let spinner = super::get_spinner(frame_idx);
    let loading_text = format!("\n  {}  Loading...\n\n  Scanning components", spinner);

    let loading = Paragraph::new(loading_text)
        .style(Style::default().fg(Color::Cyan))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .title(" Config Installer ")
                .title_style(Style::default().fg(Color::White)),
        );

    f.render_widget(loading, horizontal[1]);
}
