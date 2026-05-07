---
name: iced
description: Use this skill for any task involving `iced`, the Rust GUI framework. Trigger whenever the user mentions iced by name in a Rust, GUI, or desktop-app context — no matter the sub-task: writing a new iced app, adding a widget/feature, debugging a compile error, migrating between iced versions (0.12 → 0.13/0.14, `Application`/`Sandbox` traits → `iced::application()` builder, `Command` → `Task`, styling API changes), explaining any iced API (`Message`/`update`/`view`/`Element`, `Task::perform`, `Subscription`, `canvas::Program`, `window::Settings`, `Theme`/`extended_palette`, widget styles), wiring async work, or choosing iced over Electron/Tauri for a desktop project. Assume iced-flavored intent whenever the user says "iced 앱", "iced::", "러스트 iced", "iced 마이그레이션", pastes iced code, or asks how to do X in iced. Do NOT use for other Rust GUIs (egui, Slint, Dioxus, gtk-rs, Tauri), ratatui (terminal — separate skill), web frameworks, or Rust work without a GUI.
keywords: [iced, rust-gui, elm, elm-architecture, widget, application, task, subscription, canvas, theme, palette, migration, 마이그레이션, 러스트gui, 데스크톱ui]
---

# iced (Rust GUI) — 프로덕션 가이드

Rust용 retained-mode, Elm 아키텍처 GUI 라이브러리. 하나의 `Message` enum, 하나의 `update`, 하나의 `view`로 런타임이 루프를 구동한다. 이 skill은 0.13/0.14 관용구 — 구 `Application`/`Sandbox` trait를 대체한 것 — 와 실제 앱에서 검증된 async, 테마, canvas 패턴을 인코딩한다.

## 어떤 reference를 언제 사용할지

이 SKILL.md는 인덱스다. 하위 작업에 맞는 집중 reference를 연다:

| Task | Reference |
|---|---|
| 앱 진입점: `iced::run` vs `iced::application()` builder, `Settings`, `window::Settings` | `references/application.md` |
| 위젯 선택 (text/button/column/row/container/text_input/scrollable/pick_list/checkbox/slider/toggler/tooltip) | `references/widgets.md` |
| 비동기 작업: `Task::perform`, `Task::sip`, `Task::batch`, 체이닝, 취소, `iced::exit()` | `references/tasks.md` |
| 런타임 이벤트: `time::every`, `event::listen`, 키보드/마우스, subscription 배칭 | `references/subscriptions.md` |
| 스타일링: 내장 styler (`button::primary`, `container::rounded_box`), 커스텀 스타일 클로저, `Theme::ALL`, `extended_palette()` | `references/theming.md` |
| 커스텀 그리기: `canvas::Program`, `Cache`, `Path`, `Frame::fill/stroke/fill_text`, `Geometry` | `references/canvas.md` |
| 0.12 (`Application`/`Sandbox` trait) → 0.13/0.14 builder API 마이그레이션 | `references/migration.md` |
| iced 흔한 함정 회피 (Element 라이프타임, Message Clone, 런타임 선택, view 변동) | `references/gotchas.md` |

## 프로젝트 설정

edition 2024 (Rust 1.85+) 와 현재 stable `iced`를 사용한다. builder API (`iced::application`) 는 99%의 앱을 커버한다; `iced_runtime`은 고급 임베드용으로 남겨둔다.

```toml
[package]
name = "my-iced-app"
edition = "2024"
rust-version = "1.85"

[dependencies]
iced = { version = "0.14", features = ["tokio", "canvas", "advanced"] }
# For 0.13, just pin "0.13". API is compatible for everything in this skill.
```

알아둘 만한 feature flag:

| Feature | What it adds |
|---|---|
| `tokio` | `Task::perform`을 위해 tokio를 async 런타임으로 사용 |
| `smol` | 대안 런타임; 정확히 하나만 선택 |
| `canvas` | `canvas::Program`, `Geometry`, `Path`, `Frame` |
| `advanced` | 커스텀 위젯용 저수준 `widget::Widget` trait |
| `image` / `svg` | 비트맵 / 벡터 이미지 위젯 |
| `markdown` | `iced::widget::markdown`로 마크다운 렌더 |

