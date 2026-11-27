use crate::app::{App, FilterChoice, HiddenVisibility};
use ratatui::{
    Frame,
    style::Style,
    widgets::{Block, Clear, List, ListState},
};

pub(super) fn render(frame: &mut Frame, app: &App, cursor: FilterChoice) {
    let choices = FilterChoice::choices();
    let mut rows = Vec::new();
    let mut selected = 0;
    for choice in choices {
        if choice == FilterChoice::FIRST {
            rows.push("Status".to_owned());
        } else if choice == FilterChoice::Visibility(HiddenVisibility::Exclude) {
            rows.push("Visibility".to_owned());
        }
        if choice == cursor {
            selected = rows.len();
        }
        rows.push(match choice {
            FilterChoice::Status(filter) => {
                let mark = if app.component_filter.contains(filter) {
                    "x"
                } else {
                    " "
                };
                format!("[{mark}] {}", filter.display())
            }
            FilterChoice::Visibility(mode) => {
                let label = match mode {
                    HiddenVisibility::Exclude => "Visible only",
                    HiddenVisibility::Include => "Include hidden",
                    HiddenVisibility::Only => "Hidden only",
                };
                let mark = if app.hidden_visibility == mode {
                    "*"
                } else {
                    " "
                };
                format!("({mark}) {label}")
            }
        });
    }
    let area = super::layout::centered(frame.area(), 46, rows.len() as u16 + 2);
    let block = Block::bordered()
        .title(" Filters ")
        .title_bottom(" ↑↓ Move  Space/Enter Toggle  Esc Close ")
        .style(
            Style::default()
                .fg(app.theme.text_primary())
                .bg(app.theme.bg_secondary()),
        )
        .border_style(Style::default().fg(app.theme.accent_primary()));
    frame.render_widget(Clear, area);
    let list = List::new(rows)
        .block(block)
        .highlight_symbol("> ")
        .highlight_style(Style::default().fg(app.theme.accent_primary()).bold());
    frame.render_stateful_widget(
        list,
        area,
        &mut ListState::default().with_selected(Some(selected)),
    );
}
