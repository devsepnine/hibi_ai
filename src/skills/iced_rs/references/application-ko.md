# iced Application Entry Points

iced는 두 가지 entry point를 제공한다: 작은 `iced::run`과 풀 빌더 `iced::application`. `update`/`view` 이상의 무언가가 필요하면 즉시 빌더를 사용한다.

## `iced::run` — smallest possible app

```rust
use iced::widget::{button, column, text};

pub fn main() -> iced::Result {
    iced::run(update, view)
}

fn update(count: &mut u64, msg: Message) { /* ... */ }
fn view(count: &u64) -> iced::Element<'_, Message> { /* ... */ }
```

**When**: 데모, 한 파일 실험, 테스트.
**When not**: theme, 윈도우 크기, subscription, 비동기 init, title이 필요한 경우. 그건 모두 빌더 영역이다.

`iced::run`의 state는 `Default`가 필요하다 (런타임이 초기 값을 구성할 수 있도록). state가 `Default`가 아니라면 `iced::application()`으로 옮긴다.

## `iced::application()` — the builder

```rust
use iced::{Element, Task, Theme, Subscription};

pub fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .title(App::title)
        .theme(App::theme)
        .subscription(App::subscription)
        .settings(settings())
        .centered()
        .run()
}
```

모든 빌더 메서드는 선택적이다; 해당하는 것만 chain한다. 각 빌더 메서드는 closure 또는 `&State`를 받는 함수 포인터를 받는다 — 둘 다 작동하니 더 깔끔하게 읽히는 쪽을 사용한다.

### Required triple

```rust
fn new() -> (App, Task<Message>)              // or: fn new(flags: Flags) -> (App, Task<Msg>)
fn update(&mut self, Message) -> Task<Message>
fn view(&self) -> Element<'_, Message>
```

`new`가 `(State, Task)`를 반환하므로 부트스트래핑 `Message::Init` 없이 앱이 초기 비동기 작업(설정 로드, 데이터 fetch)을 시작할 수 있다.

### Title

정적 문자열 또는 state의 함수:

```rust
.title("My App")                       // static
.title(|app: &App| format!("My App — {}", app.filename))   // dynamic
```

동적 title은 state가 변경될 때 자동으로 업데이트된다.

### Theme

```rust
.theme(|app: &App| app.theme.clone())
```

Built-ins: `Theme::Light`, `Theme::Dark`, `Theme::Dracula`, `Theme::Nord`, `Theme::SolarizedLight/Dark`, `Theme::GruvboxLight/Dark`, `Theme::CatppuccinLatte/Frappe/Macchiato/Mocha`, `Theme::TokyoNight/Storm/Light`, `Theme::KanagawaWave/Dragon/Lotus`, `Theme::Moonfly`, `Theme::Nightfly`, `Theme::Oxocarbon`, `Theme::Ferra`. `Theme::ALL`은 이 모두를 슬라이스로 제공한다 — `pick_list`에 유용하다.

커스텀 팔레트는 `references/theming.md` 참조.

### Subscription

```rust
.subscription(|app: &App| {
    Subscription::batch([
        time::every(Duration::from_secs(1)).map(|_| Message::Tick),
        event::listen().map(Message::Event),
    ])
})
```

Subscription은 반환되는 동안 실행된다. batch에서 조건부로 제외하여 subscription을 중지한다. `references/subscriptions.md` 참조.

## Window & Runtime Settings

```rust
use iced::{Size, Settings};
use iced::window;

fn settings() -> Settings {
    Settings {
        window: window::Settings {
            size: Size::new(1024.0, 720.0),
            position: window::Position::Centered,
            min_size: Some(Size::new(640.0, 480.0)),
            max_size: None,
            visible: true,
            resizable: true,
            decorations: true,
            transparent: false,
            icon: None,
            ..Default::default()
        },
        ..Default::default()
    }
}
```

`.settings(settings())`로 전달.

**Common knobs**:

| Setting | Use for |
|---|---|
| `size: Size::new(w, h)` | 초기 윈도우 크기 (logical pixels) |
| `position: Position::Centered` | 또는 정확한 배치를 위해 `Specific(Point)` |
| `min_size` / `max_size` | 리사이즈 제약 |
| `resizable` | 고정 크기 다이얼로그는 `false` |
| `decorations` | 커스텀 타이틀바는 `false` |
| `transparent` | 둥근 모서리 / acrylic 배경에 필요 |
| `icon` | `Some(window::icon::from_file_data(bytes, None)?)` |

**Don't set**: `flags`와 `id`를 직접 설정하지 않는다 — 빌더 형식을 사용한다; 런타임이 둘 다 관리한다.

### Builder-level window shortcuts

일부 윈도우 속성은 직접 `Settings`를 작성하는 것보다 명확한 first-class 빌더 메서드를 가진다:

```rust
iced::application(App::new, App::update, App::view)
    .window_size(Size::new(1024.0, 720.0))
    .centered()
    .resizable(false)
    .run()
```

앱이 한두 개의 override만 필요할 때 이를 사용한다. 세 개 이상이 필요하면 전체 `Settings`로 떨어진다.

## Headless / default-less state

`State`가 `Default`를 구현할 수 없다면 (`Connection`, `Runtime` 등을 보유), 명시적 `new`를 가진 `iced::application()`을 사용한다. `iced::run`의 경우, non-default 필드를 `Option<T>`로 옮기고 `new`에서 첫 `update`에 채워 넣어 state를 `Default`로 감싼다.

## Multi-window

0.13+는 서버 스타일 앱(시작 윈도우 없음)을 위한 `iced::daemon`과 명령형 윈도우 관리를 위한 `window::open` / `window::close`를 통해 다중 네이티브 윈도우를 지원한다. window handle은 메시지로 이동한다:

```rust
use iced::{window, Task};

fn update(&mut self, msg: Message) -> Task<Message> {
    match msg {
        Message::OpenDetails(id) => {
            let (_window_id, task) = window::open(window::Settings {
                size: Size::new(600.0, 400.0),
                ..Default::default()
            });
            task.map(move |window_id| Message::WindowOpened(window_id, id))
        }
        // ...
    }
}
```

진지한 멀티 윈도우 앱은 upstream `multi_window` 예제를 읽는 것이 가장 좋은 레퍼런스이다.

## When to pick which entry point

| Need | Use |
|---|---|
| Demo, test, one-file | `iced::run` |
| Theme, title, async init | `iced::application()` |
| Multi-window | `iced::application()` with `window::open` |
| No main window (tray app / background) | `iced::daemon()` |

빌더는 거의 over-engineering이 아니다; 사소한 앱조차도 `view`에 이름이 붙은 메서드의 이점을 누리며, 이는 미래의 "테마 추가" 변경을 사소하게 만든다.
