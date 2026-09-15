use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use super::layout;
use crate::app::App;

/// A keybinding row: which keys, and what they do.
type Binding = (&'static str, &'static str);

/// Every key the List view answers, grouped by the pane that answers it.
///
/// This is where a new binding gets documented, and `cli::handle_list_input` is
/// the only other place one may appear. The status bar deliberately lists none
/// of them: it used to, and at 80 columns the line ran off the right edge and
/// took `[q] Quit` with it. A scrollable box has no edge to fall off.
const SECTIONS: [(&str, &[Binding]); 4] = [
    (
        "Panes",
        &[
            ("1", "focus the tab bar"),
            ("2", "focus the content list"),
            ("Tab / Shift+Tab", "toggle between panes"),
        ],
    ),
    (
        "Tab bar",
        &[
            ("h / \u{2190}", "previous tab"),
            ("l / \u{2192}", "next tab"),
            ("Enter / Esc / j / \u{2193}", "back to the list"),
        ],
    ),
    (
        "Content list",
        &[
            ("j / \u{2193}, k / \u{2191}", "move the cursor"),
            ("h / \u{2190}", "collapse folder, or parent"),
            ("l / \u{2192}", "expand folder"),
            ("Space", "toggle selection"),
            ("a / n", "select all / none"),
            ("Enter", "expand folder, or diff file"),
            ("d", "diff against installed"),
            ("i", "install selected"),
            ("r", "remove selected"),
            ("s / u", "set / unset default"),
            ("o", "MCP scope (user / project)"),
            ("Esc", "back to the CLI picker"),
        ],
    ),
    (
        "Global",
        &[("?", "this help"), ("t", "switch theme"), ("q", "quit")],
    ),
];

/// Width of the key column, taken from the longest entry so a new binding
/// cannot silently overlap the description beside it.
fn key_width() -> usize {
    SECTIONS
        .iter()
        .flat_map(|(_, rows)| rows.iter())
        .map(|(keys, _)| keys.chars().count())
        .max()
        .unwrap_or(0)
}

/// Rows in the table: one per binding, a header per section, and a blank line
/// between sections.
fn row_count() -> u16 {
    let bindings: usize = SECTIONS.iter().map(|(_, rows)| rows.len()).sum();
    let separators = SECTIONS.len() * 2 - 1;
    (bindings + separators) as u16
}

/// Height the box wants: the whole table plus its top and bottom border.
fn preferred_height() -> u16 {
    row_count() + 2
}

/// Width the box wants: indent, key column, gutter, description, borders, and
/// two columns of right-hand slack so the text does not touch the frame.
fn preferred_width() -> u16 {
    let desc = SECTIONS
        .iter()
        .flat_map(|(_, rows)| rows.iter())
        .map(|(_, desc)| desc.chars().count())
        .max()
        .unwrap_or(0);
    (key_width() + desc + 8) as u16
}

/// Height the box takes in a terminal `height` rows tall.
///
/// 80% is a ceiling, not a target: the box is never taller than its own table,
/// so a roomy terminal still gets a snug box with no empty rows, while a short
/// one keeps a margin. Without the ceiling a 24-row window got a box flush
/// against all four edges, which read as a screen that had replaced the list
/// rather than one floating over it.
///
/// The floor of three rows is what the ceiling may not take away — two borders
/// and one row of table. Below that the box would be a frame around nothing.
pub(super) fn box_height(height: u16) -> u16 {
    const CEILING_PERCENT: u32 = 80;
    let ceiling = (u32::from(height) * CEILING_PERCENT / 100) as u16;
    preferred_height().min(ceiling.max(3)).min(height)
}

/// Rows of the table sitting below the fold in a terminal `height` rows tall.
///
/// The key handler needs the same answer the renderer computes, so both read it
/// from `box_height`: clamping the scroll to the table length instead would let
/// a fully visible table be scrolled off the top of a tall terminal, and sizing
/// the two independently would put the last row out of reach in a short one.
pub fn max_scroll(height: u16) -> u16 {
    let visible = box_height(height).saturating_sub(2);
    row_count().saturating_sub(visible)
}

fn lines(app: &App) -> Vec<Line<'static>> {
    let pad = key_width();
    let header_style = Style::default()
        .fg(app.theme.accent_primary())
        .add_modifier(Modifier::BOLD);
    let key_style = Style::default().fg(app.theme.text_primary());
    let desc_style = Style::default().fg(app.theme.text_secondary());

    let mut out = Vec::with_capacity(row_count() as usize);
    for (i, (section, rows)) in SECTIONS.iter().enumerate() {
        if i > 0 {
            out.push(Line::from(""));
        }
        out.push(Line::from(vec![
            Span::raw(" "),
            Span::styled(*section, header_style),
        ]));
        for (keys, desc) in rows.iter() {
            out.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(format!("{:<pad$}", keys), key_style),
                Span::raw("  "),
                Span::styled(*desc, desc_style),
            ]));
        }
    }
    out
}

/// Draw the keybinding reference over `area`.
///
/// The scroll and close keys ride on the bottom border rather than in the
/// status bar, so they stay on screen even when the box grows tall enough to
/// cover it.
pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let box_area = layout::centered(area, preferred_width(), box_height(area.height));

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(app.theme.border_focused()))
        .title(" Keybindings ")
        .title_style(Style::default().fg(app.theme.text_primary()))
        .title_bottom(" [j/k] Scroll  [?/Esc/q] Close ")
        .style(Style::default().bg(app.theme.bg_secondary()));

    let paragraph = Paragraph::new(lines(app))
        .block(block)
        .scroll((app.help_scroll.min(max_scroll(area.height)), 0));

    f.render_widget(Clear, box_area);
    f.render_widget(paragraph, box_area);
}
