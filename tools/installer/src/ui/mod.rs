mod tabs;
pub mod list;
mod mcp_list;
mod plugin_list;
mod diff;
pub mod help;
mod confirm_exit;
mod layout;
mod env_input;
mod project_path;
mod installing;
mod cli_selection;
pub mod loading_screen;
mod sources;
mod source_wizard;
#[cfg(test)]
mod tests;

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

use ratatui::style::Style;
use ratatui::text::Span;

use crate::app::{App, FocusArea, Tab, View};
use crate::theme::Theme;

/// Render a source tag (e.g., " [bundled]") for multi-source display.
pub fn source_tag_span(source_name: &str, theme: &Theme) -> Span<'static> {
    Span::styled(
        format!(" [{}]", source_name),
        Style::default().fg(theme.text_muted()),
    )
}

/// Border style for a pane, brightened while that pane holds focus.
///
/// Both panes are always on screen, so the border is what tells the user
/// which one the next keystroke reaches — without it the `1`/`2`/`Tab`
/// focus switch is invisible in the content pane, whose cursor highlight
/// looks identical either way.
///
/// `View::List` is the only view that dispatches those focus keys, so it is
/// also the only one that may claim the cue. Elsewhere something else owns
/// the keyboard: an input modal draws its own accent border, which a lit
/// pane behind it would compete with, and a diff overlay draws a plain one,
/// so a lit pane there would advertise keys the view no longer answers.
pub fn pane_border_style(app: &App, pane: FocusArea) -> Style {
    let color = if app.current_view == View::List && app.focus == pane {
        app.theme.border_focused()
    } else {
        app.theme.border()
    };
    Style::default().fg(color)
}

/// Border title for a pane, prefixed with the digit that jumps to it.
///
/// The prefix is where the shortcut is taught: the status bar's `[1/2] Pane`
/// says the keys exist, but only the title says which pane each one reaches.
/// `text` is trimmed so callers can pass their already-padded title verbatim.
pub fn pane_title(pane: FocusArea, text: &str) -> String {
    format!(" [{}]-{} ", pane.shortcut(), text.trim())
}

// Spinner animation frames
pub const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

pub fn get_spinner(frame: usize) -> &'static str {
    SPINNER_FRAMES[frame % SPINNER_FRAMES.len()]
}

pub fn draw(f: &mut Frame, app: &App) {
    use ratatui::style::Style;

    // Clear entire background with theme color
    // This ensures terminal background doesn't show through
    f.render_widget(
        ratatui::widgets::Block::default()
            .style(Style::default().bg(app.theme.bg_primary()).fg(app.theme.text_primary())),
        f.area()
    );

    if render_full_screen(f, app) {
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Tabs
            Constraint::Min(0),     // Content
            Constraint::Length(3),  // Status bar
        ])
        .split(f.area());

    tabs::render(f, app, chunks[0]);
    render_content_pane(f, app, chunks[1]);
    render_status_bar(f, app, chunks[2]);

    // Painted after the status bar so a box may cover it — in a short terminal
    // the help overlay needs every row it can take, and both boxes carry their
    // own keys on their own borders.
    match app.current_view {
        View::Help => help::render(f, app, f.area()),
        View::ConfirmExit => confirm_exit::render(f, app, f.area()),
        _ => {}
    }
}

