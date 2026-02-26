mod app;
mod component;
mod mcp;
mod plugin;
mod fs;
mod tree;
mod ui;
mod theme;

use std::io;
use std::thread;
use anyhow::Result;
use std::time::Duration;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers, poll},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, Clear, ClearType},
    cursor::MoveTo,
};
use ratatui::{backend::CrosstermBackend, Terminal};

use app::App;

fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, Clear(ClearType::All), MoveTo(0, 0))?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app in background thread while showing loading animation
    use std::sync::{Arc, Mutex};
    use std::thread;

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
        terminal.draw(|f| {
            use ratatui::{
                layout::{Alignment, Constraint, Direction, Layout},
                style::{Color, Style},
                widgets::{Block, Borders, Paragraph},
            };

            let area = f.area();

            // Center vertically
            let vertical = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(40),
                    Constraint::Length(7),
                    Constraint::Percentage(40),
                ])
                .split(area);

            // Center horizontally
            let horizontal = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(30),
                    Constraint::Percentage(40),
                    Constraint::Percentage(30),
                ])
                .split(vertical[1]);

            let spinner = ui::get_spinner(frame_idx);
            let loading_text = format!(
                "\n  {}  Loading...\n\n  Scanning components",
                spinner
            );

            let loading = Paragraph::new(loading_text)
                .style(Style::default().fg(Color::Cyan))
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::DarkGray))
                        .title(" Config Installer ")
                        .title_style(Style::default().fg(Color::White)),
                );

            f.render_widget(loading, horizontal[1]);
        })?;

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

type RefreshResult = (Vec<component::Component>, Vec<mcp::McpServer>, Vec<plugin::Plugin>);

/// Bundles all channels and state for async process management.
struct ProcessingChannels {
    process_tx: std::sync::mpsc::Sender<Result<String>>,
    process_rx: std::sync::mpsc::Receiver<Result<String>>,
    cancel_tx: std::sync::mpsc::Sender<()>,
    cancel_rx: std::sync::mpsc::Receiver<()>,
    current_cancel_tx: std::sync::mpsc::Sender<()>,
    processing_active: bool,
    refresh_tx: std::sync::mpsc::Sender<Result<RefreshResult>>,
    refresh_rx: std::sync::mpsc::Receiver<Result<RefreshResult>>,
}

impl ProcessingChannels {
    fn new() -> Self {
        use std::sync::mpsc;
        let (process_tx, process_rx) = mpsc::channel::<Result<String>>();
        let (cancel_tx, cancel_rx) = mpsc::channel::<()>();
        let current_cancel_tx = cancel_tx.clone();
        let (refresh_tx, refresh_rx) = mpsc::channel::<Result<RefreshResult>>();

        Self {
            process_tx,
            process_rx,
            cancel_tx,
            cancel_rx,
            current_cancel_tx,
            processing_active: false,
            refresh_tx,
            refresh_rx,
        }
    }

    /// Replace the cancel channel pair, returning the old receiver for thread use.
    fn take_cancel_rx(&mut self) -> std::sync::mpsc::Receiver<()> {
        use std::sync::mpsc;
        let (new_tx, new_rx) = mpsc::channel::<()>();
        let old_rx = std::mem::replace(&mut self.cancel_rx, new_rx);
        self.cancel_tx = new_tx;
        old_rx
    }

    /// Reset the cancel channel (used after process completion).
    fn reset_cancel_channel(&mut self) {
        use std::sync::mpsc;
        let (new_tx, new_rx) = mpsc::channel::<()>();
        self.cancel_tx = new_tx;
        self.cancel_rx = new_rx;
    }
}

/// Handle completion of a processing thread.
fn handle_process_completion(app: &mut App, channels: &mut ProcessingChannels) {
    use std::sync::mpsc::TryRecvError;

    match channels.process_rx.try_recv() {
        Ok(result) => {
            channels.processing_active = false;
            app.cancelling = false;
            match result {
                Ok(msg) => app.processing_log.push(msg),
                Err(e) => {
                    let err_msg = e.to_string();
                    if err_msg.contains("Cancelled by user") {
                        app.processing_log.push("[WARN] Cancelled by user".to_string());
                        if !app.is_removing {
                            app.processing_log.push("[INFO] Cleaning up cancelled installation...".to_string());
                        }
                        app.processing_queue.clear();
                    } else if err_msg.contains("timed out") {
                        app.processing_log.push(format!("[ERR] {}", err_msg));
                        if !app.is_removing {
                            app.processing_log.push("[INFO] Cleaning up timed out installation...".to_string());
                        }
                    } else {
                        app.processing_log.push(format!("[ERR] {}", err_msg));
                    }
                }
            }

            let progress = app.processing_progress.unwrap_or(0) + 1;
            app.processing_progress = Some(progress);
            channels.reset_cancel_channel();

            if app.processing_queue.is_empty() {
                app.start_finish_processing();
            }
        }
        Err(TryRecvError::Empty) => {}
        Err(TryRecvError::Disconnected) => {
            channels.processing_active = false;
            app.processing_log.push("[ERR] Process thread crashed".to_string());
            if app.processing_queue.is_empty() {
                app.start_finish_processing();
            }
        }
    }
}