`tokio`/`smol` 중 **하나**만 선택한다. 둘 다 활성화하면 의존성을 낭비하고 라이브러리가 다른 것을 선택하면 런타임 충돌을 일으킬 수 있다.

## 최소 앱 — `iced::run` (state 구조체 불필요)

가장 작은 합법적 iced 앱. state는 그냥 `u64`도 된다; iced는 구조체를 강제하지 않는다.

```rust
use iced::widget::{button, column, text};

pub fn main() -> iced::Result {
    iced::run(update, view)
}

#[derive(Debug, Clone)]
enum Message {
    Increment,
}

fn update(counter: &mut u64, message: Message) {
    match message {
        Message::Increment => *counter += 1,
    }
}

fn view(counter: &u64) -> iced::Element<'_, Message> {
    column![
        text(counter).size(50),
        button("+").on_press(Message::Increment),
    ]
    .into()
}
```

**이게 동작하는 이유**: `iced::run`은 어떤 `State: Default`와 순수 `update(&mut State, Msg)`, `view(&State) -> Element`도 받는다. 데모와 작은 도구에 좋다. 테마, subscription, async 작업, 윈도우 설정이 필요해지는 순간 `iced::application()`으로 업그레이드한다.

## 표준 앱 — `iced::application()` builder

거의 모든 것에 대한 관용적 선택. 각 builder 메서드는 선택적이다; 필요한 것만 체이닝한다.

```rust
use iced::widget::{button, column, text};
use iced::{Element, Task, Theme, Subscription};

pub fn main() -> iced::Result {
    iced::application(Counter::new, Counter::update, Counter::view)
        .title("Counter")
        .theme(Counter::theme)
        .subscription(Counter::subscription)
        .centered()
        .run()
}

#[derive(Default)]
struct Counter { value: i64 }

#[derive(Debug, Clone, Copy)]
enum Message { Increment, Decrement }

impl Counter {
    fn new() -> (Self, Task<Message>) {
        (Self::default(), Task::none())
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Increment => self.value += 1,
            Message::Decrement => self.value -= 1,
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        column![
            button("+").on_press(Message::Increment),
            text(self.value).size(50),
            button("-").on_press(Message::Decrement),
        ]
        .padding(20)
        .into()
    }

    fn theme(&self) -> Theme { Theme::Dark }
    fn subscription(&self) -> Subscription<Message> { Subscription::none() }
}
```

**핵심 형태**: `new`는 `(Self, Task<Message>)`를 반환해 앱이 특별한 "init" 메시지 없이 초기 async 로드 (예: config 읽기) 를 발사할 수 있다. `update`도 `Task<Message>`를 반환한다 — 런타임이 실행하고 결과가 다른 `Message`로 돌아오는 Elm 스타일 "command".

윈도우 설정 (사이즈, 위치, min/max, decoration) 과 다중 윈도우 앱을 포함한 세부사항은 `references/application.md`에 있다.

## 레이아웃 필수 사항

`column!`과 `row!`는 이질적 자식을 위한 매크로다; `.function_form` 변형은 자식이 동적으로 만들어질 때 `Vec<Element>`를 받는다.

```rust
use iced::widget::{column, row, container, text};
use iced::{Fill, Center, Element};

fn view(state: &State) -> Element<'_, Message> {
    container(
        column![
            text("Header").size(24),
            row![text("Left"), text("Right")].spacing(20),
            text("Footer"),
        ]
        .spacing(15)
        .padding(20)
        .align_x(Center)
    )
    .center_x(Fill)
    .center_y(Fill)
    .into()
}
```

**Length primitive**: `Fill` (남은 공간 모두 차지), `Shrink` (콘텐츠에 맞춤), `FillPortion(n)` (비례 분할), 또는 그냥 `f32` (고정 픽셀). 윈도우 리사이즈에서 레이아웃이 살아남도록 하드코딩된 너비보다 `Fill`/`FillPortion`을 선호한다.

