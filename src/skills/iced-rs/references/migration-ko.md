# iced Migration Notes: 0.12 → 0.13 → 0.14

`iced::Application`이나 `iced::Sandbox`를 import하는 `iced = "0.12"` 코드베이스가 있다면, 당신은 *오래된* API에 있다. 0.13은 둘 다 `iced::application()` 빌더 함수로 교체했다. 마이그레이션은 기계적이다. 이 페이지가 체크리스트이다.

## The Big Change: `Application` / `Sandbox` → `application()`

Old in 0.12:

```rust
use iced::{Application, Command, Element, Settings, Subscription, Theme};

struct Counter { value: i64 }

impl Application for Counter {
    type Message = Message;
    type Theme = Theme;
    type Executor = iced::executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (Self { value: 0 }, Command::none())
    }
    fn title(&self) -> String { "Counter".into() }
    fn update(&mut self, msg: Message) -> Command<Message> { /* ... */ }
    fn view(&self) -> Element<'_, Message> { /* ... */ }
}

fn main() -> iced::Result {
    Counter::run(Settings::default())
}
```

0.13은 첫 인자로 창 제목을 받는다. 초기 상태와 작업을 만드는 함수는 `run_with`에 전달한다.

```rust
fn main() -> iced::Result {
    iced::application(Counter::title, Counter::update, Counter::view)
        .theme(Counter::theme)
        .run_with(Counter::new)
}
```

0.14는 첫 인자로 앱 초기화 함수를 받는다. 창 제목은 빌더에서 설정한다.

```rust
use iced::{Element, Task, Theme, Subscription};

struct Counter { value: i64 }

impl Counter {
    fn new() -> (Self, Task<Message>) { /* ... */ }
    fn update(&mut self, msg: Message) -> Task<Message> { /* ... */ }
    fn view(&self) -> Element<'_, Message> { /* ... */ }
    fn title(&self) -> String { "Counter".into() }
    fn theme(&self) -> Theme { Theme::Dark }
}

fn main() -> iced::Result {
    iced::application(Counter::new, Counter::update, Counter::view)
        .title(Counter::title)
        .theme(Counter::theme)
        .run()
}
```

관찰:

- **No more trait**: `update`/`view`/`title`/`theme`이 일반 메서드가 된다. Rust 규칙이 적용된다; 연관 타입 없음.
- **`Command<T>` → `Task<T>`**: 동일한 의미, 이름만 바뀌었다. 코드베이스 전반에서 교체. `Command::none()` → `Task::none()`, `Command::perform` → `Task::perform`, `Command::batch` → `Task::batch`.
- 기존 `Executor` 연관 타입은 제거됐다. iced 0.14는 Cargo 기능으로 기본 네이티브 실행기를 선택한다. 프로젝트의 자체 실행기는 빌더의 `.executor` 메서드로 지정한다.
- `Flags`에 보관하던 설정값은 대상 버전의 초기화 함수에서 캡처한다.
- 기본 상태와 설정을 사용하는 작은 앱은 0.13에서 `iced::run(title, update, view)`, 0.14에서 `iced::run(update, view)`로 실행한다.

### Mechanical migration checklist

1. `impl Application for MyApp` / `impl Sandbox for MyApp`을 삭제한다.
2. 메서드를 유지하고; 시그니처를 변경한다:
   - `fn update(&mut self, msg) -> Command<Msg>` → `fn update(&mut self, msg) -> Task<Msg>`
   - 메서드 측의 다른 모든 것은 그대로.
3. 프로젝트가 고정한 버전에 맞춰 진입점을 선택한다. 다음 예시는 0.14 설정이다.
   ```rust
   fn main() -> iced::Result {
       iced::application(MyApp::new, MyApp::update, MyApp::view)
           .title(MyApp::title)       // only if you had one
           .theme(MyApp::theme)       // only if you had one
           .subscription(MyApp::subscription) // only if you had one
           .run()
       // or with settings: .settings(settings()).run()
   }
   ```
