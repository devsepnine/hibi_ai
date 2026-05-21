use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Tabs as RataTabs},
    Frame,
};

use crate::app::{App, FocusArea};

/// Glyphs shown when one or more tabs are scrolled off-screen.
const LEFT_INDICATOR: &str = "‹";
const RIGHT_INDICATOR: &str = "›";

/// Display width of `‹`/`›`. Both are 1-column single-glyph chars.
const INDICATOR_WIDTH: usize = 1;

/// Width of the divider `ratatui::Tabs` inserts between titles (" │ ").
const DIVIDER_WIDTH: usize = 3;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    // Titles are just the display names now — keyboard nav has replaced the
    // legacy 1-0/- direct shortcuts, so there's no reason to spend columns
    // on number prefixes.
    let titles: Vec<String> = app.available_tabs
        .iter()
        .map(|tab| tab.display_name().to_string())
        .collect();

    let title = if let Some(cli) = app.target_cli {
        format!(" {} Config Installer ({}) ", cli.display_name(), app.theme.mode().name())
    } else {
        String::from(" Config Installer ")
    };

    let selected_idx = app.available_tabs
        .iter()
        .position(|t| *t == app.tab)
        .unwrap_or(0);

    // Inner area excludes the left/right border chars (2 columns total).
    let inner_width = (area.width as usize).saturating_sub(2);

    let (visible_titles, visible_selected) =
        build_visible_tabs(&titles, selected_idx, inner_width);

    // Focus styling: when the user has the tab bar focused, brighten the
    // border and bold/underline the selected title so it's obvious which
    // pane the next keystroke will affect. Without this cue the focus
    // toggle would be invisible.
    let focused = app.focus == FocusArea::Tabs;
    let border_color = if focused { app.theme.accent_primary() } else { app.theme.border() };
    let highlight_mod = if focused { Modifier::BOLD | Modifier::UNDERLINED } else { Modifier::BOLD };

    let tabs = RataTabs::new(visible_titles.into_iter().map(Line::from).collect::<Vec<_>>())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color))
                .title(title)
                .title_style(Style::default().fg(app.theme.text_primary())),
        )
        .select(visible_selected)
        .style(Style::default().fg(app.theme.text_primary()))
        .highlight_style(
            Style::default()
                .fg(app.theme.accent_primary())
                .add_modifier(highlight_mod),
        );

    f.render_widget(tabs, area);
}

/// Pick a slice of `titles` that keeps `selected_idx` visible inside
/// `available_width` columns. If the slice is narrower than the full list,
/// inserts `‹`/`›` markers on the truncated side so the user knows tabs
/// are hidden off-screen.
///
/// Returns the slice (with any inserted markers) and the adjusted index of
/// the selected tab within that slice.
fn build_visible_tabs(
    titles: &[String],
    selected_idx: usize,
    available_width: usize,
) -> (Vec<String>, usize) {
    if titles.is_empty() {
        return (Vec::new(), 0);
    }

    // Fast path: everything fits, render as-is.
    let total_width: usize = titles.iter().map(|t| t.len()).sum::<usize>()
        + titles.len().saturating_sub(1) * DIVIDER_WIDTH;
    if total_width <= available_width {
        return (titles.to_vec(), selected_idx);
    }

    // Reserve budget for indicators on both sides up front. If the final
    // window ends up flush against an edge, the unused slot just becomes
    // a few columns of slack — cheaper than retrying the layout.
    let indicator_cost = (INDICATOR_WIDTH + DIVIDER_WIDTH) * 2;
    let usable = available_width.saturating_sub(indicator_cost);

    let mut start = selected_idx;
    let mut end = selected_idx + 1;
    let mut used = titles[selected_idx].len().min(usable);

    // Expand alternately right then left while a neighbor still fits.
    loop {
        let mut grew = false;

        if end < titles.len() {
            let cost = DIVIDER_WIDTH + titles[end].len();
            if used + cost <= usable {
                used += cost;
                end += 1;
                grew = true;
            }
        }
        if start > 0 {
            let cost = DIVIDER_WIDTH + titles[start - 1].len();
            if used + cost <= usable {
                used += cost;
                start -= 1;
                grew = true;
            }
        }

        if !grew {
            break;
        }
    }

    let has_left = start > 0;
    let has_right = end < titles.len();

    let mut visible: Vec<String> = titles[start..end].to_vec();
    let mut visible_selected = selected_idx - start;

    if has_left {
        visible.insert(0, LEFT_INDICATOR.to_string());
        visible_selected += 1;
    }
    if has_right {
        visible.push(RIGHT_INDICATOR.to_string());
    }

    (visible, visible_selected)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn titles(names: &[&str]) -> Vec<String> {
        names.iter().map(|s| s.to_string()).collect()
    }

    fn claude_titles() -> Vec<String> {
        titles(&[
            "Agents", "Commands", "Contexts", "Rules", "Skills",
            "Hooks", "Styles", "Statusline", "Config", "MCP", "Plugins",
        ])
    }

    #[test]
    fn all_tabs_fit_returns_full_slice() {
        let t = titles(&["A", "B", "C"]);
        let (out, sel) = build_visible_tabs(&t, 1, 80);
        assert_eq!(out, t);
        assert_eq!(sel, 1);
    }

    #[test]
    fn empty_titles_short_circuit() {
        let (out, sel) = build_visible_tabs(&[], 0, 20);
        assert!(out.is_empty());
        assert_eq!(sel, 0);
    }

    #[test]
    fn narrow_width_at_start_shows_right_indicator() {
        // 11 tabs matching the Claude layout — when the cursor sits at
        // index 0 we keep tab 0 visible and surface a trailing `›` for
        // the hidden tail.
        let t = claude_titles();
        let (out, sel) = build_visible_tabs(&t, 0, 30);
        assert_eq!(out.first().unwrap(), "Agents");
        assert_eq!(out.last().unwrap(), RIGHT_INDICATOR);
        assert_eq!(sel, 0);
        assert_ne!(out.first().unwrap(), LEFT_INDICATOR);
    }

    #[test]
    fn narrow_width_at_end_shows_left_indicator() {
        let t = claude_titles();
        let last = t.len() - 1;
        let (out, sel) = build_visible_tabs(&t, last, 30);
        assert_eq!(out.first().unwrap(), LEFT_INDICATOR);
        assert_eq!(out.last().unwrap(), "Plugins");
        assert_eq!(sel, out.len() - 1);
    }

    #[test]
    fn narrow_width_in_middle_shows_both_indicators() {
        let t = claude_titles();
        let (out, sel) = build_visible_tabs(&t, 5, 30);
        assert_eq!(out.first().unwrap(), LEFT_INDICATOR);
        assert_eq!(out.last().unwrap(), RIGHT_INDICATOR);
        // Selected ("Hooks") sits inside the visible slice, offset by the
        // leading indicator.
        assert_eq!(out[sel], "Hooks");
    }

    #[test]
    fn extremely_narrow_keeps_selected_visible() {
        // Budget collapses below a single divider+title — the selected tab
        // must still render so the user never loses their cursor.
        let t = titles(&["Agents", "Commands", "Contexts", "Rules", "Skills"]);
        let (out, sel) = build_visible_tabs(&t, 2, 10);
        assert!(out.contains(&"Contexts".to_string()));
        assert_eq!(out[sel], "Contexts");
    }
}