더 많은 패턴 — `scrollable`, `pick_list`, `tooltip`, `text_input`, `canvas` — 은 `references/widgets.md`에 있다.

## Message 규율

Message enum이 앱의 척추다. 유지보수성을 유지하는 세 가지 규칙:

1. **모든 Message에 `#[derive(Debug, Clone)]`.** 런타임이 이벤트 경계를 가로질러 메시지를 클론한다. 페이로드가 허용하면 `Copy` — iced가 잘 사용한다.
2. **사용자가 관찰 가능한 이벤트당 Message variant 하나**, 내부 전이당 하나가 아니다. `ThemeChanged(Theme)` > `InternalMutationA/B/C`.
3. **하위 모듈의 메시지를 평탄화하지 말고 variant 안에 중첩**시킨다. `Message::Editor(editor::Message)`가 30 variant에서 `Message::EditorKeyPressed / Message::EditorSaved / ...`보다 잘 확장된다.

```rust
#[derive(Debug, Clone)]
enum Message {
    Editor(editor::Message),
    FileOpened(Result<String, String>),
    ThemeChanged(Theme),
}
```

## 비동기 작업 — `Task::perform`

`update`는 `Task<Message>`를 반환한다. `Task::perform(future, on_complete)`로 async 작업을 발행한다; 결과는 메시지로 돌아온다.

```rust
use iced::Task;

fn update(app: &mut App, message: Message) -> Task<Message> {
    match message {
        Message::Load => Task::perform(load_config(), Message::Loaded),
        Message::Loaded(Ok(cfg)) => { app.cfg = cfg; Task::none() }
        Message::Loaded(Err(e)) => { app.error = Some(e); Task::none() }
    }
}
```

스트리밍 진행률, 취소, batch, 시퀀싱은 `references/tasks.md` 참조.

## 스타일링 한눈에

내장 styler가 대부분의 케이스를 커버한다; 커스텀은 `(theme, status) -> Style` 클로저다.

```rust
button("Save").style(button::primary);
container("Card").style(container::rounded_box);

// Custom: react to hover status
button("Delete").style(|theme, status| {
    let palette = theme.extended_palette();
    match status {
        button::Status::Hovered => button::Style {
            background: Some(palette.danger.strong.color.into()),
            ..button::danger(theme, status)
        },
        _ => button::danger(theme, status),
    }
});
```

Palette role (`primary`/`secondary`/`success`/`danger`/`background`) × strength (`base`/`strong`/`weak`) × what (`color`/`text`) — 모두 `theme.extended_palette()`를 통해. 전체 reference: `references/theming.md`.

## 성능 빠른 규칙

1. **`view`는 저렴하게 유지** — state가 변할 때마다 매 프레임 실행된다. 핫 루프 안에서 할당이나 포맷팅을 하지 말 것; `update`에서 미리 계산하고 `App`에 캐싱한다.
2. **`view`에서 borrow, `update`에서 own.** `Element<'a, Message>`는 `&self`에서 빌리므로, 비싼 owned 데이터를 App 구조체로 옮기고 builder에 참조를 노출한다.
3. **커스텀 그리기에는 `canvas::Cache`** 사용 — `Cache::draw`는 `.clear()`를 호출할 때까지 재 tessellation을 건너뛴다.
4. **레이아웃을 과도하게 중첩하지 말 것.** 각 `container`/`column`/`row`는 노드다; 5단계 깊이는 OK, 15는 경고 신호.
5. **`Task::batch`를 spawn보다 선호**; 런타임이 이미 병렬화한다. `update` 안에서 `tokio::spawn` 하지 말 것 — 메시지 배관을 잃는다.

## 관련 skill

- 일반 Rust 관용구: `rust-best-practices`
- UI가 터미널이라면: `ratatui` (이 프로젝트)
- iced 상태 로직과 공유되는 테스팅/에러 패턴: `rust-best-practices` / `tdd-workflow`
