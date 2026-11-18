# iced Application Entry Points

다음 진입점은 iced 0.14 기준이다. 0.13을 사용하는 프로젝트는 [마이그레이션](migration-ko.md)에서 제목과 초기화 함수를 전달하는 위치를 확인한다.

## `iced::run`: smallest possible app

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

`iced::run`의 state는 `Default`가 필요하다. 런타임이 초기 값을 구성할 수 있어야 하기 때문이다. state가 `Default`가 아니라면 `iced::application()`으로 옮긴다.

## `iced::application()`: the builder

```rust
use iced::{Element, Task, Theme, Subscription};

pub fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .title(App::title)
        .theme(App::theme)
        .subscription(App::subscription)
        .window(window_settings())
        .run()
}
```

제목, 테마, 구독은 상태를 읽는 함수로 설정할 수 있다. 창 설정은 완성된 값을 빌더에 전달한다.

### 초기화, 갱신, 뷰

0.14 초기화 함수는 인자 없이 상태 또는 상태와 첫 작업을 반환한다. 설정값이 필요한 경우 인자 없는 클로저에서 캡처한다.

```rust
iced::application(move || App::new(flags.clone()), App::update, App::view)
```

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

Built-ins: `Theme::Light`, `Theme::Dark`, `Theme::Dracula`, `Theme::Nord`, `Theme::SolarizedLight/Dark`, `Theme::GruvboxLight/Dark`, `Theme::CatppuccinLatte/Frappe/Macchiato/Mocha`, `Theme::TokyoNight/Storm/Light`, `Theme::KanagawaWave/Dragon/Lotus`, `Theme::Moonfly`, `Theme::Nightfly`, `Theme::Oxocarbon`, `Theme::Ferra`. `Theme::ALL`은 이 모두를 슬라이스로 제공한다. `pick_list`에 유용하다.

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
use iced::{window, Size};

fn window_settings() -> window::Settings {
    window::Settings {
        size: Size::new(1024.0, 720.0),
        position: window::Position::Centered,
        min_size: Some(Size::new(640.0, 480.0)),
        ..Default::default()
    }
}
```

창 크기와 위치는 빌더의 `.window(window_settings())`로 전달한다. 기본 글꼴처럼 앱 전체에 적용할 값은 `.settings(...)`에 전달한다.

**Common knobs**:

| Setting | Use for |
|---|---|
| `size: Size::new(w, h)` | 초기 윈도우 크기, logical pixels |
| `position: Position::Centered` | 또는 정확한 배치를 위해 `Specific(Point)` |
| `min_size` / `max_size` | 리사이즈 제약 |
| `resizable` | 고정 크기 다이얼로그는 `false` |
| `decorations` | 커스텀 타이틀바는 `false` |
| `transparent` | 둥근 모서리 / acrylic 배경에 필요 |
| `icon` | 이미지 파일을 읽는 `window::icon::from_file_data`에는 iced의 `image` 기능이 필요하다. |

### Builder-level window shortcuts

일부 윈도우 속성은 직접 `Settings`를 작성하는 것보다 명확한 first-class 빌더 메서드를 가진다:

```rust
iced::application(App::new, App::update, App::view)
    .window_size(Size::new(1024.0, 720.0))
    .centered()
    .resizable(false)
    .run()
```

창 설정을 함께 전달할 때는 `window::Settings`를 사용한다.

## 초기화에 데이터가 필요한 상태

연결이나 설정값으로 상태를 생성하는 앱은 빌더에 초기화 함수를 전달한다. 초기화 함수는 첫 작업도 함께 반환할 수 있다.

## Multi-window

0.13+는 시작 윈도우가 없는 서버 스타일 앱을 위한 `iced::daemon`과 명령형 윈도우 관리를 위한 `window::open` / `window::close`를 통해 다중 네이티브 윈도우를 지원한다. window handle은 메시지로 이동한다:

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
| No main window, such as a tray app or background app | `iced::daemon()` |

빌더는 거의 over-engineering이 아니다; 사소한 앱조차도 `view`에 이름이 붙은 메서드의 이점을 누리며, 이는 미래의 "테마 추가" 변경을 사소하게 만든다.
