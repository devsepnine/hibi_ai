use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};

use crate::app::{self, App};
use crate::loading::{self, RefreshResult};
use crate::source;
use crate::fs;

/// Read a single key press, filtering out release events.
pub(crate) fn read_key_press() -> Result<Option<(KeyCode, KeyModifiers)>> {
    if let Event::Key(key) = event::read()? {
        if key.kind != KeyEventKind::Release {
            return Ok(Some((key.code, key.modifiers)));
        }
    }
    Ok(None)
}

/// Dispatch a key press to the appropriate handler based on current view.
pub(crate) fn dispatch_key(
    app: &mut App,
    code: KeyCode,
    modifiers: KeyModifiers,
    refresh_tx: &std::sync::mpsc::Sender<Result<RefreshResult>>,
) -> Result<()> {
    match app.current_view {
        app::View::CliSelection => handle_cli_selection(app, code, refresh_tx),
        app::View::EnvInput => handle_env_input(app, code),
        app::View::ProjectPath => { handle_project_path_input(app, code); Ok(()) }
        app::View::List => handle_list_input(app, code, modifiers),
        app::View::Diff => handle_diff_input(app, code),
        app::View::Sources => app.handle_sources_key(code),
        app::View::SourceAddType => app.handle_source_type_key(code),
        app::View::SourceAddMapTo => app.handle_source_map_to_key(code),
        app::View::SourceConfirmRemove => app.handle_source_confirm_key(code),
        app::View::SourceAddUrl | app::View::SourceAddBranch
        | app::View::SourceAddPath | app::View::SourceAddRoot => app.handle_source_input_key(code),
        _ => Ok(()),
    }
}

/// Handle the SourceSyncing view with non-blocking poll.
pub(crate) fn handle_source_syncing(app: &mut App) -> Result<()> {
    use crossterm::event::poll;
    if poll(Duration::from_millis(100))? {
        if let Some((code, _)) = read_key_press()? {
            if code == KeyCode::Char('q') {
                if let Some(tx) = app.source_sync_cancel_tx.take() {
                    let _ = tx.send(());
                }
                app.source_sync_rx = None;
                app.source_sync_status = Some(app::SyncStatus::Error("Cancelled".to_string()));
                app.current_view = app::View::Sources;
            }
        }
    }
    app.tick();
    app.check_source_sync();
    Ok(())
}

fn handle_list_input(app: &mut App, key: KeyCode, modifiers: KeyModifiers) -> Result<()> {
    match key {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Char('t') => {
            app.theme.toggle();
            app.status_message = Some(format!("Theme: {}", app.theme.mode().name()));
        }
        KeyCode::Tab => {
            if modifiers.contains(KeyModifiers::SHIFT) { app.prev_tab(); } else { app.next_tab(); }
        }
        KeyCode::BackTab => app.prev_tab(),
        KeyCode::Char('h') | KeyCode::Left => handle_folder_collapse(app),
        KeyCode::Char('l') | KeyCode::Right => { if app.is_cursor_on_folder() { app.expand_folder(); } }
        KeyCode::Char('1') => app.set_tab(0),
        KeyCode::Char('2') => app.set_tab(1),
        KeyCode::Char('3') => app.set_tab(2),
        KeyCode::Char('4') => app.set_tab(3),
        KeyCode::Char('5') => app.set_tab(4),
        KeyCode::Char('6') => app.set_tab(5),
        KeyCode::Char('7') => app.set_tab(6),
        KeyCode::Char('8') => app.set_tab(7),
        KeyCode::Char('9') => app.set_tab(8),
        KeyCode::Char('0') => app.set_tab(9),
        KeyCode::Char('-') => app.set_tab(10),
        KeyCode::Down | KeyCode::Char('j') => app.next_item(),
        KeyCode::Up | KeyCode::Char('k') => app.prev_item(),
        KeyCode::Char(' ') => app.toggle_selected(),
        KeyCode::Char('a') => app.select_all(),
        KeyCode::Char('n') => app.deselect_all(),
        KeyCode::Enter => handle_enter(app)?,
        KeyCode::Char('d') => app.show_diff()?,
        KeyCode::Char('i') => app.install_selected()?,
        KeyCode::Char('r') => app.remove_selected()?,
        KeyCode::Char('s') | KeyCode::Char('u') => handle_default_toggle(app, key)?,
        KeyCode::Char('o') => { if app.tab == app::Tab::McpServers { app.toggle_mcp_scope(); } }
        _ => {}
    }
    Ok(())
}

