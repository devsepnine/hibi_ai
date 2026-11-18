# iced Migration Notes: 0.12 → 0.13 → 0.14

If you have a codebase on `iced = "0.12"` that imports `iced::Application` or `iced::Sandbox`, you are on the *old* API. 0.13 replaced both with the `iced::application()` builder function. The migration is mechanical. This page is the checklist.

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

In 0.13 the first argument is the title. Supply the initial state and task to `run_with`.

```rust
fn main() -> iced::Result {
    iced::application(Counter::title, Counter::update, Counter::view)
        .theme(Counter::theme)
        .run_with(Counter::new)
}
```

In 0.14 the first argument initializes the application. Set the title on the builder.

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

Observations:

- **No more trait**: `update`/`view`/`title`/`theme` become plain methods. Rust rules apply; no associated types.
- **`Command<T>` → `Task<T>`**: identical semantics, renamed. Replace across the codebase. `Command::none()` → `Task::none()`, `Command::perform` → `Task::perform`, `Command::batch` → `Task::batch`.
- The old `Executor` associated type is gone. iced 0.14 selects the default native executor by Cargo features. Use the builder’s `.executor` method when the project supplies its own.
- Capture configuration previously held in `Flags` in the initializer for your target version.
- For a small app with default state and settings, 0.13 takes `iced::run(title, update, view)` and 0.14 takes `iced::run(update, view)`.

### Mechanical migration checklist

1. Delete `impl Application for MyApp` / `impl Sandbox for MyApp`.
2. Keep the methods; change signatures:
   - `fn update(&mut self, msg) -> Command<Msg>` → `fn update(&mut self, msg) -> Task<Msg>`
   - Everything else unchanged on the method side.
3. Choose the entry point for your pinned version. This example configures 0.14.
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
4. Global find-replace:
   - `Command::` → `Task::`
   - `Application,` removed from imports
   - `Sandbox,` removed from imports
   - `Executor,` removed from imports
5. `cargo check`: compiler tells you exactly what remains.

## Other Breaking Changes, 0.12 → 0.13

### `Row::with_children` / `Column::with_children`

Now take `impl IntoIterator<Item = Element>`. Collects a `Vec` don't need `into()`.

```rust
// 0.12 (awkward)
Column::with_children(vec![a.into(), b.into(), c.into()])

// 0.13+
Column::with_children([a, b, c])          // or any IntoIterator
```

### `Length::Fill` / `Length::Shrink` / `Length::FillPortion`

Now re-exported at crate root as `iced::Fill`, `iced::Shrink`, `iced::FillPortion`. Both forms work; prefer the short form.

```rust
use iced::{Fill, FillPortion};

.width(Fill)
.width(FillPortion(2))
```

### `button::Appearance` → `button::Style`

The nested-layer style API was flattened. Same fields, namely background, text_color, border, shadow, under a new name.

```rust
// 0.12
button::Appearance { background: ..., ..Default::default() }

// 0.13+
button::Style { background: ..., ..Default::default() }
```

Applies to `container`, `text_input`, `checkbox`, `pick_list`, `slider`, etc. All `Appearance` → `Style`.

### `theme::Button::*` enum variants → styler functions

Old was an enum variant passed to `.style(theme::Button::Primary)`. New is a function reference: `.style(button::primary)`. No more enum.

```rust
// 0.12
button("Go").style(theme::Button::Primary)

// 0.13+
button("Go").style(button::primary)
```

Same for text/container/etc. The functions live in the widget's module, for example `iced::widget::button::primary`.

### `f.area()` was never a thing: that's ratatui

iced doesn't have `f.area()`. If you see that, the code is ratatui, not iced. Keep them straight.

## 0.13 → 0.14

Update the entry point as shown above and compile against the target version. For apps that stream progress, see the 0.14 `Task::sip` example in [Tasks](tasks.md).

## Features by version

| Feature | iced 0.13 | iced 0.14 |
|---|---|---|
| `tokio` | Tokio executor | Tokio executor |
| `async-std` | Available | Removed. Choose an executor compatible with the async dependencies |
| `debug` | Available. The application builder requires a `Debug` message even with this feature disabled | Enable for developer tools and implement `Debug` on the message |

Native iced 0.14 defaults to `thread-pool`. If both `tokio` and `smol` are enabled, the default executor selects Tokio.

## After Migration: Sanity Check

- [ ] App builds with `cargo check`
- [ ] All `.style(theme::X::Y)` references are now `.style(widget::style_fn)`
- [ ] No remaining `Command<Message>` types
- [ ] No `impl Application for` / `impl Sandbox for`
- [ ] Native executor features match the async dependencies and Linux builds include `x11` or `wayland`
- [ ] App runs and a theme change still works, a canary for palette access regressions

## When NOT to migrate

- The app is on 0.10 or 0.11: don't jump to 0.14 in one shot. Go to 0.12 first, then 0.13. Every hop has its own errata.
- You rely on a third-party widget crate such as `iced_aw`: check whether it supports your target iced version *first*. These crates sometimes lag.
