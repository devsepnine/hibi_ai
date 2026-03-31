use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::app::App;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Length(3),  // Progress bar
            Constraint::Min(0),     // Log
        ])
        .split(area);

    render_title(f, app, chunks[0]);
    render_progress(f, app, chunks[1]);
    render_log(f, app, chunks[2]);
}

fn render_title(f: &mut Frame, app: &App, area: Rect) {
    let (title_text, title_color) = if app.processing_complete {
        ("✓ Complete".to_string(), app.theme.success())
    } else if app.needs_refresh {
        let spinner = super::get_spinner(app.animation_frame);
        (format!("{} Refreshing status...", spinner), app.theme.warning())
    } else {
        let spinner = super::get_spinner(app.animation_frame);
        let text = if app.is_removing {
            format!("{} Removing...", spinner)
        } else {
            format!("{} Installing...", spinner)
        };
        let color = if app.is_removing { app.theme.error() } else { app.theme.accent_secondary() };
        (text, color)
    };

    let title = Paragraph::new(title_text)
        .style(Style::default().fg(title_color).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(title, area);
}

fn render_progress(f: &mut Frame, app: &App, area: Rect) {
    let progress = app.processing_progress.unwrap_or(0);
    let total = app.processing_total.unwrap_or(1).max(1);
    let percent = ((progress as f64 / total as f64) * 100.0).min(100.0) as u16;

    let gauge_color = if app.is_removing { app.theme.error() } else { app.theme.success() };
    let gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(app.theme.border()))
                .title(" Progress ")
                .title_style(Style::default().fg(app.theme.text_primary())),
        )
        .gauge_style(Style::default().fg(gauge_color))
        .percent(percent)
        .label(format!("{}/{}", progress, total));
    f.render_widget(gauge, area);
}

fn render_log(f: &mut Frame, app: &App, area: Rect) {
    let log_items: Vec<ListItem> = app
        .processing_log
        .iter()
        .map(|msg| {
            let style = if msg.starts_with("[OK]") {
                Style::default().fg(app.theme.success())
            } else if msg.starts_with("[ERR]") {
                Style::default().fg(app.theme.error())
            } else if msg.starts_with("[SKIP]") {
                Style::default().fg(app.theme.warning())
            } else {
                Style::default().fg(app.theme.text_secondary())
            };
            ListItem::new(Line::from(Span::styled(msg.clone(), style)))
        })
        .collect();

    let log_len = log_items.len();
    let log_list = List::new(log_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(app.theme.border()))
                .title(" Log ")
                .title_style(Style::default().fg(app.theme.text_primary())),
        );

    // Auto-scroll to the last log entry
    let mut log_state = ListState::default();
    if log_len > 0 {
        log_state.select(Some(log_len - 1));
    }
    f.render_stateful_widget(log_list, area, &mut log_state);
}
