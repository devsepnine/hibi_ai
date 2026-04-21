# Testing Ratatui Apps

Ratatui ships with `TestBackend`, a backend that renders into an in-memory buffer instead of a real terminal. This makes rendering deterministic and unit-testable.

## TestBackend: First Test in 60 Seconds

```rust
use ratatui::{
    backend::TestBackend,
    buffer::Buffer,
    widgets::{Block, Paragraph},
    Terminal,
};

#[test]
fn renders_paragraph_with_title() {
    let backend = TestBackend::new(20, 3); // 20 cols × 3 rows
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|f| {
        let p = Paragraph::new("hi").block(Block::bordered().title("T"));
        f.render_widget(p, f.area());
    }).unwrap();

    let expected = Buffer::with_lines([
        "┌T─────────────────┐",
        "│hi                │",
        "└──────────────────┘",
    ]);
    terminal.backend().assert_buffer(&expected);
}
```

`assert_buffer` compares cell-by-cell and prints a visual diff on mismatch — clearer than `assert_eq!` on raw strings.

## Test the App Logic Separately From the UI

The most testable structure separates **state mutation** from **rendering**. Then state has plain unit tests with no terminal involved:

```rust
struct App {
    counter: i32,
    items: Vec<String>,
    selected: usize,
}

impl App {
    fn select_next(&mut self) {
        if !self.items.is_empty() {
            self.selected = (self.selected + 1) % self.items.len();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn select_next_wraps_around() {
        let mut app = App {
            counter: 0,
            items: vec!["a".into(), "b".into()],
            selected: 1,
        };
        app.select_next();
        assert_eq!(app.selected, 0);
    }

    #[test]
    fn select_next_on_empty_does_not_panic() {
        let mut app = App { counter: 0, items: vec![], selected: 0 };
        app.select_next(); // would underflow if guard missing
        assert_eq!(app.selected, 0);
    }
}
```

Most bugs in TUIs are state-machine bugs (off-by-one in selection, missed mode transition). Cover those with cheap unit tests; reserve `TestBackend` for rendering regressions you actually see.

## Snapshot-Style Rendering Tests

For complex layouts, `Buffer::with_lines(...)` matches expected character cells but ignores style. To check styles too, use `Buffer::with_lines` + `set_style`:

```rust
use ratatui::style::{Color, Style};

let mut expected = Buffer::with_lines(["┌Title──┐", "│hello  │", "└───────┘"]);
expected.set_style(
    ratatui::layout::Rect::new(1, 0, 5, 1), // "Title" cells
    Style::new().fg(Color::Cyan),
);
terminal.backend().assert_buffer(&expected);
```

## Testing Stateful Widgets

`StatefulWidget` (List, Table) requires its state during render. In tests, construct the state explicitly:

```rust
use ratatui::widgets::{List, ListItem, ListState};

#[test]
fn list_shows_selection_marker() {
    let backend = TestBackend::new(15, 4);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|f| {
        let items = vec![ListItem::new("a"), ListItem::new("b"), ListItem::new("c")];
        let list = List::new(items).highlight_symbol("> ");
        let mut state = ListState::default();
        state.select(Some(1));
        f.render_stateful_widget(list, f.area(), &mut state);
    }).unwrap();

    terminal.backend().assert_buffer(&Buffer::with_lines([
        "  a            ",
        "> b            ",
        "  c            ",
        "               ",
    ]));
}
```

## Driving Key Events in Tests

`event::read()` is hard to mock. Instead, refactor the key dispatch into a pure function:

```rust
fn handle_key(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Down => app.select_next(),
        KeyCode::Up => app.select_prev(),
        _ => {}
    }
}

#[test]
fn down_selects_next() {
    let mut app = App::new(vec!["a".into(), "b".into()]);
    handle_key(&mut app, KeyCode::Down);
    assert_eq!(app.selected, 1);
}
```

This also makes the dispatcher itself testable as a table:

```rust
#[test]
fn key_table_smoke() {
    let cases = [
        (KeyCode::Down, 1),
        (KeyCode::Up, 0),
        (KeyCode::Char('q'), 0), // unchanged
    ];
    for (key, expected) in cases {
        let mut app = App::new(vec!["a".into(), "b".into()]);
        handle_key(&mut app, key);
        assert!(app.selected <= expected);
    }
}
```

## What to Test, What Not To

| Test | Don't test |
|---|---|
| State transitions (App methods) | Exact pixel positions of decorative borders |
| Key → action dispatch | crossterm's own event parsing |
| Widget rendering you've customized | Standard widgets unchanged from upstream |
| Cancel/timeout logic in spawn helpers | Real child-process exit codes (use a fake) |

The goal of tests in a TUI is to catch state-machine regressions and rendering of *your* widgets, not to re-test the framework.

## Running Tests

```bash
cargo test
cargo test --lib                    # only library tests
cargo test renders_                 # filter by name
cargo test -- --nocapture           # see println! output
```

For TUI apps that also test child-process spawning, prefer in-process fakes (a fn that takes `&mut Vec<u8>` to write to, instead of a real `Command`) — this keeps tests fast and OS-independent.