/// Draw the views that own the whole terminal, reporting whether one did.
///
/// These skip both the tab bar and the status bar: each already carries its own
/// footer, and `draw`'s three-chunk layout would crop them to make room for bars
/// they do not use. The CLI picker in particular is the first impression, and
/// borrowing the global status bar there would clutter it.
///
/// Every Sources arm repaints the list beneath its wizard step, because the step
/// is an overlay on that list rather than a screen of its own.
fn render_full_screen(f: &mut Frame, app: &App) -> bool {
    let area = f.area();
    match app.current_view {
        View::CliSelection => cli_selection::render(f, app, area),
        View::Loading => render_loading_screen(f, app),
        // The CLI `--version` probe has nothing of its own to show.
        View::Preflighting => render_preflighting_screen(f, app),
        View::Sources => sources::render(f, app, area),
        View::SourceAddType => {
            sources::render(f, app, area);
            source_wizard::render_type_select(f, app, area);
        }
        View::SourceAddUrl => {
            sources::render(f, app, area);
            source_wizard::render_text_input(f, app, area, "Git URL", "URL");
        }
        View::SourceAddBranch => {
            sources::render(f, app, area);
            source_wizard::render_text_input(f, app, area, "Git Branch (optional)", "Branch");
        }
        View::SourceAddPath => {
            sources::render(f, app, area);
            source_wizard::render_text_input(f, app, area, "Local Path", "Path");
        }
        View::SourceAddRoot => {
            sources::render(f, app, area);
            source_wizard::render_text_input(f, app, area, "Subdirectory (optional)", "Root");
        }
        View::SourceAddMapTo => {
            sources::render(f, app, area);
            source_wizard::render_map_to_select(f, app, area);
        }
        View::SourceConfirmRemove => {
            sources::render(f, app, area);
            source_wizard::render_confirm_remove(f, app, area);
        }
        View::SourceSyncing => {
            sources::render(f, app, area);
            source_wizard::render_syncing(f, app, area);
        }
        _ => return false,
    }
    true
}

/// Draw the middle chunk for whichever view is live.
///
/// Only reached for the views `render_full_screen` declined, so every arm here
/// is one that wants the tab bar above it and the status bar below.
fn render_content_pane(f: &mut Frame, app: &App, area: Rect) {
    match app.current_view {
        // Both overlays paint their box over the list rather than replacing it,
        // so all three views share a body and dismissing a box restores the
        // screen unchanged.
        View::List | View::Help | View::ConfirmExit => render_tab_list(f, app, area),
        View::Diff => diff::render(f, app, area),
        // The dialogs sit over the MCP list they were opened from, which stays
        // visible so the user can see which server they are answering for.
        View::EnvInput => {
            mcp_list::render(f, app, area);
            env_input::render(f, app, area);
        }
        View::ProjectPath => {
            mcp_list::render(f, app, area);
            project_path::render(f, app, area);
        }
        View::Installing => installing::render(f, app, area),
        View::CliSelection
        | View::Loading
        | View::Preflighting
        | View::Sources
        | View::SourceAddType
        | View::SourceAddUrl
        | View::SourceAddBranch
        | View::SourceAddPath
        | View::SourceAddRoot
        | View::SourceAddMapTo
        | View::SourceConfirmRemove
        | View::SourceSyncing => unreachable!("drawn full-screen"),
    }
}

/// Draw whichever list belongs to the current tab.
fn render_tab_list(f: &mut Frame, app: &App, area: Rect) {
    if app.tab == Tab::McpServers {
        mcp_list::render(f, app, area);
    } else if app.tab == Tab::Plugins {
        plugin_list::render(f, app, area);
    } else {
        list::render(f, app, area);
    }
}

/// The List view's status-bar help — fixed, and short enough to survive any
/// terminal width worth supporting.
///
/// It used to be five per-focus, per-tab lines running up to 102 columns. At the
/// standard 80 the `Paragraph` clipped them, and what fell off the right edge
/// was the tail of the line — including `[q] Quit`, on exactly the terminals
/// most likely to need it. The full binding list now lives behind `?`, where it
/// can scroll instead of truncate.
const LIST_HELP: &str = "[1/2] Pane  [?] Keys  [q] Quit";

/// Key hints for the current view, or `""` where the view draws its own footer
/// instead of borrowing this one.
fn status_help(app: &App) -> &'static str {
    match app.current_view {
        // CliSelection has its own version footer and skips the global
        // status bar (see early-return in `draw`); this arm exists only
        // to keep the match exhaustive.
        View::CliSelection => "",
        View::Loading => "Loading...  [q] Quit",
        View::Preflighting => "Checking CLI...  [Esc] Cancel  [q] Quit",
        View::List => LIST_HELP,
        // Both overlays name their own keys on their own frames, and the List
        // keys behind them are no longer live — repeating them here would
        // advertise a pane jump the modal will not answer.
        View::Help | View::ConfirmExit => "",
        View::Diff => "[j/k/↑/↓] Scroll  [q/Esc] Close",
        View::EnvInput => "[Enter] Submit  [Esc] Cancel  [Backspace] Delete",
        View::ProjectPath => "[Enter] Confirm  [Esc] Cancel  [Backspace] Delete",
        View::Installing => {
            if app.processing_complete {
                "[Enter/q] Close"
            } else if app.is_removing {
                "Removing..."
            } else {
                "Installing..."
            }
        }
        // Sources views have their own footer
        View::Sources | View::SourceAddType | View::SourceAddUrl
        | View::SourceAddBranch | View::SourceAddPath | View::SourceAddRoot
        | View::SourceAddMapTo | View::SourceConfirmRemove | View::SourceSyncing => "",
    }
}

