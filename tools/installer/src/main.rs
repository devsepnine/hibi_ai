mod app;
mod component;
mod mcp;
mod plugin;
mod fs;
mod source;
mod tree;
mod ui;
mod theme;
mod loading;
mod process_exec;

use std::io;
use std::thread;
use anyhow::Result;
use std::time::Duration;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, Clear, ClearType},
    cursor::MoveTo,
};
use ratatui::{backend::CrosstermBackend, Terminal};

use app::App;
use loading::{ProcessingChannels, RefreshResult};

fn main() -> Result<()> {
    // Handle CLI flags
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_help();
        return Ok(());
    }

    if args.iter().any(|a| a == "--version" || a == "-v") {
        println!("hibi {} (Config Installer)", fs::VERSION);
        return Ok(());
    }

    if args.iter().any(|a| a == "--sync") {
        return run_sync();
    }

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, Clear(ClearType::All), MoveTo(0, 0))?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app in background thread while show loading animation
    use std::sync::{Arc, Mutex};

    let app_result: Arc<Mutex<Option<Result<App>>>> = Arc::new(Mutex::new(None));
    let app_result_clone = Arc::clone(&app_result);

    // Spawn background thread to create app
    thread::spawn(move || {
        let result = App::new();
        *app_result_clone.lock().unwrap() = Some(result);
    });

    // Show loading animation while app is being created
    let mut frame_idx = 0;

    loop {
        terminal.draw(|f| ui::loading_screen::draw(f, frame_idx))?;

        // Check if app is ready
        let result_lock = app_result.lock().unwrap();
        if result_lock.is_some() {
            break;
        }
        drop(result_lock);

        // Update animation frame
        frame_idx += 1;
        std::thread::sleep(Duration::from_millis(100));
    }

    // Get app from result
    let mut app = app_result.lock().unwrap().take().unwrap()?;
    let result = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = result {
        eprintln!("Error: {err:?}");
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()>
where
    <B as ratatui::backend::Backend>::Error: Send + Sync + 'static,
{
    let mut channels = ProcessingChannels::new();

    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        match app.current_view {
            app::View::Installing => loading::handle_installing_view(app, &mut channels)?,
            app::View::Loading => loading::handle_loading_view(app, &channels.refresh_rx)?,
            app::View::SourceSyncing => handle_source_syncing(app)?,
            _ => {
                if let Some((code, modifiers)) = read_key_press()? {
                    dispatch_key(app, code, modifiers, &channels.refresh_tx)?;
                }
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

/// Read a single key press, filtering out release events.
fn read_key_press() -> Result<Option<(KeyCode, KeyModifiers)>> {
    if let Event::Key(key) = event::read()? {
        if key.kind != KeyEventKind::Release {
            return Ok(Some((key.code, key.modifiers)));
        }
    }
    Ok(None)
}

/// Dispatch a key press to the appropriate handler based on current view.
fn dispatch_key(
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
fn handle_source_syncing(app: &mut App) -> Result<()> {
    use crossterm::event::poll;
    if poll(Duration::from_millis(100))? {
        if let Some((code, _)) = read_key_press()? {
            if code == KeyCode::Char('q') {
                app.source_sync_rx = None;
                app.source_sync_status = Some("Cancelled".to_string());
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
        // Tab navigation: Tab = next, Shift+Tab = prev
        KeyCode::Tab => {
            if modifiers.contains(KeyModifiers::SHIFT) {
                app.prev_tab();
            } else {
                app.next_tab();
            }
        }
        KeyCode::BackTab => app.prev_tab(), // Some terminals send BackTab for Shift+Tab
        // h/l keys and Left/Right arrow keys: folder navigation
        KeyCode::Char('h') | KeyCode::Left => {
            if app.is_cursor_on_folder() && app.is_current_folder_expanded() {
                // On expanded folder: collapse it
                app.collapse_folder();
            } else {
                // On collapsed folder or file: collapse parent folder
                app.collapse_parent_folder();
            }
        }
        KeyCode::Char('l') | KeyCode::Right => {
            if app.is_cursor_on_folder() {
                // On folder: expand it
                app.expand_folder();
            }
            // On file: do nothing
        }
        // Number keys 1-0 for direct tab selection
        KeyCode::Char('1') => app.set_tab(0),  // Agents
        KeyCode::Char('2') => app.set_tab(1),  // Commands
        KeyCode::Char('3') => app.set_tab(2),  // Contexts
        KeyCode::Char('4') => app.set_tab(3),  // Rules
        KeyCode::Char('5') => app.set_tab(4),  // Skills
        KeyCode::Char('6') => app.set_tab(5),  // Hooks
        KeyCode::Char('7') => app.set_tab(6),  // OutputStyles
        KeyCode::Char('8') => app.set_tab(7),  // Statusline
        KeyCode::Char('9') => app.set_tab(8),  // Config
        KeyCode::Char('0') => app.set_tab(9),  // MCP
        KeyCode::Char('-') => app.set_tab(10), // Plugins
        // List navigation
        KeyCode::Down | KeyCode::Char('j') => app.next_item(),
        KeyCode::Up | KeyCode::Char('k') => app.prev_item(),
        // Selection
        KeyCode::Char(' ') => app.toggle_selected(),
        KeyCode::Char('a') => app.select_all(),
        KeyCode::Char('n') => app.deselect_all(),
        // Actions
        KeyCode::Enter => {
            if app.is_cursor_on_folder() {
                // On folder: toggle expand/collapse
                app.toggle_folder_expand();
            } else {
                // On file: show diff
                app.show_diff()?;
            }
        }
        KeyCode::Char('d') => app.show_diff()?,
        KeyCode::Char('i') => app.install_selected()?,
        KeyCode::Char('r') => app.remove_selected()?,
        KeyCode::Char('s') => {
            // 's' sets default for OutputStyles or Statusline tabs
            if app.tab == app::Tab::OutputStyles {
                app.set_default_style()?;
            } else if app.tab == app::Tab::Statusline {
                app.set_statusline()?;
            }
        }
        KeyCode::Char('u') => {
            // 'u' unsets default for OutputStyles or Statusline tabs
            if app.tab == app::Tab::OutputStyles {
                app.unset_default_style()?;
            } else if app.tab == app::Tab::Statusline {
                app.unset_statusline()?;
            }
        }
        KeyCode::Char('o') => {
            // 'o' toggles MCP scope (user/local) when on MCP tab
            if app.tab == app::Tab::McpServers {
                app.toggle_mcp_scope();
            }
        }
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

fn print_help() {
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
fn run_sync() -> Result<()> {
    let app = App::new()?;

    let git_count = app.sources.iter().filter(|s| s.kind == source::SourceKind::Git).count();
    let has_bundled_git = app.bundled_git_root.is_some();

    if git_count == 0 && !has_bundled_git {
        println!("No git sources configured. Add sources to ~/.hibi/sources.yaml");
        return Ok(());
    }

    if let Some(git_root) = &app.bundled_git_root {
        println!("Pulling bundled source...");
        match source::git::pull_local_repo(git_root) {
            Ok(()) => println!("  bundled: updated"),
            Err(e) => println!("  bundled: failed ({})", e),
        }
    }

    if git_count > 0 {
        println!("Syncing {} git source(s)...", git_count);
        let (_updated, summaries) = source::update_git_sources(&app.sources);
        for summary in &summaries {
            println!("{}", summary);
        }
    }

    println!("\nRun `hibi` to install changes.");
    Ok(())
}
