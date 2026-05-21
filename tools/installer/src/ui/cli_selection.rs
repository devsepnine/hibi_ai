use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;

/// ANSI Shadow figlet rendering of "HIBI AI". 6 lines tall, ~50 cols wide.
const BRAND_ART: &[&str] = &[
    "██╗  ██╗██╗██████╗ ██╗     █████╗ ██╗",
    "██║  ██║██║██╔══██╗██║    ██╔══██╗██║",
    "███████║██║██████╔╝██║    ███████║██║",
    "██╔══██║██║██╔══██╗██║    ██╔══██║██║",
    "██║  ██║██║██████╔╝██║    ██║  ██║██║",
    "╚═╝  ╚═╝╚═╝╚═════╝ ╚═╝    ╚═╝  ╚═╝╚═╝",
];

/// Compact fallback when the terminal is too narrow for the figlet art.
/// Triggers below `BRAND_ART[0].chars().count()` columns of inner width.
const BRAND_COMPACT: &str = "HIBI AI";

/// Rows used by [`render_brand`]: one blank line, the 6 art rows, the
/// tagline, plus a blank line. Used to compute the brand area height so
/// the layout stays stable across themes.
const BRAND_HEIGHT: u16 = 1 + 6 + 1 + 1;

/// Options shown on the initial screen. Order is the cursor index order.
struct OptionRow {
    label: &'static str,
    detail: &'static str,
}

const OPTIONS: &[OptionRow] = &[
    OptionRow {
        label: "Claude Code",
        detail: "Anthropic's official CLI for Claude (~/.claude)",
    },
    OptionRow {
        label: "Codex CLI",
        detail: "OpenAI's ChatGPT-based CLI (~/.codex)",
    },
    OptionRow {
        label: "Manage Sources",
        detail: "Configure component sources (~/.hibi/sources.yaml)",
    },
];

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    // Three rows: brand banner, options, single-line version footer.
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(BRAND_HEIGHT),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    render_brand(f, app, chunks[0]);
    render_options(f, app, chunks[1]);
    render_version_footer(f, app, chunks[2]);
}

fn render_brand(f: &mut Frame, app: &App, area: Rect) {
    let inner_width = area.width as usize;
    let art_width = BRAND_ART[0].chars().count();
    let use_compact = inner_width < art_width;

    let mut lines: Vec<Line> = vec![Line::from("")];

    if use_compact {
        lines.push(Line::from(Span::styled(
            BRAND_COMPACT,
            Style::default()
                .fg(app.theme.accent_primary())
                .add_modifier(Modifier::BOLD),
        )));
        // Pad to keep the brand block the same height regardless of mode.
        for _ in 1..BRAND_ART.len() {
            lines.push(Line::from(""));
        }
    } else {
        for art_line in BRAND_ART {
            lines.push(Line::from(Span::styled(
                art_line.to_string(),
                Style::default()
                    .fg(app.theme.accent_primary())
                    .add_modifier(Modifier::BOLD),
            )));
        }
    }

    lines.push(Line::from(Span::styled(
        "Config Installer",
        Style::default().fg(app.theme.text_secondary()),
    )));

    let paragraph = Paragraph::new(lines).alignment(Alignment::Center);
    f.render_widget(paragraph, area);
}

fn render_options(f: &mut Frame, app: &App, area: Rect) {
    // Two visual rows per option (label + detail) plus a blank spacer; the
    // whole block is centered horizontally so the longest detail line
    // anchors the column.
    let mut lines: Vec<Line> = Vec::with_capacity(OPTIONS.len() * 3);

    for (idx, opt) in OPTIONS.iter().enumerate() {
        let selected = idx == app.cli_selection_index;
        let (marker, label_color, label_mod) = if selected {
            ("▶ ", app.theme.accent_primary(), Modifier::BOLD)
        } else {
            ("  ", app.theme.text_primary(), Modifier::empty())
        };

        lines.push(Line::from(vec![
            Span::styled(
                marker,
                Style::default().fg(app.theme.accent_primary()),
            ),
            Span::styled(
                opt.label,
                Style::default().fg(label_color).add_modifier(label_mod),
            ),
        ]));
        lines.push(Line::from(Span::styled(
            format!("    {}", opt.detail),
            Style::default().fg(app.theme.text_muted()),
        )));
        lines.push(Line::from(""));
    }

    let paragraph = Paragraph::new(lines)
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(app.theme.border()))
                .title(" Select target ")
                .title_style(Style::default().fg(app.theme.text_primary())),
        );

    f.render_widget(paragraph, area);
}

fn render_version_footer(f: &mut Frame, app: &App, area: Rect) {
    // Single trailing space keeps the version one column off the right
    // border, matching the visual rhythm of the rest of the screen.
    let text = format!("{} ", crate::fs::VERSION);
    let paragraph = Paragraph::new(Line::from(Span::styled(
        text,
        Style::default().fg(app.theme.text_muted()),
    )))
    .alignment(Alignment::Right);
    f.render_widget(paragraph, area);
}