fn handle_folder_collapse(app: &mut App) {
    if app.is_cursor_on_folder() && app.is_current_folder_expanded() {
        app.collapse_folder();
    } else {
        app.collapse_parent_folder();
    }
}

fn handle_enter(app: &mut App) -> Result<()> {
    if app.is_cursor_on_folder() { app.toggle_folder_expand(); } else { app.show_diff()?; }
    Ok(())
}

fn handle_default_toggle(app: &mut App, key: KeyCode) -> Result<()> {
    match (key, app.tab) {
        (KeyCode::Char('s'), app::Tab::OutputStyles) => app.set_default_style()?,
        (KeyCode::Char('s'), app::Tab::Statusline) => app.set_statusline()?,
        (KeyCode::Char('u'), app::Tab::OutputStyles) => app.unset_default_style()?,
        (KeyCode::Char('u'), app::Tab::Statusline) => app.unset_statusline()?,
        _ => {}
    }
    Ok(())
}

fn handle_diff_input(app: &mut App, key: KeyCode) -> Result<()> {
    match key {
        KeyCode::Char('q') | KeyCode::Esc => app.close_diff(),
        KeyCode::Down | KeyCode::Char('j') => app.scroll_diff_down(),
        KeyCode::Up | KeyCode::Char('k') => app.scroll_diff_up(),
        _ => {}
    }
    Ok(())
}

fn handle_cli_selection(
    app: &mut App,
    key: KeyCode,
    refresh_tx: &std::sync::mpsc::Sender<Result<RefreshResult>>,
) -> Result<()> {
    match key {
        KeyCode::Char('1') => {
            app.select_cli(app::TargetCli::Claude)?;
            loading::start_loading_thread(app, refresh_tx);
        }
        KeyCode::Char('2') => {
            app.select_cli(app::TargetCli::Codex)?;
            loading::start_loading_thread(app, refresh_tx);
        }
        KeyCode::Char('s') => {
            app.current_view = app::View::Sources;
        }
        KeyCode::Char('q') => app.should_quit = true,
        _ => {}
    }
    Ok(())
}

fn handle_env_input(app: &mut App, key: KeyCode) -> Result<()> {
    match key {
        KeyCode::Esc => app.env_input_cancel(),
        KeyCode::Enter => app.env_input_submit()?,
        KeyCode::Backspace => app.env_input_backspace(),
        KeyCode::Char(c) => app.env_input_char(c),
        _ => {}
    }
    Ok(())
}

fn handle_project_path_input(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Esc => app.project_path_cancel(),
        KeyCode::Enter => app.project_path_submit(),
        KeyCode::Backspace => app.project_path_backspace(),
        KeyCode::Char(c) => app.project_path_char(c),
        _ => {}
    }
}

pub(crate) fn print_help() {
    println!("hibi {} - Claude Code Config Installer", fs::VERSION);
    println!();
    println!("Usage: hibi [OPTIONS]");
    println!();
    println!("Options:");
    println!("  -h, --help       Show this help message");
    println!("  -v, --version    Show version");
    println!("      --sync       Sync git sources without TUI");
    println!();
    println!("Run without options to launch the interactive installer.");
}

/// `hibi --sync`: fetch latest from git sources and print summary, no TUI.
pub(crate) fn run_sync() -> Result<()> {
    let source_dir = source::find_source_dir()?;
    let bundled_git_root = source::git::find_git_root(&source_dir);

    let has_git = source::config::load_config()
        .map(|(e, _)| e.iter().any(|en| matches!(en, source::SourceEntry::Git { .. })))
        .unwrap_or(false);

    if !has_git && bundled_git_root.is_none() {
        println!("No git sources configured. Add sources to ~/.hibi/sources.yaml");
        return Ok(());
    }

    println!("Syncing sources...");
    let (_, dummy_rx) = std::sync::mpsc::channel::<()>();
    let report = source::sync_all_sources(bundled_git_root.as_deref(), &source_dir, &dummy_rx);

    for s in &report.summaries {
        println!("{}", s);
    }
    println!("\nRun `hibi` to install changes.");
    Ok(())
}
