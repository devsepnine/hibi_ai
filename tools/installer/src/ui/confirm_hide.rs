use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::Line,
    widgets::{Block, Borders, Clear, Paragraph},
};

use super::layout;
use crate::app::{App, HideConfirmation};

const WIDTH: u16 = 64;
const HEIGHT: u16 = 15;

pub fn max_scroll(app: &App, width: u16, height: u16) -> usize {
    let Some(pending) = &app.hide_confirmation else {
        return 0;
    };
    let width = width.min(WIDTH).saturating_sub(2);
    let height = height.min(HEIGHT).saturating_sub(2);
    body_lines(pending, width)
        .len()
        .saturating_sub(usize::from(height))
}

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let Some(pending) = &app.hide_confirmation else {
        return;
    };
    let dialog = layout::centered(area, WIDTH, HEIGHT);
    let block = Block::default()
        .title(" Confirm hide ")
        .title_bottom(" [y] Hide  [Esc/n/q] Cancel  [j/k] Scroll ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(app.theme.warning()))
        .style(
            Style::default()
                .fg(app.theme.text_primary())
                .bg(app.theme.bg_secondary()),
        );
    let inner = block.inner(dialog);
    f.render_widget(Clear, dialog);
    f.render_widget(block, dialog);
    let lines = body_lines(pending, inner.width);
    let offset = pending
        .scroll
        .min(lines.len().saturating_sub(usize::from(inner.height)));
    let shown: Vec<_> = lines.into_iter().skip(offset).map(Line::raw).collect();
    f.render_widget(Paragraph::new(shown), inner);
}

fn body_lines(pending: &HideConfirmation, width: u16) -> Vec<String> {
    let summary = [
        format!("Hide {} folder(s)?", pending.folders.len()),
        format!("Clear {} selected file(s).", pending.selected_files),
        "Files remain on disk.".to_string(),
        String::new(),
    ];
    summary
        .into_iter()
        .chain(pending.folders.iter().map(|name| format!("  {name}/")))
        .flat_map(|line| wrap_line(&line, width))
        .collect()
}

fn wrap_line(text: &str, width: u16) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for character in text.chars() {
        current.push(character);
        if layout::columns(&current) > usize::from(width) && current.len() > character.len_utf8() {
            current.pop();
            lines.push(std::mem::take(&mut current));
            current.push(character);
        }
    }
    lines.push(current);
    lines
}
