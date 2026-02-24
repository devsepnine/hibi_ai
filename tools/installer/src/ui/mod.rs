mod tabs;
pub mod list;
mod mcp_list;
mod plugin_list;
mod diff;
mod env_input;
mod project_path;
mod installing;
mod cli_selection;

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

use crate::app::{App, Tab, View};

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

    // CLI selection screen takes full screen
    if app.current_view == View::CliSelection {
        cli_selection::render(f, app, f.area());
        return;
    }

    // Loading screen takes full screen
    if app.current_view == View::Loading {
        render_loading_screen(f, app);
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

    match app.current_view {
        View::CliSelection => unreachable!(),
        View::Loading => unreachable!(),
        View::List => {
            if app.tab == Tab::McpServers {
                mcp_list::render(f, app, chunks[1]);
            } else if app.tab == Tab::Plugins {
                plugin_list::render(f, app, chunks[1]);
            } else {
                list::render(f, app, chunks[1]);
            }
        }
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
    }

    render_status_bar(f, app, chunks[2]);
}

fn render_status_bar(f: &mut Frame, app: &App, area: Rect) {
    use ratatui::{
        style::Style,
        text::{Line, Span},
        widgets::{Block, Borders, Paragraph},
    };

    let help_text = match app.current_view {
        View::CliSelection => "[1/2] Select  [q] Quit",
        View::Loading => "Loading...  [q] Quit",
        View::List => {
            if app.tab == Tab::McpServers {
                "[Space] Toggle  [i] Install  [r] Remove  [o] Scope  [t] Theme  [Tab/1-0,-] Switch  [q] Quit"
            } else if app.tab == Tab::Plugins {
                "[Space] Toggle  [i] Install  [r] Remove  [t] Theme  [Tab/1-0,-] Switch  [q] Quit"
            } else if app.tab == Tab::OutputStyles || app.tab == Tab::Statusline {
                "[Space] Toggle  [i] Install  [r] Remove  [d] Diff  [s] Set [u] Unset  [t] Theme  [Tab/1-0,-] Switch  [q] Quit"
            } else {
                "[Space] Toggle  [i] Install  [r] Remove  [d] Diff  [h/l/←/→] Folder  [t] Theme  [Tab/1-0,-] Switch  [q] Quit"
            }
        }
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
    };

    let status = app.status_message.as_deref().unwrap_or("");

    let spans = vec![
        Span::styled(help_text, Style::default().fg(app.theme.text_secondary())),
        Span::raw("  "),
        Span::styled(status, Style::default().fg(app.theme.warning())),
    ];

    let paragraph = Paragraph::new(Line::from(spans))
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(app.theme.border())));

    f.render_widget(paragraph, area);
}

fn render_loading_screen(f: &mut Frame, app: &App) {
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
    let cli_name = app.target_cli
        .map(|c| c.display_name().to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    let loading_text = vec![
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
                format!("Loading {} configuration...", cli_name),
                Style::default().fg(app.theme.text_primary()),
            ),
        ]),
        Line::from(""),
    ];

    let loading = Paragraph::new(loading_text)
        .style(Style::default().fg(app.theme.accent_primary()).bg(app.theme.bg_secondary()))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(app.theme.info()))
                .title(" Config Installer ")
                .title_style(Style::default().fg(app.theme.text_primary()))
                .style(Style::default().bg(app.theme.bg_secondary())),
        );

    f.render_widget(loading, horizontal[1]);
}
