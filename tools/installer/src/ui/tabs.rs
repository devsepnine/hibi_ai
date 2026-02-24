use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Tabs as RataTabs},
    Frame,
};

use crate::app::App;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    // Build tab titles with number prefix: "1:Skills", "2:MCP", etc.
    let titles: Vec<String> = app.available_tabs
        .iter()
        .enumerate()
        .map(|(i, tab)| format!("{}:{}", i + 1, tab.display_name()))
        .collect();

    // Dynamic title based on selected CLI
    let title = if let Some(cli) = app.target_cli {
        format!(" {} Config Installer ({}) ", cli.display_name(), app.theme.mode().name())
    } else {
        String::from(" Config Installer ")
    };

    // Find selected tab index in available_tabs
    let selected_idx = app.available_tabs
        .iter()
        .position(|t| *t == app.tab)
        .unwrap_or(0);

    let tabs = RataTabs::new(titles.into_iter().map(Line::from).collect::<Vec<_>>())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(app.theme.border()))
                .title(title)
                .title_style(Style::default().fg(app.theme.text_primary())),
        )
        .select(selected_idx)
        .style(Style::default().fg(app.theme.text_primary()))
        .highlight_style(
            Style::default()
                .fg(app.theme.accent_primary())
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(tabs, area);
}
