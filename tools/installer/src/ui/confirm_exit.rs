use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use super::layout;
use crate::app::App;

/// The prompt's two lines plus their surrounding blanks and borders.
const HEIGHT: u16 = 7;

/// Wide enough for the longest line below with a column of slack on each side.
/// `layout::centered` crops it on a narrower terminal rather than overflowing.
const WIDTH: u16 = 40;

/// Draw the "leave for the CLI picker?" prompt over `area`.
///
/// Styled after the source-removal prompt — warning border, `[y]`/`[Esc]` on
/// the last line — because both ask the same kind of question, and answering
/// them should not require reading two different dialogs.
pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let dialog = layout::centered(area, WIDTH, HEIGHT);
    f.render_widget(Clear, dialog);

    let text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "  Leave for the CLI picker?",
            Style::default().fg(app.theme.warning()),
        )),
        Line::from(Span::styled(
            "  Selected items will be discarded.",
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
        .title_style(Style::default().fg(app.theme.text_primary()).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(app.theme.warning()))
        .style(Style::default().bg(app.theme.bg_secondary()));

    f.render_widget(Paragraph::new(text).block(block), dialog);
}
