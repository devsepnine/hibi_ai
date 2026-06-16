---
name: ratatui
description: Rust terminal UI (TUI) with ratatui/crossterm — widgets, ListState/TableState selection, layouts, keyboard navigation, event loops with background-task cancel, panic-safe setup, and 0.28→0.30 migration. 러스트 터미널 UI, TUI, 대화형 CLI, 이벤트 루프, 백그라운드 작업 취소, ratatui 마이그레이션. NOT for ncurses, Python rich/textual, GUI (Iced/egui), or Bubble Tea (Go).
keywords: [ratatui, tui, 터미널ui, rust-tui, cli, crossterm, cancelable, 취소, cross-platform, 크로스플랫폼, listState, migration, 마이그레이션]
---

# Ratatui (Rust TUI) — 프로덕션 가이드

Rust용 immediate-mode 터미널 UI 프레임워크. 이 skill은 프로덕션에서 검증된 패턴 (취소 가능한 백그라운드 작업, 크로스 플랫폼 처리, panic-safe 터미널 복원) 을 인코딩한다 — hello-world가 아니다.

## 어떤 reference를 언제 사용할지

이 SKILL.md는 인덱스다. 작업에 맞는 집중 reference를 읽는다:

| Task | Reference |
|---|---|
| 위젯 사용 (List/Table/Paragraph/Gauge/Tabs/...) | `references/widgets.md` |
| 백그라운드 작업 + 사용자 취소 가능 spawn | `references/cancelable-processes.md` |
| Windows 경로/명령 특이사항, MSYS 변환 | `references/cross-platform.md` |
| `TestBackend`로 렌더링 단위 테스트 | `references/testing.md` |
| TUI 흔한 함정 회피 (panic, eprintln, JoinHandle) | `references/gotchas.md` |
| 앱 구조, 모듈, state machine, channel | `references/best-practices.md` |

## 프로젝트 설정

panic-safe 터미널 처리를 위해 Edition 2024 (Rust 1.85+) 와 `ratatui::init()` (0.28+ 추가) 를 사용한다.

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

## 최소 앱 (관용적 0.30)

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

**수동 `Terminal::new` 대신 `ratatui::init()`을 쓰는 이유**: 앱이 렌더링 중간에 panic해도 터미널을 복원하는 panic hook을 설치한다. 이게 없으면 panic이 터미널을 raw mode로 남겨두어 사용자는 자신이 입력하는 것을 볼 수 없게 된다.

## 레이아웃 (0.30 destructuring 스타일)

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

비례 잔여 공간에는 `Constraint::Fill(n)` (0.27+) 을 사용한다 — `Min(0)`보다 의도가 명확하다.

## 백그라운드 채널이 있는 이벤트 루프

키 입력 AND 백그라운드 이벤트 (timer, async I/O, 자식 프로세스 출력) 모두에서 TUI가 업데이트되어야 할 때의 패턴:

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

**취소 가능한** 자식 프로세스가 있는 프로덕션급 버전은 `references/cancelable-processes.md` 참조.

## 상태 관리 (View enum 패턴)

비자명한 앱은, 가능한 화면을 enum으로 모델링해 전이가 타입 체크되도록 한다:

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

이유: 이 패턴 없는 13-state TUI는 `if/else`의 쥐 둥지가 된다. 이 패턴이 있으면 화면을 추가할 때 컴파일러가 누락된 분기를 잡아낸다.

## 커스텀 위젯 (Widget trait)

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

커스텀 위젯은 `draw(...)` 콜백을 짧게 유지하고 `TestBackend`로 렌더링을 단위 테스트할 수 있게 한다 (`references/testing.md` 참조).

## 성능 빠른 규칙

1. **Block 말고 Poll** — 백그라운드 채널을 비울 수 있도록 `event::poll(Duration::from_millis(50–100))`을 사용한다.
2. **`tick()` ≠ 재 그리기** — 상태가 실제로 바뀐 경우에만 재 그리기; ratatui의 더블 버퍼는 저렴하지만, build_ui 클로저는 그렇지 않다.
3. **raw mode 켜진 동안 `eprintln!` 금지** — alt screen이 망가진다. `references/gotchas.md` 참조.
4. **detach vs join 스레드를 신중히** — `drop(JoinHandle)`은 join 하지 않는다. `references/gotchas.md` 참조.

## 마이그레이션 노트 (0.28 → 0.30)

- 수동 `enable_raw_mode` + `EnterAlternateScreen` 대신 `ratatui::init()` / `ratatui::restore()` 선호.
- `Block::default().borders(Borders::ALL)` 대신 `Block::bordered()` 선호.
- `.split(area)` 인덱싱 대신 `Layout::vertical([...]).areas(area)` destructuring 선호.
- 잔여 공간에는 `Constraint::Min(0)` 대신 `Constraint::Fill(1)` 선호.
- `f.size()` → `f.area()`.

## 관련 skill

- 일반 Rust 관용구: `rust-best-practices`
- 크로스 플랫폼 셸 안전성: `references/cross-platform.md` (이 skill) + `coding-standards`
