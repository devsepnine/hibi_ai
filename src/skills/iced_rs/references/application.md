# iced Application Entry Points

iced offers two entry points: the tiny `iced::run` and the full builder `iced::application`. Use the builder the moment you need anything beyond `update`/`view`.

## `iced::run` — smallest possible app

```rust
use iced::widget::{button, column, text};

pub fn main() -> iced::Result {
    iced::run(update, view)
}

fn update(count: &mut u64, msg: Message) { /* ... */ }
fn view(count: &u64) -> iced::Element<'_, Message> { /* ... */ }
```

**When**: demos, one-file experiments, tests.
**When not**: you need a theme, window size, subscription, async init, or a title. That's all the builder.

State for `iced::run` needs `Default` (so the runtime can construct the initial value). If your state isn't `Default`, move to `iced::application()`.

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

Every builder method is optional; chain only what applies. Each builder method accepts either a closure or a function pointer that takes `&State` — both work, whichever reads cleaner.

### Required triple

```rust
fn new() -> (App, Task<Message>)              // or: fn new(flags: Flags) -> (App, Task<Msg>)
fn update(&mut self, Message) -> Task<Message>
fn view(&self) -> Element<'_, Message>
```

The `new` returns a `(State, Task)` so the app can kick off initial async work (load config, fetch data) without a bootstrapping `Message::Init`.

### Title

Either a static string or a function of state:

```rust
.title("My App")                       // static
.title(|app: &App| format!("My App — {}", app.filename))   // dynamic
```

Dynamic titles update automatically when state changes.

### Theme

```rust
.theme(|app: &App| app.theme.clone())
```

Built-ins: `Theme::Light`, `Theme::Dark`, `Theme::Dracula`, `Theme::Nord`, `Theme::SolarizedLight/Dark`, `Theme::GruvboxLight/Dark`, `Theme::CatppuccinLatte/Frappe/Macchiato/Mocha`, `Theme::TokyoNight/Storm/Light`, `Theme::KanagawaWave/Dragon/Lotus`, `Theme::Moonfly`, `Theme::Nightfly`, `Theme::Oxocarbon`, `Theme::Ferra`. `Theme::ALL` gives you all of them as a slice — useful for a `pick_list`.

See `references/theming.md` for custom palettes.

### Subscription

```rust
.subscription(|app: &App| {
    Subscription::batch([
        time::every(Duration::from_secs(1)).map(|_| Message::Tick),
        event::listen().map(Message::Event),
    ])
})
```

Subscriptions run for as long as they're returned. Stop a subscription by conditionally excluding it from the batch. See `references/subscriptions.md`.

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

Pass with `.settings(settings())`.

**Common knobs**:

| Setting | Use for |
|---|---|
| `size: Size::new(w, h)` | Initial window size (logical pixels) |
| `position: Position::Centered` | Or `Specific(Point)` for exact placement |
| `min_size` / `max_size` | Constrain resizes |
| `resizable` | `false` for fixed-size dialogs |
| `decorations` | `false` for custom title bars |
| `transparent` | Required for rounded corners / acrylic backgrounds |
| `icon` | `Some(window::icon::from_file_data(bytes, None)?)` |

**Don't set**: `flags` and `id` directly — use the builder's shape; the runtime manages both.

### Builder-level window shortcuts

Some window properties have first-class builder methods that are clearer than hand-rolling a `Settings`:

```rust
iced::application(App::new, App::update, App::view)
    .window_size(Size::new(1024.0, 720.0))
    .centered()
    .resizable(false)
    .run()
```

Use these when the app only needs one or two overrides. Drop down to full `Settings` when you need more than three.

## Headless / default-less state

If `State` can't implement `Default` (it holds a `Connection`, a `Runtime`, etc.), use `iced::application()` with an explicit `new`. For `iced::run`, wrap the state in `Default` by moving non-default fields into `Option<T>` and filling them in the first `update` from `new`.

## Multi-window

0.13+ supports multiple native windows via `iced::daemon` for server-style apps (no startup window) and `window::open` / `window::close` for imperative window management. The window handle travels in messages:

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

For serious multi-window apps, read the upstream `multi_window` example — it's the single best reference.

## When to pick which entry point

| Need | Use |
|---|---|
| Demo, test, one-file | `iced::run` |
| Theme, title, async init | `iced::application()` |
| Multi-window | `iced::application()` with `window::open` |
| No main window (tray app / background) | `iced::daemon()` |

The builder is rarely over-engineered; even a trivial app benefits from the named method for `view`, which makes the future "add a theme" change trivial.