4. 글로벌 find-replace:
   - `Command::` → `Task::`
   - import에서 `Application,` 제거
   - import에서 `Sandbox,` 제거
   - import에서 `Executor,` 제거
5. `cargo check`: 컴파일러가 정확히 무엇이 남았는지 알려준다.

## Other Breaking Changes, 0.12 → 0.13

### `Row::with_children` / `Column::with_children`

이제 `impl IntoIterator<Item = Element>`를 받는다. `Vec`를 모으면 `into()`가 필요 없다.

```rust
// 0.12 (awkward)
Column::with_children(vec![a.into(), b.into(), c.into()])

// 0.13+
Column::with_children([a, b, c])          // or any IntoIterator
```

### `Length::Fill` / `Length::Shrink` / `Length::FillPortion`

이제 crate root에서 `iced::Fill`, `iced::Shrink`, `iced::FillPortion`로 re-export된다. 두 형식 모두 작동; 짧은 형식을 선호한다.

```rust
use iced::{Fill, FillPortion};

.width(Fill)
.width(FillPortion(2))
```

### `button::Appearance` → `button::Style`

중첩-레이어 스타일 API가 평탄화되었다. 동일한 필드인 background, text_color, border, shadow에 새 이름.

```rust
// 0.12
button::Appearance { background: ..., ..Default::default() }

// 0.13+
button::Style { background: ..., ..Default::default() }
```

`container`, `text_input`, `checkbox`, `pick_list`, `slider` 등에 적용된다. 모두 `Appearance` → `Style`.

### `theme::Button::*` enum variants → styler functions

이전은 `.style(theme::Button::Primary)`에 전달되는 enum variant였다. 새것은 함수 참조이다: `.style(button::primary)`. 더 이상 enum이 아니다.

```rust
// 0.12
button("Go").style(theme::Button::Primary)

// 0.13+
button("Go").style(button::primary)
```

text/container 등도 동일. 함수는 widget의 모듈에 산다. 예: `iced::widget::button::primary`.

### `f.area()` was never a thing: that's ratatui

iced에는 `f.area()`가 없다. 보인다면 그 코드는 ratatui이지 iced가 아니다. 둘을 똑바로 유지하라.

## 0.13 → 0.14

위 예시에 따라 진입점을 바꾸고 대상 버전에서 컴파일한다. 진행률을 스트리밍하는 앱은 [비동기 작업](tasks-ko.md)의 0.14 `Task::sip` 예시를 참조한다.

## 버전별 기능

| 기능 | iced 0.13 | iced 0.14 |
|---|---|---|
| `tokio` | Tokio 실행기 | Tokio 실행기 |
| `async-std` | 사용 가능 | 제거됨. 비동기 의존성과 호환되는 실행기 선택 |
| `debug` | 사용 가능. 이 기능을 꺼도 앱 빌더는 메시지에 `Debug`를 요구 | 개발 도구를 쓸 때 켜고 메시지에 `Debug` 구현 |

네이티브 iced 0.14의 기본 실행기는 `thread-pool`이다. `tokio`와 `smol`을 함께 켜면 기본 실행기로 Tokio가 선택된다.

## After Migration: Sanity Check

- [ ] `cargo check`로 앱 빌드
- [ ] 모든 `.style(theme::X::Y)` 참조가 이제 `.style(widget::style_fn)`이다
- [ ] 남은 `Command<Message>` 타입 없음
- [ ] `impl Application for` / `impl Sandbox for` 없음
- [ ] 네이티브 실행기 기능이 비동기 의존성과 호환되며 Linux 빌드에 `x11`이나 `wayland`가 포함됨
- [ ] 앱이 실행되고 theme 변경이 여전히 작동한다, palette 접근 회귀의 카나리

## When NOT to migrate

- 앱이 0.10 또는 0.11에 있다: 한 번에 0.14로 점프하지 말 것. 먼저 0.12로 가고, 그 다음 0.13. 모든 hop은 자체 errata가 있다.
- `iced_aw` 같은 서드파티 widget crate에 의존한다: *먼저* 그것이 대상 iced 버전을 지원하는지 확인한다. 이러한 crate는 때때로 lag한다.
