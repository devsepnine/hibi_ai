use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use super::layout;
use crate::app::App;
use crate::component::InstallStatus;

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

struct RemovalLayout {
    dialog: Rect,
    warning: Rect,
    list: Rect,
    names: Vec<String>,
}

impl RemovalLayout {
    fn scroll_limit(&self) -> usize {
        self.names.len().saturating_sub(self.list.height.into())
    }
}

fn removal_layout(app: &App, area: Rect) -> RemovalLayout {
    const WIDTH: u16 = 78;
    const MAX_ITEM_ROWS: usize = 18;

    let content_width = WIDTH.min(area.width).saturating_sub(2);
    let names: Vec<_> = app
        .pending_removal_items()
        .iter()
        .flat_map(|item| {
            let name = item.display_name();
            let text = if let Some(status @ (InstallStatus::Modified | InstallStatus::External)) =
                &item.status
            {
                format!("{}  [{}]", name, status.display())
            } else {
                name
            };
            layout::wrap_line(&text, content_width)
        })
        .collect();
    let item_rows = names.len().clamp(1, MAX_ITEM_ROWS);
    let dialog = layout::centered(area, WIDTH, (item_rows + 4) as u16);
    let [warning, list] = Layout::vertical([Constraint::Length(2), Constraint::Min(0)])
        .areas(Block::bordered().inner(dialog));
    RemovalLayout {
        dialog,
        warning,
        list,
        names,
    }
}

pub(crate) fn removal_scroll_limit(app: &App, area: Rect) -> usize {
    removal_layout(app, area).scroll_limit()
}

pub fn render_remove(f: &mut Frame, app: &App, area: Rect) {
    let count = app.pending_removal_items().len();
    let label = if count == 1 { "item" } else { "items" };
    let layout = removal_layout(app, area);
    let max_scroll = layout.scroll_limit();
    let keys = if max_scroll > 0 {
        " [y] Remove [Esc] Cancel [↑/↓] Scroll "
    } else {
        " [y] Remove [Esc] Cancel "
    };
    let block = Block::bordered()
        .title(format!(" Remove {count} selected {label}? "))
        .title_bottom(keys)
        .style(Style::default().bg(app.theme.bg_secondary()))
        .border_style(Style::default().fg(app.theme.warning()));
    f.render_widget(Clear, layout.dialog);
    f.render_widget(block, layout.dialog);
    f.render_widget(
        Paragraph::new("This action cannot be undone.")
            .style(Style::default().fg(app.theme.error()))
            .wrap(ratatui::widgets::Wrap { trim: true }),
        layout.warning,
    );

    let lines = layout
        .names
        .into_iter()
        .skip(app.removal_scroll.min(max_scroll))
        .take(layout.list.height.into())
        .map(Line::raw)
        .collect::<Vec<_>>();
    f.render_widget(Paragraph::new(lines), layout.list);
}
