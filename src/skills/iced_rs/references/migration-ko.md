# iced Migration Notes (0.12 → 0.13 → 0.14)

`iced::Application`이나 `iced::Sandbox`를 import하는 `iced = "0.12"` 코드베이스가 있다면, 당신은 *오래된* API에 있다. 0.13은 둘 다 `iced::application()` 빌더 함수로 교체했다. 마이그레이션은 기계적이다 — 이 페이지가 체크리스트이다.

## The Big Change: `Application` / `Sandbox` → `application()`

Old (0.12):

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

New (0.13+):

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
- **`Executor`**: 사용자 API에서 사라졌다. executor는 `tokio` / `smol` feature 플래그에 의해 선택된다.
- **`Flags`**: 사라졌다. 초기 state는 `new()`의 반환에 살고; 이전에 `Flags`에 살던 모든 구성은 이제 `application()`에 전달되는 `new` 주변의 closure capture에 산다.
- **`Sandbox`**: 완전히 제거되었다. `Sandbox`(no async)를 사용하고 있었다면, 마이그레이션 대상은 여전히 `iced::application()`이다 — 그저 `Task::perform`을 사용하지 않을 뿐. 또는 *매우* 최소한의 경우 `iced::run(update, view)`를 사용한다.

### Mechanical migration checklist

1. `impl Application for MyApp` / `impl Sandbox for MyApp`을 삭제한다.
2. 메서드를 유지하고; 시그니처를 변경한다:
   - `fn update(&mut self, msg) -> Command<Msg>` → `fn update(&mut self, msg) -> Task<Msg>`
   - 메서드 측의 다른 모든 것은 그대로.
3. `main`을 교체:
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
5. `cargo check` — 컴파일러가 정확히 무엇이 남았는지 알려준다.

## Other Breaking Changes (0.12 → 0.13)

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

중첩-레이어 스타일 API가 평탄화되었다. 동일한 필드 (background, text_color, border, shadow), 새 이름.

```rust
// 0.12
button::Appearance { background: ..., ..Default::default() }

// 0.13+
button::Style { background: ..., ..Default::default() }
```

`container`, `text_input`, `checkbox`, `pick_list`, `slider` 등에 적용된다 — 모두 `Appearance` → `Style`.

### `theme::Button::*` enum variants → styler functions

이전은 `.style(theme::Button::Primary)`에 전달되는 enum variant였다. 새것은 함수 참조이다: `.style(button::primary)`. 더 이상 enum이 아니다.

```rust
// 0.12
button("Go").style(theme::Button::Primary)

// 0.13+
button("Go").style(button::primary)
```

text/container 등도 동일. 함수는 widget의 모듈에 산다 (`iced::widget::button::primary`).

### `f.area()` was never a thing (that's ratatui)

iced에는 `f.area()`가 없다. 보인다면 그 코드는 ratatui이지 iced가 아니다. 둘을 똑바로 유지하라.

## 0.13 → 0.14

훨씬 작다. 주요 추가:

- **`Task::sip`** — 취소와 함께 진행을 스트리밍한다. `references/tasks.md` 참조.
- **`iced::daemon`** — 메인 윈도우 없는 백그라운드 앱.
- **Multi-window 개선**: `window::open`이 `(window::Id, Task<Message>)`를 반환한다.
- **`text_editor`**가 더 풍부한 액션과 undo group을 얻었다.
- **More themes** (KanagawaWave 등 — 일부 릴리스에서는 0.14 이전에 있었음; 버전 확인).

Breaking change는 드물다; 대부분의 0.13 코드는 변경 없이 컴파일된다. 릴리스 노트는 짧다 — 블로그 포스트를 읽는 대신 훑어본다.

## Feature Flag Renames

| 0.12 feature | 0.13+ feature |
|---|---|
| `tokio` | `tokio` (unchanged) |
| `async-std` | 제거됨 — `smol` 사용 (`async-std` crate는 위에서 작동) |
| `debug` | 기본값에서 제거됨; 명시적 feature 불필요 |

`Cargo.toml`이 `iced/async-std`를 참조한다면 제거한다 — 이제 `tokio`나 `smol`에 있다.

## After Migration — Sanity Check

- [ ] `cargo check`로 앱 빌드
- [ ] 모든 `.style(theme::X::Y)` 참조가 이제 `.style(widget::style_fn)`이다
- [ ] 남은 `Command<Message>` 타입 없음
- [ ] `impl Application for` / `impl Sandbox for` 없음
- [ ] `Cargo.toml`이 정확히 `tokio` / `smol` 중 하나를 지정한다
- [ ] 앱이 실행되고 theme 변경이 여전히 작동한다 (palette 접근 회귀의 카나리)

## When NOT to migrate

- 앱이 0.10 또는 0.11에 있다: 한 번에 0.14로 점프하지 말 것. 먼저 0.12로 가고, 그 다음 0.13. 모든 hop은 자체 errata가 있다.
- 서드파티 widget crate (`iced_aw` 등)에 의존한다: *먼저* 그것이 대상 iced 버전을 지원하는지 확인한다. 이러한 crate는 때때로 lag한다.