/// Dequeue and spawn the next processing task.
fn dispatch_next_process(app: &mut App, channels: &mut ProcessingChannels) {
    let idx = app.processing_queue.remove(0);
    channels.processing_active = true;

    let item_name = get_item_name(app, idx);
    let action = if app.is_removing { "Removing" } else { "Installing" };
    app.processing_log.push(format!("{} {}...", action, item_name));

    let tx_clone = channels.process_tx.clone();
    let is_removing = app.is_removing;
    let target_cli = app.target_cli.unwrap_or(app::TargetCli::Claude);
    let process_data = prepare_process_data(app, idx);

    // Update current_cancel_tx BEFORE spawning the thread
    channels.current_cancel_tx = channels.cancel_tx.clone();
    let cancel_rx_for_thread = channels.take_cancel_rx();

    thread::spawn(move || {
        let result = execute_process_step(process_data, is_removing, target_cli, cancel_rx_for_thread);
        let _ = tx_clone.send(result);
    });
}

/// Start a background thread to rescan components after install/remove.
fn start_refresh_thread(app: &mut App, refresh_tx: &std::sync::mpsc::Sender<Result<RefreshResult>>) {
    app.refreshing = true;

    let tx_clone = refresh_tx.clone();
    let source_dir = app.source_dir.clone();
    let dest_dir = app.dest_dir.clone();
    let target_cli = app.target_cli.unwrap_or(app::TargetCli::Claude);

    thread::spawn(move || {
        let result = (|| -> Result<RefreshResult> {
            let components = fs::scanner::scan_components(&source_dir, &dest_dir, target_cli)?;
            let mcp_servers = fs::scanner::scan_mcp_servers(&source_dir, target_cli, &dest_dir)?;
            let plugins = fs::scanner::scan_plugins(&source_dir)?;
            Ok((components, mcp_servers, plugins))
        })();
        let _ = tx_clone.send(result);
    });
}

/// Check if a refresh thread has completed and apply results.
fn check_refresh_completion(app: &mut App, refresh_rx: &std::sync::mpsc::Receiver<Result<RefreshResult>>) {
    use std::sync::mpsc::TryRecvError;

    match refresh_rx.try_recv() {
        Ok(Ok((components, mcp_servers, plugins))) => {
            app.apply_refresh_result(components, mcp_servers, plugins);
        }
        Ok(Err(e)) => {
            app.processing_log.push(format!("[ERROR] Refresh failed: {}", e));
            app.needs_refresh = false;
            app.refreshing = false;
            app.processing_complete = true;
        }
        Err(TryRecvError::Empty) => {}
        Err(TryRecvError::Disconnected) => {
            app.processing_log.push("[ERROR] Refresh thread crashed".to_string());
            app.needs_refresh = false;
            app.refreshing = false;
            app.processing_complete = true;
        }
    }
}

/// Handle a single tick of the Installing view.
fn handle_installing_view(app: &mut App, channels: &mut ProcessingChannels) -> Result<()> {
    if poll(Duration::from_millis(100))? {
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Release {
                handle_installing_input(app, key.code, &channels.current_cancel_tx, &channels.processing_active)?;
            }
        }
    }

    app.tick();

    if channels.processing_active {
        handle_process_completion(app, channels);
    }

    if !channels.processing_active && !app.processing_queue.is_empty() {
        dispatch_next_process(app, channels);
    } else if !channels.processing_active && app.processing_queue.is_empty() && app.needs_refresh && !app.refreshing {
        start_refresh_thread(app, &channels.refresh_tx);
    } else if app.refreshing {
        check_refresh_completion(app, &channels.refresh_rx);
    }

    Ok(())
}

