---
name: ratatui
description: Use this skill when the user is building or modifying a terminal UI (TUI) in Rust, especially with the `ratatui` or `crossterm` crates. Triggers include rendering widgets (List, Table, Paragraph, Gauge, Chart, Sparkline, Tabs), stateful selection with ListState/TableState, layouts and constraints, keyboard-driven menus and arrow-key navigation, event loops combining input with background tasks (channels, tokio, esc-to-cancel), panic-safe terminal setup, custom Widget implementations, Ratatui 0.28→0.30 migration and deprecated idioms, or building lazygit/btop/htop-style interactive Rust CLIs. Also triggers on Korean intents: 러스트/Rust로 터미널 UI, TUI, 대화형 CLI, 메뉴 화면, 화살표 이동, 위젯 렌더링/조합, 레이아웃, ListState 하이라이트, 이벤트 루프, crossterm 이벤트, 백그라운드 작업 취소, ratatui 마이그레이션. Do NOT use for ncurses/C, Python rich/textual, bash/tput, non-TUI Rust (web/server/lib/embedded), Tauri/Iced/egui (GUI), or Bubble Tea (Go).
keywords: [ratatui, tui, 터미널ui, rust-tui, cli, crossterm, cancelable, 취소, cross-platform, 크로스플랫폼, listState, migration, 마이그레이션]
---

# Ratatui (Rust TUI) — Production Guide

Immediate-mode terminal UI framework for Rust. This skill encodes patterns proven in production (cancelable background tasks, cross-platform handling, panic-safe terminal restoration) — not just hello-world.

## When to use which reference

This SKILL.md is the index. Read the focused reference based on the task:

| Task | Reference |
|---|---|
| Use a widget (List/Table/Paragraph/Gauge/Tabs/...) | `references/widgets.md` |
| Background task + user-cancelable spawn | `references/cancelable-processes.md` |
| Windows path/command quirks, MSYS conversion | `references/cross-platform.md` |
| Unit-test rendering with `TestBackend` | `references/testing.md` |
| Avoiding common TUI footguns (panic, eprintln, JoinHandle) | `references/gotchas.md` |
| App structure, modules, state machines, channels | `references/best-practices.md` |

## Project Setup

Use Edition 2024 (Rust 1.85+) with `ratatui::init()` (added in 0.28+) for panic-safe terminal handling.

```toml
[package]
name = "my-tui-app"
edition = "2024"
rust-version = "1.85"

[dependencies]
ratatui = "0.30"
crossterm = "0.29"
anyhow = "1.0"
```

## Minimal App (idiomatic 0.30)

```rust
use ratatui::{
    crossterm::event::{self, Event, KeyCode},
    widgets::{Block, Paragraph},
    DefaultTerminal,
};
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    // ratatui::init() enables raw mode, alt screen, and installs a panic hook
    // that restores the terminal on panic. ratatui::restore() unwinds it.
    let mut terminal = ratatui::init();
    let result = run(&mut terminal);
    ratatui::restore();
    result
}

fn run(terminal: &mut DefaultTerminal) -> anyhow::Result<()> {
    let mut counter: i32 = 0;
    loop {
        terminal.draw(|f| {
            let block = Block::bordered().title("Counter");
            let para = Paragraph::new(format!("Count: {counter}")).block(block);
            f.render_widget(para, f.area());
        })?;

        // Poll instead of block-read so a future tick/channel-recv is possible.
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Up => counter += 1,
                    KeyCode::Down => counter -= 1,
                    _ => {}
                }
            }
        }
    }
}
```

**Why `ratatui::init()` instead of manual `Terminal::new`**: it installs a panic hook that restores the terminal even if the app panics mid-render. Without it, a panic leaves the terminal in raw mode and the user can't see anything they type.

## Layout (0.30 destructuring style)

```rust
use ratatui::layout::{Constraint, Layout};

let [header, body, status] = Layout::vertical([
    Constraint::Length(3),  // Fixed
    Constraint::Fill(1),    // Take remaining (replaces Min(0))
    Constraint::Length(1),  // Status bar
]).areas(f.area());

let [sidebar, main] = Layout::horizontal([
    Constraint::Percentage(30),
    Constraint::Percentage(70),
]).areas(body);
```

Use `Constraint::Fill(n)` (0.27+) for proportional remaining space — clearer intent than `Min(0)`.

## Event Loop with Background Channel

The pattern when a TUI must update on both keypress AND background events (timers, async I/O, child-process output):

```rust
use std::sync::mpsc;
use std::time::Duration;

enum AppEvent {
    Tick,
    Output(String),
    Done,
}

fn run(terminal: &mut DefaultTerminal, rx: mpsc::Receiver<AppEvent>) -> anyhow::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &state))?;

        // 1) Drain background events (non-blocking)
        while let Ok(event) = rx.try_recv() {
            state.handle(event);
        }

        // 2) Poll keyboard with short timeout so background events can arrive
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') { return Ok(()); }
                state.handle_key(key.code);
            }
        }
    }
}
```

For the production-grade version with **cancelable** child processes, see `references/cancelable-processes.md`.

## State Management (View enum pattern)

For non-trivial apps, model possible screens as an enum so transitions are type-checked:

```rust
#[derive(Clone, Copy, PartialEq)]
pub enum View {
    Loading,
    List,
    Detail,
    Modal(ModalKind),
    Confirm,
}

struct App {
    view: View,
    // ... per-view state
    should_quit: bool,
}

impl App {
    fn handle_key(&mut self, code: KeyCode) {
        match self.view {
            View::Loading => {} // ignore input
            View::List => self.handle_list_key(code),
            View::Detail => self.handle_detail_key(code),
            View::Modal(_) => self.handle_modal_key(code),
            View::Confirm => self.handle_confirm_key(code),
        }
    }
}
```

Why: a 13-state TUI without this pattern becomes an `if/else` rats' nest. With it, the compiler catches missing branches when you add a screen.

## Custom Widget (Widget trait)

```rust
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget, text::Line, style::Style};

struct StatusBar { msg: String }

impl Widget for StatusBar {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Line::raw(self.msg).style(Style::new()).render(area, buf);
    }
}

// Usage
f.render_widget(StatusBar { msg: "Ready".into() }, area);
```

Custom widgets keep `draw(...)` callbacks short and let you unit-test rendering with `TestBackend` (see `references/testing.md`).

## Performance Quick Rules

1. **Poll, don't block** — use `event::poll(Duration::from_millis(50–100))` so background channels can be drained.
2. **`tick()` ≠ redraw** — only redraw if state actually changed; the double-buffer in ratatui is cheap, but your build_ui closure isn't.
3. **No `eprintln!` while raw mode is on** — it corrupts the alt screen. See `references/gotchas.md`.
4. **Detach vs join threads carefully** — `drop(JoinHandle)` does NOT join. See `references/gotchas.md`.

## Migration Notes (0.28 → 0.30)

- Prefer `ratatui::init()` / `ratatui::restore()` over manual `enable_raw_mode` + `EnterAlternateScreen`.
- Prefer `Block::bordered()` over `Block::default().borders(Borders::ALL)`.
- Prefer `Layout::vertical([...]).areas(area)` destructuring over `.split(area)` indexing.
- Prefer `Constraint::Fill(1)` over `Constraint::Min(0)` for remaining space.
- `f.size()` → `f.area()`.

## Related Skills

- General Rust idioms: `rust-best-practices`
- Cross-platform shell-safety: `references/cross-platform.md` (this skill) + `coding-standards`
