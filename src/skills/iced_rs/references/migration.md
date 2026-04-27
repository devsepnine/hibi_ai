# iced Migration Notes (0.12 → 0.13 → 0.14)

If you have a codebase on `iced = "0.12"` that imports `iced::Application` or `iced::Sandbox`, you are on the *old* API. 0.13 replaced both with the `iced::application()` builder function. The migration is mechanical — this page is the checklist.

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

Observations:

- **No more trait**: `update`/`view`/`title`/`theme` become plain methods. Rust rules apply; no associated types.
- **`Command<T>` → `Task<T>`**: identical semantics, renamed. Replace across the codebase. `Command::none()` → `Task::none()`, `Command::perform` → `Task::perform`, `Command::batch` → `Task::batch`.
- **`Executor`**: gone from user API. The executor is chosen by the `tokio` / `smol` feature flag.
- **`Flags`**: gone. Initial state lives in `new()`'s return; any configuration that used to live in `Flags` now lives in a closure capture around the `new` passed to `application()`.
- **`Sandbox`**: flat-out removed. If you were using `Sandbox` (no async), the migration target is still `iced::application()` — you just don't use `Task::perform`. Or use `iced::run(update, view)` for the *very* minimal case.

### Mechanical migration checklist

1. Delete `impl Application for MyApp` / `impl Sandbox for MyApp`.
2. Keep the methods; change signatures:
   - `fn update(&mut self, msg) -> Command<Msg>` → `fn update(&mut self, msg) -> Task<Msg>`
   - Everything else unchanged on the method side.
3. Replace the `main`:
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
5. `cargo check` — compiler tells you exactly what remains.

## Other Breaking Changes (0.12 → 0.13)

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

The nested-layer style API was flattened. Same fields (background, text_color, border, shadow), new name.

```rust
// 0.12
button::Appearance { background: ..., ..Default::default() }

// 0.13+
button::Style { background: ..., ..Default::default() }
```

Applies to `container`, `text_input`, `checkbox`, `pick_list`, `slider`, etc. — all `Appearance` → `Style`.

### `theme::Button::*` enum variants → styler functions

Old was an enum variant passed to `.style(theme::Button::Primary)`. New is a function reference: `.style(button::primary)`. No more enum.

```rust
// 0.12
button("Go").style(theme::Button::Primary)

// 0.13+
button("Go").style(button::primary)
```

Same for text/container/etc. The functions live in the widget's module (`iced::widget::button::primary`).

### `f.area()` was never a thing (that's ratatui)

iced doesn't have `f.area()`. If you see that, the code is ratatui, not iced. Keep them straight.

## 0.13 → 0.14

Much smaller. The main additions are:

- **`Task::sip`** — stream progress with cancellation. See `references/tasks.md`.
- **`iced::daemon`** — background apps with no main window.
- **Multi-window improvements**: `window::open` returns `(window::Id, Task<Message>)`.
- **`text_editor`** gained richer actions and undo groups.
- **More themes** (KanagawaWave etc. — predated 0.14 in some releases; check your version).

Breaking changes are rare; most 0.13 code compiles unchanged. The release notes are short — skim them rather than reading blog posts.

## Feature Flag Renames

| 0.12 feature | 0.13+ feature |
|---|---|
| `tokio` | `tokio` (unchanged) |
| `async-std` | removed — use `smol` (`async-std` crates work on top) |
| `debug` | removed from default; no explicit feature needed |

If `Cargo.toml` references `iced/async-std`, drop it — you're on `tokio` or `smol` now.

## After Migration — Sanity Check

- [ ] App builds with `cargo check`
- [ ] All `.style(theme::X::Y)` references are now `.style(widget::style_fn)`
- [ ] No remaining `Command<Message>` types
- [ ] No `impl Application for` / `impl Sandbox for`
- [ ] `Cargo.toml` specifies exactly one of `tokio` / `smol`
- [ ] App runs and a theme change still works (canary for palette access regressions)

## When NOT to migrate

- The app is on 0.10 or 0.11: don't jump to 0.14 in one shot. Go to 0.12 first, then 0.13. Every hop has its own errata.
- You rely on a third-party widget crate (`iced_aw`, etc.): check whether it supports your target iced version *first*. These crates sometimes lag.
