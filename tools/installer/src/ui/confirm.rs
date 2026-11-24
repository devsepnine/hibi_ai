use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use super::layout;
use crate::app::{App, View};

pub(crate) fn render_source_error(f: &mut Frame, app: &App, message: &str, scroll: usize) -> usize {
    let dialog = layout::centered(f.area(), 78, 14);
    let block = Block::bordered().title(" Source settings ");
    let inner = block.inner(dialog);
    let lines: Vec<_> = message
        .lines()
        .flat_map(|line| layout::wrap_line(line, inner.width))
        .collect();
    let max_scroll = lines.len().saturating_sub(usize::from(inner.height));
    let keys = if max_scroll > 0 {
        " [j/k] Scroll  [Esc] Close "
    } else {
        " [Enter/Esc] Close "
    };
    f.render_widget(Clear, dialog);
    let block = block.title_bottom(keys).style(
        Style::default()
            .fg(app.theme.error())
            .bg(app.theme.bg_secondary()),
    );
    let visible: Vec<_> = lines
        .into_iter()
        .skip(scroll.min(max_scroll))
        .take(inner.height.into())
        .map(Line::raw)
        .collect();
    f.render_widget(Paragraph::new(visible).block(block), dialog);
    max_scroll
}

/// The prompt's two lines plus their surrounding blanks and borders.
const HEIGHT: u16 = 7;

/// Wide enough for the longest line below with a column of slack on each side.
/// `layout::centered` crops it on a narrower terminal rather than overflowing.
const WIDTH: u16 = 40;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let (question, detail) = match app.current_view {
        View::ConfirmExit => (
            "Leave for the CLI picker?".into(),
            "Selected items will be discarded.",
        ),
        View::ConfirmRemove(count) => (
            format!("Remove {count} selected items?"),
            "This action cannot be undone.",
        ),
        _ => return,
    };
    let dialog = layout::centered(area, WIDTH, HEIGHT);
    f.render_widget(Clear, dialog);

    let text = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("  {question}"),
            Style::default().fg(app.theme.warning()),
        )),
        Line::from(Span::styled(
            format!("  {detail}"),
            Style::default().fg(app.theme.text_primary()),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("  [y] ", Style::default().fg(app.theme.error())),
            Span::raw("Yes  "),
            Span::styled("[Esc] ", Style::default().fg(app.theme.text_muted())),
            Span::raw("No"),
        ]),
    ];

    // The title takes a theme colour rather than white: the light theme's
    // `bg_secondary` is near-white, so a hardcoded white title vanishes into the
    // dialog it is supposed to name.
    let block = Block::default()
        .title(" Confirm ")
        .title_style(
            Style::default()
                .fg(app.theme.text_primary())
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(app.theme.warning()))
        .style(Style::default().bg(app.theme.bg_secondary()));

    f.render_widget(Paragraph::new(text).block(block), dialog);
}
