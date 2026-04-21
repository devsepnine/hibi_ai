# TUI Best Practices

Patterns proven in production TUIs. The focus is structure: how to organize a non-trivial app so it stays maintainable past 10 screens.

## Module Layout for a Non-Trivial TUI

Keep `main.rs` thin. The signal that you're on the right path: `main.rs` is mostly terminal setup + event loop dispatch, ~100 lines. Everything else lives in submodules.

```
src/
├── main.rs          // ratatui::init/restore, event loop dispatch (~100 LOC)
├── cli.rs           // Key handlers, dispatch by view
├── app/
│   ├── mod.rs       // App struct, public API
│   ├── types.rs     // View enum, ModeKind, IDs
│   ├── navigation.rs// Cursor / selection / mode transitions
│   ├── input.rs     // Text input state for modals
│   └── processing.rs// Background-task lifecycle (spinner, status)
├── ui/              // Pure rendering — receives &App, returns nothing
│   ├── mod.rs
│   ├── list.rs
│   └── modal.rs
├── source/          // Domain logic (independent of UI)
└── fs/              // File-system effects (install, scan)
```

**Rule of thumb**: a file in `ui/` should never spawn a thread, write to disk, or mutate `App`. It only reads `&App` and renders. This makes UI testable with `TestBackend` and lets you swap renderers later if needed.

## State: One App Struct, One View Enum

```rust
#[derive(Clone, Copy, PartialEq)]
pub enum View {
    Loading,
    List,
    Detail,
    Confirm,
    Installing, // background work in progress
}

pub struct App {
    pub view: View,
    pub items: Vec<Item>,
    pub selected: usize,
    pub processing: ProcessingState,
    pub should_quit: bool,
}
```

**Why a single struct**: easy to pass `&App` to render fns, easy to snapshot for tests.
**Why an enum for view**: the compiler catches missing branches when you add a screen. Booleans (`is_modal_open`, `is_loading`) silently allow invalid combinations.

## Channels: Bundle Them, Don't Pass Loose

When the loop needs to hear from background threads, group all channels into a single struct so the function signature stays clean:

```rust
pub struct ProcessingChannels {
    pub process_tx: mpsc::Sender<Result<String>>,
    pub process_rx: mpsc::Receiver<Result<String>>,
    pub current_cancel_tx: mpsc::Sender<()>,
    pub processing_active: bool,
    pub refresh_tx: mpsc::Sender<RefreshEvent>,
    pub refresh_rx: mpsc::Receiver<RefreshEvent>,
}

fn run_loop(terminal: &mut DefaultTerminal, app: &mut App) -> anyhow::Result<()> {
    let mut channels = ProcessingChannels::new();
    loop {
        terminal.draw(|f| ui::draw(f, app))?;
        drain_channels(app, &mut channels);
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                cli::dispatch_key(app, key.code, &channels)?;
            }
        }
        if app.should_quit { return Ok(()); }
    }
}
```

For per-job cancellation isolation, see `references/cancelable-processes.md`.

## Keyboard Shortcuts (Discoverable Defaults)

Pick the bindings users already know from vi / less / ranger / lazygit. Don't invent.

| Key | Action | Notes |
|---|---|---|
| `q`, `Esc` | Quit / Back | `q` quits app at top level; `Esc` backs out of a modal. |
| `j` / `k` / arrows | Down / Up | Both forms — vi users and arrow users coexist. |
| `h` / `l` | Left / Right (or back/forward) | |
| `g` / `G` | Top / Bottom | |
| `Enter` | Confirm / Open | |
| `Space` | Toggle selection | When the screen is checklist-like. |
| `/` | Search/filter | |
| `?` | Show help | |
| `Tab` / `Shift+Tab` | Next / Prev field in a form | |
| `Ctrl+C` | Force quit | Treat as `q` even mid-operation; cleanup must run. |

Show the active bindings in a status bar — discoverability is half the battle.

## Status Bar (Always Visible)

A footer status bar with mode, item count, and "?: help" hint trains users:

```rust
fn render_status(f: &mut Frame, area: Rect, app: &App) {
    let text = format!(
        " {mode} | {n} items | ? help | q quit ",
        mode = app.view_name(),
        n = app.items.len()
    );
    let p = Paragraph::new(text)
        .style(Style::new().bg(Color::DarkGray).fg(Color::White));
    f.render_widget(p, area);
}
```

## Responsive Layout

Adapt to terminal size. The single most useful threshold is "is there room for a sidebar?":

```rust
fn ui(f: &mut Frame, app: &App) {
    let area = f.area();
    if area.width >= 80 {
        let [sidebar, main] = Layout::horizontal([
            Constraint::Length(28),
            Constraint::Fill(1),
        ]).areas(area);
        render_sidebar(f, sidebar, app);
        render_main(f, main, app);
    } else {
        // Stack: hide sidebar, show its content as a togglable view.
        render_main(f, area, app);
    }
}
```

## Theming via Semantic Colors

Don't hardcode `Color::Cyan` everywhere. Define semantic colors in one place so you can support light/dark terminals:

```rust
pub struct Theme {
    pub fg: Color,
    pub bg: Color,
    pub accent: Color,
    pub muted: Color,
    pub error: Color,
}

impl Theme {
    pub fn dark() -> Self { Self { fg: Color::White, bg: Color::Black, accent: Color::Cyan, muted: Color::DarkGray, error: Color::Red } }
}
```

Then UI code reads `app.theme.accent`, never `Color::Cyan` literally.

## Animation Frame Counter

For spinners, store an integer that ticks per loop iteration; map to a glyph at draw time:

```rust
pub const SPINNER_FRAMES: &[&str] = &["⠋","⠙","⠹","⠸","⠼","⠴","⠦","⠧","⠇","⠏"];

impl App {
    pub fn tick(&mut self) {
        self.frame = (self.frame + 1) % SPINNER_FRAMES.len();
    }
}

// at draw time
let glyph = SPINNER_FRAMES[app.frame];
```

The render layer never owns wall-clock time — easier to test.

## Error Handling

Return `anyhow::Result<T>` from app functions; show errors as a transient toast or inline status, not a panic:

```rust
match install_item(item) {
    Ok(()) => app.toast("Installed"),
    Err(e) => app.toast(format!("Failed: {e}")),
}
```

For unrecoverable errors that should exit the app, set `app.should_quit = true` and let the loop print after `ratatui::restore()`.

## Performance: When (Not) to Care

ratatui's renderer is fast. Don't optimize until you measure. The two things that *do* matter:

1. **Build cost of widget data**: if you rebuild a 10k-item list every frame, you're hot. Cache the prepared `Vec<ListItem>` and only rebuild when items change.
2. **Polling interval**: 100ms is fine for most apps; 16ms for animation-heavy. Don't go below 16ms — you're past terminal refresh anyway.

## Accessibility

- High contrast: avoid light-gray-on-white or yellow-on-white.
- Don't rely on color alone: pair with symbols (`✓ ok`, `✗ failed`, `▶ running`).
- Always keyboard-accessible: assume mouse is unavailable.
- Predictable focus indicators: `> `, `█`, or background highlight — pick one and stick to it.

## Ship Checklist

- [ ] `ratatui::init()` / `ratatui::restore()` (or panic hook)
- [ ] All long operations cancelable (`references/cancelable-processes.md`)
- [ ] Tested on Windows + macOS + Linux (`references/cross-platform.md`)
- [ ] At least one rendering test per non-trivial screen (`references/testing.md`)
- [ ] Keybindings shown in status bar
- [ ] Errors show as toast, not panic
- [ ] Builds with `cargo build --release` warning-free
