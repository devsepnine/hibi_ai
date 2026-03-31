mod app;
mod cli;
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
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, Clear, ClearType},
    cursor::MoveTo,
};
use ratatui::{backend::CrosstermBackend, Terminal};

use app::App;
use loading::ProcessingChannels;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--help" || a == "-h") {
        cli::print_help();
        return Ok(());
    }

    if args.iter().any(|a| a == "--version" || a == "-v") {
        println!("hibi {} (Config Installer)", fs::VERSION);
        return Ok(());
    }

    if args.iter().any(|a| a == "--sync") {
        return cli::run_sync();
    }

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, Clear(ClearType::All), MoveTo(0, 0))?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app in background thread while showing loading animation
    use std::sync::{Arc, Mutex};

    let app_result: Arc<Mutex<Option<Result<App>>>> = Arc::new(Mutex::new(None));
    let app_result_clone = Arc::clone(&app_result);

    thread::spawn(move || {
        let result = App::new();
        *app_result_clone.lock().unwrap() = Some(result);
    });

    let mut frame_idx = 0;
    loop {
        terminal.draw(|f| ui::loading_screen::draw(f, frame_idx))?;
        let result_lock = app_result.lock().unwrap();
        if result_lock.is_some() {
            break;
        }
        drop(result_lock);
        frame_idx += 1;
        std::thread::sleep(Duration::from_millis(100));
    }

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
            app::View::SourceSyncing => cli::handle_source_syncing(app)?,
            _ => {
                if let Some((code, modifiers)) = cli::read_key_press()? {
                    cli::dispatch_key(app, code, modifiers, &channels.refresh_tx)?;
                }
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}
