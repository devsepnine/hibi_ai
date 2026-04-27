---
name: iced
description: Use this skill for any task involving `iced`, the Rust GUI framework. Trigger whenever the user mentions iced by name in a Rust, GUI, or desktop-app context — no matter the sub-task: writing a new iced app, adding a widget/feature, debugging a compile error, migrating between iced versions (0.12 → 0.13/0.14, `Application`/`Sandbox` traits → `iced::application()` builder, `Command` → `Task`, styling API changes), explaining any iced API (`Message`/`update`/`view`/`Element`, `Task::perform`, `Subscription`, `canvas::Program`, `window::Settings`, `Theme`/`extended_palette`, widget styles), wiring async work, or choosing iced over Electron/Tauri for a desktop project. Assume iced-flavored intent whenever the user says "iced 앱", "iced::", "러스트 iced", "iced 마이그레이션", pastes iced code, or asks how to do X in iced. Do NOT use for other Rust GUIs (egui, Slint, Dioxus, gtk-rs, Tauri), ratatui (terminal — separate skill), web frameworks, or Rust work without a GUI.
keywords: [iced, rust-gui, elm, elm-architecture, widget, application, task, subscription, canvas, theme, palette, migration, 마이그레이션, 러스트gui, 데스크톱ui]
---

# iced (Rust GUI) — Production Guide

Retained-mode, Elm-architecture GUI library for Rust. One `Message` enum, one `update`, one `view`, and the runtime drives the loop. This skill encodes the 0.13/0.14 idioms — the ones that replaced the old `Application`/`Sandbox` traits — plus async, theming, and canvas patterns proven in real apps.

## When to use which reference

This SKILL.md is the index. Open the focused reference for the sub-task:

| Task | Reference |
|---|---|
| App entry point: `iced::run` vs `iced::application()` builder, `Settings`, `window::Settings` | `references/application.md` |
| Pick a widget (text/button/column/row/container/text_input/scrollable/pick_list/checkbox/slider/toggler/tooltip) | `references/widgets.md` |
| Async work: `Task::perform`, `Task::sip`, `Task::batch`, chaining, cancellation, `iced::exit()` | `references/tasks.md` |
| Runtime events: `time::every`, `event::listen`, keyboard/mouse, batching subscriptions | `references/subscriptions.md` |
| Styling: built-in stylers (`button::primary`, `container::rounded_box`), custom style closures, `Theme::ALL`, `extended_palette()` | `references/theming.md` |
| Custom drawing: `canvas::Program`, `Cache`, `Path`, `Frame::fill/stroke/fill_text`, `Geometry` | `references/canvas.md` |
| Migrating from 0.12 (`Application`/`Sandbox` traits) to 0.13/0.14 builder API | `references/migration.md` |
| Avoiding common iced footguns (Element lifetimes, Message Clone, runtime choice, view churn) | `references/gotchas.md` |

## Project Setup

Use edition 2024 (Rust 1.85+) and the current stable `iced`. The builder API (`iced::application`) covers 99% of apps; reserve `iced_runtime` for advanced embeds.

```toml
[package]
name = "my-iced-app"
edition = "2024"
rust-version = "1.85"

[dependencies]
iced = { version = "0.14", features = ["tokio", "canvas", "advanced"] }
# For 0.13, just pin "0.13". API is compatible for everything in this skill.
```

Feature flags worth knowing:

| Feature | What it adds |
|---|---|
| `tokio` | Use tokio as the async runtime for `Task::perform` |
| `smol` | Alternative runtime; pick exactly one |
| `canvas` | `canvas::Program`, `Geometry`, `Path`, `Frame` |
| `advanced` | Low-level `widget::Widget` trait for custom widgets |
| `image` / `svg` | Bitmap / vector image widgets |
| `markdown` | Render markdown with `iced::widget::markdown` |

Pick **one** of `tokio`/`smol`. Enabling both wastes deps and can cause runtime conflicts when a library upstream picks the other.

## Minimal App — `iced::run` (no state struct needed)

The smallest legal iced app. State can be a bare `u64`; iced doesn't force a struct.

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

**Why this works**: `iced::run` accepts any `State: Default` plus a pure `update(&mut State, Msg)` and `view(&State) -> Element`. Good for demos and small tools. Upgrade to `iced::application()` the moment you need a theme, subscription, async work, or window config.

## Standard App — `iced::application()` builder

The idiomatic choice for almost everything. Each builder method is optional; chain only what you need.

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

**Key shape**: `new` returns `(Self, Task<Message>)` so the app can fire an initial async load (e.g., read a config) without a special "init" message. `update` also returns `Task<Message>` — the Elm-style "command" that the runtime executes and whose result comes back as another `Message`.

Details, including window settings (size, position, min/max, decorations) and multi-window apps, live in `references/application.md`.

## Layout Essentials

`column!` and `row!` are macros for heterogeneous children; their `.function_form` variants accept `Vec<Element>` when children are built dynamically.

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

**Length primitives**: `Fill` (take all remaining), `Shrink` (size to content), `FillPortion(n)` (proportional share), or a bare `f32` (fixed pixels). Prefer `Fill`/`FillPortion` over hardcoded widths so the layout survives window resizes.

More patterns — `scrollable`, `pick_list`, `tooltip`, `text_input`, `canvas` — in `references/widgets.md`.

## Message Discipline

The message enum is the spine of the app. Three rules keep it maintainable:

1. **`#[derive(Debug, Clone)]` on every Message.** The runtime clones messages across the event boundary. `Copy` where the payload allows — iced happily uses it.
2. **One Message variant per user-observable event**, not per internal transition. `ThemeChanged(Theme)` > `InternalMutationA/B/C`.
3. **Nest sub-modules' messages inside a variant** instead of flattening. `Message::Editor(editor::Message)` scales better than `Message::EditorKeyPressed / Message::EditorSaved / ...` at 30 variants.

```rust
#[derive(Debug, Clone)]
enum Message {
    Editor(editor::Message),
    FileOpened(Result<String, String>),
    ThemeChanged(Theme),
}
```

## Async Work — `Task::perform`

`update` returns `Task<Message>`. Issue async work via `Task::perform(future, on_complete)`; the result comes back as a message.

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

For streaming progress, cancellation, batch, and sequencing see `references/tasks.md`.

## Styling at a Glance

Built-in stylers cover most cases; custom is a closure over `(theme, status) -> Style`.

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

Palette roles (`primary`/`secondary`/`success`/`danger`/`background`) × strength (`base`/`strong`/`weak`) × what (`color`/`text`) — all of it via `theme.extended_palette()`. Full reference: `references/theming.md`.

## Performance Quick Rules

1. **`view` must stay cheap** — it runs every frame that state changes. Don't allocate or format inside hot loops; precompute in `update` and cache on `App`.
2. **Borrow in `view`, own in `update`.** `Element<'a, Message>` borrows from `&self`, so move expensive owned data into the App struct and expose references to the builder.
3. **Use `canvas::Cache`** for custom drawings — `Cache::draw` skips re-tessellation until you call `.clear()`.
4. **Don't over-nest layouts.** Each `container`/`column`/`row` is a node; 5 levels deep is fine, 15 is a smell.
5. **Prefer `Task::batch` over spawning**; the runtime already parallelizes. Don't `tokio::spawn` inside `update` — you lose the message plumbing.

## Related Skills

- General Rust idioms: `rust-best-practices`
- If the UI is a terminal instead: `ratatui` (this project)
- Testing/error patterns shared with iced state logic: `rust-best-practices` / `tdd-workflow`
