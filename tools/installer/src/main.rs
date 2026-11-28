mod app;
mod cli;
mod component;
mod exec;
mod fs;
#[cfg(feature = "gui")]
mod gui;
mod loading;
mod mcp;
mod paths;
mod plugin;
mod process_exec;
mod source;
mod target;
mod theme;
mod tree;
mod ui;

use anyhow::{Context, Result};
use crossterm::{
    cursor::{MoveTo, Show},
    event::KeyCode,
    execute,
    terminal::{
        Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
        enable_raw_mode,
    },
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;
use std::thread;
use std::time::Duration;

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

    #[cfg(feature = "gui")]
    if args.iter().any(|a| a == "--font-licenses") {
        print!("{}", include_str!("../assets/fonts/FONT-LICENSES.txt"));
        return Ok(());
    }

    if args.iter().any(|a| a == "--sync") {
        return cli::run_sync();
    }

    #[cfg(feature = "gui")]
    if !args.iter().any(|a| a == "--tui") {
        #[cfg(target_os = "linux")]
        require_graphical_display(|name| std::env::var_os(name))?;
        if exec::launch_gui_from_terminal()? {
            return Ok(());
        }
        return std::panic::catch_unwind(gui::run)
            .map_err(|_| {
                anyhow::anyhow!(
                    "Desktop interface stopped unexpectedly. Use --tui for terminal mode."
                )
            })?
            .map_err(|error| {
                anyhow::anyhow!("GUI startup failed: {error}. Use --tui for terminal mode.")
            });
    }

    run_terminal()
}

#[cfg(all(feature = "gui", target_os = "linux"))]
fn require_graphical_display(lookup: impl Fn(&str) -> Option<std::ffi::OsString>) -> Result<()> {
    if ["DISPLAY", "WAYLAND_DISPLAY", "WAYLAND_SOCKET"]
        .iter()
        .any(|name| lookup(name).is_some_and(|value| !value.is_empty()))
    {
        Ok(())
    } else {
        anyhow::bail!("No graphical display is available. Use --tui for terminal mode.");
    }
}

fn run_terminal() -> Result<()> {
    enable_raw_mode()?;
    let result = std::panic::catch_unwind(|| -> Result<()> {
        let mut stdout = io::stdout();
        execute!(
            stdout,
            EnterAlternateScreen,
            Clear(ClearType::All),
            MoveTo(0, 0)
        )?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        let (tx, rx) = std::sync::mpsc::channel();
        thread::spawn(move || {
            let _ = tx.send(App::new());
        });

        let mut frame_idx = 0;
        let mut app = loop {
            terminal.draw(|frame| ui::loading_screen::draw(frame, frame_idx))?;
            if let Some(app) = poll_startup(&rx)? {
                break app;
            }
            frame_idx += 1;
            thread::sleep(Duration::from_millis(100));
        };
        run_app(&mut terminal, &mut app)
    })
    .unwrap_or_else(|_| Err(anyhow::anyhow!("Terminal interface stopped unexpectedly")));

    // Attempt every cleanup operation before propagating any application error.
    let raw_result = disable_raw_mode().context("Cannot restore terminal input mode");
    let mut stdout = io::stdout();
    let screen_result = execute!(stdout, LeaveAlternateScreen)
        .context("Cannot leave the alternate terminal screen");
    let cursor_result = execute!(stdout, Show).context("Cannot restore the terminal cursor");
    result.and(raw_result).and(screen_result).and(cursor_result)
}

fn poll_startup(rx: &std::sync::mpsc::Receiver<Result<App>>) -> Result<Option<App>> {
    match rx.try_recv() {
        Ok(result) => result.map(Some),
        Err(std::sync::mpsc::TryRecvError::Empty) => Ok(None),
        Err(std::sync::mpsc::TryRecvError::Disconnected) => {
            anyhow::bail!("Initialization worker stopped unexpectedly");
        }
    }
}

fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()>
where
    <B as ratatui::backend::Backend>::Error: Send + Sync + 'static,
{
    let mut channels = ProcessingChannels::new();
    loop {
        let area = terminal.draw(|f| ui::draw(f, app))?.area;

        match app.current_view {
            app::View::Installing => loading::handle_installing_view(app, &mut channels)?,
            app::View::Loading => loading::handle_loading_view(app, &channels.refresh_rx)?,
            app::View::Preflighting => loading::handle_preflighting_view(app, &mut channels)?,
            app::View::SourceSyncing => cli::handle_source_syncing(app)?,
            _ => {
                if let Some(code) = cli::read_key_press()? {
                    let view = app.current_view;
                    if let Err(error) =
                        cli::dispatch_terminal_key(app, code, area, &channels.refresh_tx)
                    {
                        match view {
                            app::View::SourceConfirmRemove => app.current_view = app::View::Sources,
                            app::View::SourceAddMapTo => {}
                            _ => return Err(error),
                        }
                        show_source_error(terminal, app, &error)?;
                    }
                }
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

fn show_source_error<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &App,
    error: &anyhow::Error,
) -> Result<()>
where
    B::Error: Send + Sync + 'static,
{
    let mut message = String::from("Could not save source settings.");
    for cause in error.chain() {
        message.push('\n');
        message.push_str(&cause.to_string());
    }
    let mut scroll = 0usize;
    loop {
        let mut max_scroll = 0;
        terminal.draw(|f| {
            ui::draw(f, app);
            max_scroll = ui::render_source_error(f, app, &message, scroll);
        })?;
        scroll = scroll.min(max_scroll);
        match cli::read_key_press()? {
            Some(KeyCode::Enter | KeyCode::Esc) => return Ok(()),
            Some(KeyCode::Down | KeyCode::Char('j')) => scroll = (scroll + 1).min(max_scroll),
            Some(KeyCode::Up | KeyCode::Char('k')) => scroll = scroll.saturating_sub(1),
            _ => {}
        }
    }
}

#[cfg(test)]
mod startup_tests {
    use super::*;

    #[test]
    fn startup_worker_disconnect_is_an_error_instead_of_an_endless_spinner() {
        let (tx, rx) = std::sync::mpsc::channel();
        drop(tx);
        let error = poll_startup(&rx)
            .err()
            .expect("worker disconnect must fail");
        assert!(
            error
                .to_string()
                .contains("Initialization worker stopped unexpectedly")
        );
    }

    #[test]
    fn startup_errors_keep_the_original_reason() {
        let (tx, rx) = std::sync::mpsc::channel();
        tx.send(Err(anyhow::anyhow!("Cannot find source directory")))
            .unwrap();
        let error = poll_startup(&rx)
            .err()
            .expect("startup error must propagate");
        assert!(error.to_string().contains("Cannot find source directory"));
    }
}

#[cfg(all(test, feature = "gui", target_os = "linux"))]
mod display_tests {
    use super::*;

    #[test]
    fn absent_or_empty_display_settings_return_terminal_guidance() {
        let error = require_graphical_display(|_| None).unwrap_err();
        assert!(error.to_string().contains("--tui"));
        assert!(require_graphical_display(|_| Some(std::ffi::OsString::new())).is_err());
    }

    #[test]
    fn x11_wayland_names_and_wayland_socket_are_recognized() {
        for selected in ["DISPLAY", "WAYLAND_DISPLAY", "WAYLAND_SOCKET"] {
            assert!(
                require_graphical_display(|name| {
                    (name == selected).then(|| "display-value".into())
                })
                .is_ok()
            );
        }
    }
}
