use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use crate::app::App;
use crate::plugin::PluginStatus;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    if app.plugins.is_empty() {
        let empty = List::new(vec![ListItem::new(Line::from(vec![Span::styled(
            "No plugins found. Create plugins/plugins.yaml to add plugins.",
            Style::default().fg(app.theme.text_muted()),
        )]))])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(app.theme.border()))
                .title(" Plugins ")
                .title_style(Style::default().fg(app.theme.text_primary())),
        );
        f.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = app
        .plugins
        .iter()
        .map(|p| {
            let checkbox = if p.selected {
                "[x]"
            } else if p.status == PluginStatus::Installed {
                "[*]"
            } else {
                "[ ]"
            };

            let status_style = match p.status {
                PluginStatus::Installed => Style::default().fg(app.theme.success()),
                PluginStatus::NotInstalled => Style::default().fg(app.theme.text_muted()),
            };

            // First line: checkbox, name, status
            let line1 = Line::from(vec![
                Span::raw(format!("{} ", checkbox)),
                Span::styled(
                    format!("{:<24}", p.def.name),
                    Style::default().fg(app.theme.text_primary()),
                ),
                Span::styled(
                    format!("({:^13})", p.status.display()),
                    status_style,
                ),
            ]);

            // Second line: repo and comment (indented)
            let short_repo = p.short_repo();
            let comment = p.def.comment.as_deref().unwrap_or("");

            let mut line2_spans = vec![
                Span::raw("    "),
                Span::styled(
                    short_repo.clone(),
                    Style::default().fg(app.theme.accent_secondary()),
                ),
            ];

            if !comment.is_empty() {
                line2_spans.push(Span::styled(
                    format!(" # {}", comment),
                    Style::default().fg(app.theme.text_secondary()),
                ));
            }

            let line2 = Line::from(line2_spans);

            ListItem::new(vec![line1, line2])
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(app.theme.border()))
                .title(" Plugins ")
                .title_style(Style::default().fg(app.theme.text_primary())),
        )
        .highlight_style(
            Style::default()
                .bg(app.theme.selection_bg())
                .fg(app.theme.selection_fg())
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    let mut state = ListState::default();
    state.select(Some(app.plugin_index));

    f.render_stateful_widget(list, area, &mut state);
}