fn render_status_bar(f: &mut Frame, app: &App, area: Rect) {
    use ratatui::{
        style::Style,
        text::{Line, Span},
        widgets::{Block, Borders, Paragraph},
    };

    let help_text = status_help(app);
    let status = app.status_message.as_deref().unwrap_or("");
    let version = crate::fs::VERSION;

    let spans = vec![
        Span::styled(help_text, Style::default().fg(app.theme.text_secondary())),
        Span::raw("  "),
        Span::styled(status, Style::default().fg(app.theme.warning())),
    ];

    // Render help text left-aligned, version right-aligned
    let inner_width = area.width.saturating_sub(2) as usize; // subtract border
    let left_text = Line::from(spans);
    // `Line::width` is the measure ratatui itself paints with, so the padding
    // computed from it lands the version flush right whatever the content is.
    // Counting bytes would short the padding on the arrow glyphs (three bytes,
    // one column); counting chars would overrun it on a CJK status message
    // (one char, two columns) and push the version off the edge.
    let padding = inner_width.saturating_sub(left_text.width() + layout::columns(version));

    let mut all_spans = left_text.spans;
    all_spans.push(Span::raw(" ".repeat(padding)));
    all_spans.push(Span::styled(version, Style::default().fg(app.theme.text_secondary())));

    let paragraph = Paragraph::new(Line::from(all_spans))
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(app.theme.border())));

    f.render_widget(paragraph, area);
}

fn render_loading_screen(f: &mut Frame, app: &App) {
    let cli_name = app.target_cli
        .map(|c| c.display_name().to_string())
        .unwrap_or_else(|| "Unknown".to_string());
    render_spinner_box(
        f,
        app,
        " Config Installer ",
        &format!("Loading {} configuration...", cli_name),
    );
}

fn render_preflighting_screen(f: &mut Frame, app: &App) {
    let cli_name = app.target_cli
        .map(|c| c.display_name().to_string())
        .unwrap_or_else(|| "CLI".to_string());
    render_spinner_box(
        f,
        app,
        " Preflight ",
        &format!("Checking {} availability...", cli_name),
    );
}

/// The seven-row box the spinner screens draw into, centred in `area`.
///
/// Height is fixed because the contents are fixed — a blank line, the spinner
/// row, a blank line, and the border. Width stays proportional so a long message
/// gets more room on a wide terminal instead of wrapping at a constant column.
fn spinner_box_area(area: Rect) -> Rect {
    use ratatui::layout::Constraint;

    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Length(7),
            Constraint::Percentage(40),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(40),
            Constraint::Percentage(30),
        ])
        .split(vertical[1])[1]
}

/// Centered spinner box used by both the initial scan and the CLI
/// preflight. Same layout, theme, and animation — only the title and
/// the single line of message text vary.
fn render_spinner_box(f: &mut Frame, app: &App, title: &str, message: &str) {
    use ratatui::{
        layout::Alignment,
        style::{Modifier, Style},
        text::{Line, Span},
        widgets::{Block, Borders, Paragraph},
    };

    let area = spinner_box_area(f.area());
    let spinner = get_spinner(app.animation_frame);

    let text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                spinner,
                Style::default()
                    .fg(app.theme.spinner())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(
                message.to_string(),
                Style::default().fg(app.theme.text_primary()),
            ),
        ]),
        Line::from(""),
    ];

    let widget = Paragraph::new(text)
        .style(Style::default().fg(app.theme.accent_primary()).bg(app.theme.bg_secondary()))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(app.theme.info()))
                .title(title.to_string())
                .title_style(Style::default().fg(app.theme.text_primary()))
                .style(Style::default().bg(app.theme.bg_secondary())),
        );

    f.render_widget(widget, area);
}
