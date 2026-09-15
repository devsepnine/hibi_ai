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

    // CLI selection screen takes full screen and self-contains its
    // version footer; skip the global status bar to keep the first
    // impression uncluttered.
    if app.current_view == View::CliSelection {
        cli_selection::render(f, app, f.area());
        return;
    }

    // Loading screen takes full screen
    if app.current_view == View::Loading {
        render_loading_screen(f, app);
        return;
    }

    // Preflighting (CLI `--version` probe) reuses the loading screen.
    if app.current_view == View::Preflighting {
        render_preflighting_screen(f, app);
        return;
    }

    // Sources views take full screen (like CLI selection)
    match app.current_view {
        View::Sources => {
            sources::render(f, app, f.area());
            return;
        }
        View::SourceAddType => {
            sources::render(f, app, f.area());
            source_wizard::render_type_select(f, app, f.area());
            return;
        }
        View::SourceAddUrl => {
            sources::render(f, app, f.area());
            source_wizard::render_text_input(f, app, f.area(), "Git URL", "URL");
            return;
        }
        View::SourceAddBranch => {
            sources::render(f, app, f.area());
            source_wizard::render_text_input(f, app, f.area(), "Git Branch (optional)", "Branch");
            return;
        }
        View::SourceAddPath => {
            sources::render(f, app, f.area());
            source_wizard::render_text_input(f, app, f.area(), "Local Path", "Path");
            return;
        }
        View::SourceAddRoot => {
            sources::render(f, app, f.area());
            source_wizard::render_text_input(f, app, f.area(), "Subdirectory (optional)", "Root");
            return;
        }
        View::SourceAddMapTo => {
            sources::render(f, app, f.area());
            source_wizard::render_map_to_select(f, app, f.area());
            return;
        }
        View::SourceConfirmRemove => {
            sources::render(f, app, f.area());
            source_wizard::render_confirm_remove(f, app, f.area());
            return;
        }
        View::SourceSyncing => {
            sources::render(f, app, f.area());
            source_wizard::render_syncing(f, app, f.area());
            return;
        }
        _ => {}
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

    match app.current_view {
        View::CliSelection => unreachable!(),
        View::Loading => unreachable!(),
        View::Preflighting => unreachable!(),
        // Both overlays paint their box over the list rather than replacing it,
        // so all three views share a body and dismissing a box restores the
        // screen unchanged.
        View::List | View::Help | View::ConfirmExit => render_list_pane(f, app, chunks[1]),
        View::Diff => {
            diff::render(f, app, chunks[1]);
        }
        View::EnvInput => {
            // Show MCP list in background, then overlay env input dialog
            mcp_list::render(f, app, chunks[1]);
            env_input::render(f, app, chunks[1]);
        }
        View::ProjectPath => {
            // Show MCP list in background, then overlay project path dialog
            mcp_list::render(f, app, chunks[1]);
            project_path::render(f, app, chunks[1]);
        }
        View::Installing => {
            installing::render(f, app, chunks[1]);
        }
        // Sources views are handled above (full-screen early return)
        View::Sources | View::SourceAddType | View::SourceAddUrl
        | View::SourceAddBranch | View::SourceAddPath | View::SourceAddRoot
        | View::SourceAddMapTo | View::SourceConfirmRemove | View::SourceSyncing => unreachable!(),
    }

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

/// Draw whichever list belongs to the current tab.
fn render_list_pane(f: &mut Frame, app: &App, area: Rect) {
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
    // Columns, not bytes: the arrow glyphs in these strings are three bytes
    // wide and one column, so byte length would push the version left of the
    // right edge by one column per arrow.
    let left_len: usize = left_text
        .spans
        .iter()
        .map(|s| s.content.chars().count())
        .sum();
    let padding = inner_width.saturating_sub(left_len + version.len());

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

/// Centered spinner box used by both the initial scan and the CLI
/// preflight. Same layout, theme, and animation — only the title and
/// the single line of message text vary.
fn render_spinner_box(f: &mut Frame, app: &App, title: &str, message: &str) {
    use ratatui::{
        layout::{Alignment, Constraint},
        style::{Modifier, Style},
        text::{Line, Span},
        widgets::{Block, Borders, Paragraph},
    };

    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Length(7),
            Constraint::Percentage(40),
        ])
        .split(f.area());

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(40),
            Constraint::Percentage(30),
        ])
        .split(vertical[1]);

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

    f.render_widget(widget, horizontal[1]);
}
