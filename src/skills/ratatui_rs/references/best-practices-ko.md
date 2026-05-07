# TUI Best Practices

프로덕션 TUI에서 입증된 패턴이다. 초점은 구조이다: 10개 이상의 화면을 지나서도 유지보수 가능한 상태로 유지되도록 사소하지 않은 앱을 어떻게 구성하는가.

## Module Layout for a Non-Trivial TUI

`main.rs`를 얇게 유지한다. 올바른 경로에 있다는 신호: `main.rs`는 대부분 터미널 setup + 이벤트 루프 디스패치이며, ~100 줄. 다른 모든 것은 서브모듈에 산다.

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

**Rule of thumb**: `ui/`의 파일은 절대 thread를 spawn하거나, 디스크에 쓰거나, `App`을 mutate하지 않는다. `&App`만 읽고 렌더링한다. 이는 UI를 `TestBackend`로 테스트 가능하게 만들고 필요하면 나중에 renderer를 swap할 수 있게 한다.

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

**단일 struct인 이유**: render fn에 `&App`을 전달하기 쉽고, 테스트용 스냅샷을 잡기 쉽다.
**view에 enum인 이유**: 화면을 추가할 때 컴파일러가 누락된 분기를 잡는다. Boolean (`is_modal_open`, `is_loading`)은 잘못된 조합을 조용히 허용한다.

## Channels: Bundle Them, Don't Pass Loose

루프가 백그라운드 thread로부터 들어야 할 때, 모든 채널을 단일 struct로 그룹화하여 함수 시그니처를 깨끗하게 유지한다:

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

job별 취소 격리를 위해 `references/cancelable-processes.md` 참조.

## Keyboard Shortcuts (Discoverable Defaults)

vi / less / ranger / lazygit에서 사용자가 이미 아는 binding을 선택한다. 발명하지 마라.

| Key | Action | Notes |
|---|---|---|
| `q`, `Esc` | Quit / Back | `q`는 최상위에서 앱을 종료한다; `Esc`는 모달에서 빠져나온다. |
| `j` / `k` / arrows | Down / Up | 두 형태 모두 — vi 사용자와 화살표 사용자가 공존한다. |
| `h` / `l` | Left / Right (또는 back/forward) | |
| `g` / `G` | Top / Bottom | |
| `Enter` | Confirm / Open | |
| `Space` | Toggle selection | 화면이 체크리스트 같을 때. |
| `/` | Search/filter | |
| `?` | Show help | |
| `Tab` / `Shift+Tab` | 폼의 Next / Prev field | |
| `Ctrl+C` | Force quit | 작업 중이라도 `q`처럼 취급; cleanup이 실행되어야 한다. |

상태 표시줄에 활성 binding을 표시한다 — discoverability가 절반의 싸움이다.

## Status Bar (Always Visible)

mode, item count, "?: help" 힌트가 있는 footer 상태 표시줄이 사용자를 훈련시킨다:

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

터미널 크기에 적응한다. 가장 유용한 단일 임계값은 "사이드바를 위한 공간이 있는가?"이다:

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

곳곳에 `Color::Cyan`을 하드코딩하지 마라. 한 곳에 의미적 색을 정의하여 light/dark 터미널을 지원할 수 있도록 한다:

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

그러면 UI 코드는 `app.theme.accent`를 읽고, 절대로 `Color::Cyan` 리터럴을 읽지 않는다.

## Animation Frame Counter

스피너의 경우, 루프 반복마다 tick하는 정수를 저장한다; draw 시 glyph로 매핑한다:

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

렌더 레이어는 절대 wall-clock 시간을 소유하지 않는다 — 테스트하기 더 쉽다.

## Error Handling

앱 함수에서 `anyhow::Result<T>`를 반환한다; 에러를 panic이 아닌 일시적 toast나 인라인 status로 보여준다:

```rust
match install_item(item) {
    Ok(()) => app.toast("Installed"),
    Err(e) => app.toast(format!("Failed: {e}")),
}
```

앱을 종료해야 하는 복구 불가능한 에러는 `app.should_quit = true`를 설정하고 루프가 `ratatui::restore()` 후에 출력하도록 한다.

## Performance: When (Not) to Care

ratatui의 renderer는 빠르다. 측정할 때까지 최적화하지 마라. 두 가지가 *진짜로* 중요하다:

1. **위젯 데이터의 빌드 비용**: 매 프레임마다 1만 항목 리스트를 재빌드한다면 hot이다. 준비된 `Vec<ListItem>`을 캐시하고 항목이 변경될 때만 재빌드한다.
2. **폴링 간격**: 대부분의 앱에 100ms가 괜찮다; 애니메이션 헤비는 16ms. 16ms 아래로 가지 마라 — 어쨌든 터미널 refresh를 넘어선다.

## Accessibility

- 높은 대비: 라이트 그레이-온-화이트나 옐로-온-화이트를 피한다.
- 색에만 의존하지 마라: 기호와 짝짓는다 (`✓ ok`, `✗ failed`, `▶ running`).
- 항상 키보드 접근 가능: 마우스가 사용 불가능하다고 가정한다.
- 예측 가능한 focus 표시기: `> `, `█`, 또는 배경 하이라이트 — 하나를 선택하고 고수한다.

## Ship Checklist

- [ ] `ratatui::init()` / `ratatui::restore()` (또는 panic hook)
- [ ] 모든 긴 작업 cancelable (`references/cancelable-processes.md`)
- [ ] Windows + macOS + Linux에서 테스트됨 (`references/cross-platform.md`)
- [ ] 사소하지 않은 화면당 적어도 하나의 렌더링 테스트 (`references/testing.md`)
- [ ] 상태 표시줄에 키 binding 표시
- [ ] 에러는 panic이 아닌 toast로 표시
- [ ] `cargo build --release`로 경고 없이 빌드