/// Handle a single tick of the Loading view.
fn handle_loading_view(app: &mut App, refresh_rx: &std::sync::mpsc::Receiver<Result<RefreshResult>>) -> Result<()> {
    use std::sync::mpsc::TryRecvError;

    if poll(Duration::from_millis(100))? {
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Release && key.code == KeyCode::Char('q') {
                app.should_quit = true;
            }
        }
    }

    app.tick();

    match refresh_rx.try_recv() {
        Ok(Ok((components, mcp_servers, plugins))) => {
            app.finish_loading(components, mcp_servers, plugins);
        }
        Ok(Err(e)) => {
            app.status_message = Some(format!("Error loading: {}", e));
            app.current_view = app::View::CliSelection;
        }
        Err(TryRecvError::Empty) => {}
        Err(TryRecvError::Disconnected) => {
            app.status_message = Some("Loading failed".to_string());
            app.current_view = app::View::CliSelection;
        }
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
            app::View::Installing => handle_installing_view(app, &mut channels)?,
            app::View::CliSelection => {
                if let Event::Key(key) = event::read()? {
                    if key.kind != KeyEventKind::Release {
                        handle_cli_selection(app, key.code, &channels.refresh_tx)?;
                    }
                }
            }
            app::View::Loading => handle_loading_view(app, &channels.refresh_rx)?,
            app::View::EnvInput => {
                if let Event::Key(key) = event::read()? {
                    if key.kind != KeyEventKind::Release {
                        handle_env_input(app, key.code)?;
                    }
                }
            }
            app::View::ProjectPath => {
                if let Event::Key(key) = event::read()? {
                    if key.kind != KeyEventKind::Release {
                        handle_project_path_input(app, key.code);
                    }
                }
            }
            _ => {
                if let Event::Key(key) = event::read()? {
                    if key.kind != KeyEventKind::Release {
                        match app.current_view {
                            app::View::List => handle_list_input(app, key.code, key.modifiers)?,
                            app::View::Diff => handle_diff_input(app, key.code)?,
                            _ => {}
                        }
                    }
                }
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
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

fn handle_installing_input(
    app: &mut App,
    key: KeyCode,
    cancel_tx: &std::sync::mpsc::Sender<()>,
    processing_active: &bool,
) -> Result<()> {
    match key {
        KeyCode::Esc => {
            if *processing_active && !app.cancelling {
                // Cancel current operation (only once)
                let _ = cancel_tx.send(());
                app.processing_log.push("[WARN] Cancelling current operation...".to_string());
                app.cancelling = true;
            } else if app.processing_complete {
                // Exit after completion
                app.close_processing();
            }
        }
        KeyCode::Char('q') | KeyCode::Enter => {
            if app.processing_complete {
                app.close_processing();
            }
        }
        _ => {}
    }
    Ok(())
}

fn handle_cli_selection(
    app: &mut App,
    key: KeyCode,
    refresh_tx: &std::sync::mpsc::Sender<Result<(Vec<component::Component>, Vec<mcp::McpServer>, Vec<plugin::Plugin>)>>,
) -> Result<()> {
    match key {
        KeyCode::Char('1') => {
            app.select_cli(app::TargetCli::Claude)?;
            start_loading_thread(app, refresh_tx);
        }
        KeyCode::Char('2') => {
            app.select_cli(app::TargetCli::Codex)?;
            start_loading_thread(app, refresh_tx);
        }
        KeyCode::Char('q') => app.should_quit = true,
        _ => {}
    }
    Ok(())
}

fn start_loading_thread(
    app: &App,
    refresh_tx: &std::sync::mpsc::Sender<Result<(Vec<component::Component>, Vec<mcp::McpServer>, Vec<plugin::Plugin>)>>,
) {
    let tx_clone = refresh_tx.clone();
    let source_dir = app.source_dir.clone();
    let dest_dir = app.dest_dir.clone();
    let target_cli = app.target_cli.unwrap_or(app::TargetCli::Claude);

    thread::spawn(move || {
        let components = fs::scanner::scan_components(&source_dir, &dest_dir, target_cli);
        let mcp_servers = fs::scanner::scan_mcp_servers(&source_dir, target_cli, &dest_dir);
        let plugins = fs::scanner::scan_plugins(&source_dir);

        match (components, mcp_servers, plugins) {
            (Ok(c), Ok(m), Ok(p)) => {
                let _ = tx_clone.send(Ok((c, m, p)));
            }
            (Err(e), _, _) | (_, Err(e), _) | (_, _, Err(e)) => {
                let _ = tx_clone.send(Err(e));
            }
        }
    });
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

/// Data needed for async processing
#[derive(Clone)]
enum ProcessData {
    Component {
        name: String,
        source_path: std::path::PathBuf,
        dest_path: std::path::PathBuf,
        component_type: component::ComponentType,
        hook_config: Option<component::HookConfig>,
        source_dir: std::path::PathBuf,
        dest_dir: std::path::PathBuf,
    },
    McpServer {
        server: mcp::McpServer,
        scope: mcp::McpScope,
        project_path: Option<String>,
        env_values: Vec<(String, String)>,
    },
    Plugin {
        plugin: plugin::Plugin,
    },
}

fn prepare_process_data(app: &App, idx: usize) -> ProcessData {
    if app.tab == app::Tab::McpServers {
        let server = app.mcp_servers[idx].clone();
        let env_values = if app.env_input_server_idx == Some(idx) {
            app.env_input_values.clone()
        } else {
            Vec::new()
        };
        let project_path = if app.mcp_scope == mcp::McpScope::Local {
            Some(app.mcp_project_path.clone())
        } else {
            None
        };
        ProcessData::McpServer {
            server,
            scope: app.mcp_scope,
            project_path,
            env_values,
        }
    } else if app.tab == app::Tab::Plugins {
        ProcessData::Plugin {
            plugin: app.plugins[idx].clone(),
        }
    } else {
        let c = &app.components[idx];
        ProcessData::Component {
            name: c.name.clone(),
            source_path: c.source_path.clone(),
            dest_path: c.dest_path.clone(),
            component_type: c.component_type.clone(),
            hook_config: c.hook_config.clone(),
            source_dir: app.source_dir.clone(),
            dest_dir: app.dest_dir.clone(),
        }
    }
}

fn get_item_name(app: &App, idx: usize) -> String {
    if app.tab == app::Tab::McpServers {
        app.mcp_servers.get(idx).map(|s| s.def.name.clone()).unwrap_or_default()
    } else if app.tab == app::Tab::Plugins {
        app.plugins.get(idx).map(|p| p.def.name.clone()).unwrap_or_default()
    } else {
        app.components.get(idx).map(|c| c.name.clone()).unwrap_or_default()
    }
}

fn execute_process_step(
    data: ProcessData,
    is_removing: bool,
    target_cli: app::TargetCli,
    cancel_rx: std::sync::mpsc::Receiver<()>,
) -> Result<String> {
    match data {
        ProcessData::McpServer { server, scope, project_path, env_values } => {
            let name = server.def.name.clone();
            let timeout = if is_removing { 30 } else { 120 };

            if is_removing {
                match fs::installer::remove_mcp_server(&server, target_cli, timeout, &cancel_rx) {
                    Ok(_) => Ok(format!("[OK] Removed {}", name)),
                    Err(e) => Err(e),
                }
            } else {
                match fs::installer::install_mcp_server(
                    &server,
                    fs::installer::McpInstallConfig {
                        scope,
                        project_path: project_path.as_deref(),
                        env_values: &env_values,
                        target_cli,
                        timeout_secs: timeout,
                        cancel_rx: &cancel_rx,
                    },
                ) {
                    Ok(_) => Ok(format!("[OK] Installed {}", name)),
                    Err(e) => Err(e),
                }
            }
        }
        ProcessData::Plugin { plugin } => {
            let name = plugin.def.name.clone();
            let timeout = if is_removing { 30 } else { 60 };

            if is_removing {
                match fs::installer::remove_plugin(&plugin, timeout, &cancel_rx) {
                    Ok(_) => Ok(format!("[OK] Removed {}", name)),
                    Err(e) => Err(e),
                }
            } else {
                match fs::installer::install_plugin(&plugin, timeout, &cancel_rx) {
                    Ok(_) => Ok(format!("[OK] Installed {}", name)),
                    Err(e) => Err(e),
                }
            }
        }
        ProcessData::Component { name, source_path, dest_path, component_type, hook_config, source_dir, dest_dir } => {
            if is_removing {
                // Create a temporary Component for removal
                let comp = component::Component {
                    name: name.clone(),
                    source_path: source_path.clone(),
                    dest_path: dest_path.clone(),
                    component_type,
                    hook_config,
                    status: component::InstallStatus::Unchanged,
                    selected: false,
                };
                match fs::installer::remove_component(&comp, &dest_dir) {
                    Ok(_) => Ok(format!("[OK] Removed {}", name)),
                    Err(e) => Ok(format!("[ERR] {}: {}", name, e)),
                }
            } else {
                // Create a temporary Component for install
                let comp = component::Component {
                    name: name.clone(),
                    source_path,
                    dest_path,
                    component_type,
                    hook_config,
                    status: component::InstallStatus::New,
                    selected: false,
                };
                match fs::installer::install_component(&comp, &source_dir, &dest_dir) {
                    Ok(_) => Ok(format!("[OK] Installed {}", name)),
                    Err(e) => Ok(format!("[ERR] {}: {}", name, e)),
                }
            }
        }
    }
}
