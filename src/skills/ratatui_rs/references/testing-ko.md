# Testing Ratatui Apps

Ratatui는 실제 터미널 대신 in-memory 버퍼로 렌더링하는 backend인 `TestBackend`를 제공한다. 이는 렌더링을 결정론적이고 단위 테스트 가능하게 만든다.

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

`assert_buffer`는 셀 단위로 비교하고 불일치 시 시각적 diff를 출력한다 — raw 문자열의 `assert_eq!`보다 명확하다.

## Test the App Logic Separately From the UI

가장 테스트 가능한 구조는 **state mutation**과 **rendering**을 분리한다. 그러면 state는 터미널이 관여하지 않는 평이한 단위 테스트를 가진다:

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

TUI의 대부분 버그는 state-machine 버그이다 (선택의 off-by-one, 놓친 모드 전환). 저렴한 단위 테스트로 이를 다룬다; 실제로 보는 렌더링 회귀를 위해 `TestBackend`를 예약한다.

## Snapshot-Style Rendering Tests

복잡한 레이아웃의 경우, `Buffer::with_lines(...)`가 예상 문자 셀과 일치하지만 스타일은 무시한다. 스타일도 확인하려면 `Buffer::with_lines` + `set_style`을 사용한다:

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

`StatefulWidget` (List, Table)은 렌더링 중 state가 필요하다. 테스트에서 명시적으로 state를 구성한다:

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

`event::read()`는 mock하기 어렵다. 대신 key dispatch를 순수 함수로 리팩토링한다:

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

이는 dispatcher 자체도 테이블로 테스트 가능하게 만든다:

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
| State 전환 (App 메서드) | 장식적 테두리의 정확한 픽셀 위치 |
| Key → action 디스패치 | crossterm 자체의 이벤트 파싱 |
| 커스터마이즈한 위젯 렌더링 | upstream에서 변경되지 않은 표준 위젯 |
| spawn 헬퍼의 cancel/timeout 로직 | 실제 자식 프로세스 종료 코드 (fake 사용) |

TUI의 테스트 목표는 state-machine 회귀와 *자신의* 위젯 렌더링을 잡는 것이지, 프레임워크를 다시 테스트하는 것이 아니다.

## Running Tests

```bash
cargo test
cargo test --lib                    # only library tests
cargo test renders_                 # filter by name
cargo test -- --nocapture           # see println! output
```

자식 프로세스 spawn도 테스트하는 TUI 앱의 경우, in-process fake를 선호한다 (실제 `Command` 대신 쓸 `&mut Vec<u8>`을 받는 fn) — 이는 테스트를 빠르고 OS-독립적으로 유지한다.
