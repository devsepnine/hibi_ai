# iced Gotchas — Things That Will Bite You

A catalog of the subtle issues that don't crash at compile time but produce confusing runtime behavior or friction in the code. Each entry: what, why, fix.

## 1. `Message` Must Be `Clone` — And Everything Inside It Too

**What**: `#[derive(Debug, Clone)]` on `Message` fails because a variant carries a non-`Clone` type like `std::io::Error` or `reqwest::Error`.

**Why**: The iced runtime clones messages across the event queue. Without `Clone`, the whole app doesn't compile.

**Fix**: Convert errors to `String` at the async boundary, or wrap in `Arc<T>`:

```rust
async fn fetch() -> Result<String, String> {
    reqwest::get("...").await
        .map_err(|e| e.to_string())?       // error → String early
        .text().await
        .map_err(|e| e.to_string())
}

#[derive(Debug, Clone)]
enum Message {
    Fetched(Result<String, String>),       // both arms are Clone
}
```

For richer errors:

```rust
#[derive(Debug, Clone)]
enum Message {
    Fetched(Result<Data, Arc<anyhow::Error>>),  // Arc<E> is Clone even if E isn't
}
```

## 2. `Element<'a, Message>` Lifetime Confusion

**What**: A `view` function that tries to build an `Element` from owned strings or other owned data fails borrow-check.

**Why**: `Element<'a, Message>` borrows from `&self`. Widgets like `text(&self.name)` take a reference into the App state and the `Element`'s lifetime is tied to that borrow.

**Fix**: If the view needs an owned string, either store it owned on `self` (`self.display_name: String`) or build it as owned inside the widget call:

```rust
// Borrow — preferred when the data already lives on self
text(&self.name)

// Owned — when the string is computed each render
text(format!("Count: {}", self.counter))
```

Don't do this:

```rust
fn view(&self) -> Element<'_, Message> {
    let formatted = format!("Count: {}", self.counter);
    text(&formatted).into()                // BORROW ERROR — formatted dies at return
}
```

Do this:

```rust
fn view(&self) -> Element<'_, Message> {
    text(format!("Count: {}", self.counter)).into()    // widget owns the String
}
```

## 3. `iced::run` Doesn't Work With Non-`Default` State

**What**: You switch state from `(u64,)` to a struct holding a `Connection` and suddenly `iced::run` won't compile.

**Why**: `iced::run` requires `State: Default` so it can construct the initial value. A struct with non-default fields can't impl `Default`.

**Fix**: Move to `iced::application()`, where you provide the `new` function:

```rust
iced::application(App::new, App::update, App::view).run()

impl App {
    fn new() -> (Self, Task<Message>) {
        (Self { conn: Connection::open().unwrap(), ... }, Task::none())
    }
}
```

## 4. `tokio::spawn` Inside `update` Vanishes Into the Void

**What**: You `tokio::spawn` an async task inside `update` hoping it'll update the UI when done. It doesn't. The UI just sits there.

**Why**: The spawned task runs, but its output never becomes a `Message`. The runtime has no idea it exists.

**Fix**: Always use `Task::perform` (or `Task::sip` for streaming):

```rust
// Wrong
Message::Load => {
    tokio::spawn(async { fetch().await });
    Task::none()
}

// Right
Message::Load => {
    Task::perform(fetch(), Message::Loaded)
}
```

See `references/tasks.md`.

## 5. Two Async Runtimes at Once

**What**: The app panics at startup with an error about multiple runtime initializations.

**Why**: `Cargo.toml` enabled both `tokio` and `smol` features on `iced`, or one of your transitive deps pulled in the other.

**Fix**: Pick one. Add it explicitly:

```toml
iced = { version = "0.14", features = ["tokio"], default-features = false }
```

Then `cargo tree | grep -E "tokio|async-std|smol"` to find stragglers.

## 6. `view` Re-runs Every Frame — Don't Do Work In It

**What**: UI feels laggy, CPU usage is high, framerate is low with only trivial widgets on screen.

**Why**: `view` is called every frame a redraw happens (usually on each message). If `view` formats strings from scratch, sorts a `Vec`, or reads a file, it multiplies that cost by framerate.

**Fix**: Precompute in `update`, cache on `self`:

```rust
// Wrong — sort runs every frame
fn view(&self) -> Element<'_, Message> {
    let mut items = self.items.clone();
    items.sort_by_key(|i| i.priority);
    column(items.iter().map(|i| text(&i.name).into()).collect()).into()
}

// Right — sort only when data changes
impl App {
    fn update(&mut self, msg: Message) -> Task<Message> {
        if let Message::DataLoaded(items) = msg {
            let mut items = items;
            items.sort_by_key(|i| i.priority);
            self.items = items;
        }
        Task::none()
    }
}
```

## 7. `Container` Doesn't Fill Unless Told To

**What**: You wrap something in `container` expecting it to center in the window. It sits in the top-left corner.

**Why**: `container` defaults to `Shrink` on both axes — it sizes to its child. Centering only works if the container has space to center *within*.

**Fix**: Give it a Fill-axis explicitly:

```rust
container(content)
    .center_x(Fill)           // width = Fill, content horizontally centered
    .center_y(Fill)           // height = Fill, content vertically centered
```

Or compose `.width(Fill).height(Fill).align_x(Center).align_y(Center)` — same result, more verbose.

## 8. Forgetting `.on_press` Renders A Disabled Button

**What**: A button looks grayed out and doesn't click.

**Why**: A `button` without `.on_press(message)` is considered disabled and styled accordingly.

**Fix**: Always call `.on_press(...)`. For conditional enablement:

```rust
button("Submit").on_press_maybe(
    if self.form_valid() { Some(Message::Submit) } else { None }
)
```

## 9. `print!` / `eprintln!` Doesn't Corrupt iced (but silently logs)

**What**: Logs from `println!` don't show up anywhere visible.

**Why** (and the *not* a problem): unlike ratatui, iced owns the window, not the terminal — logging to stdout/stderr from an iced app is harmless but usually invisible because the app isn't launched from a terminal on a typical user install.

**Fix**: Use `tracing` or `log` with a file appender for anything you need post-hoc. Don't try to read stderr from a running GUI.

## 10. Changing Theme in `update` Doesn't Repaint Until Next Message

**What**: Theme-switch message fires, but the UI doesn't visibly update until the next keystroke or mouse move.

**Why**: iced redraws on state change *triggered by a message*. The `theme(&self)` function is called on each frame, but frames only happen on events.

**Fix**: The fix is "emit a follow-up message to force a redraw", which iced handles automatically — if you're seeing a stuck frame, you probably have state that isn't tied to a `Message` mutation. Make sure the theme switch *is* going through `update`. If it does, it'll repaint.

For apps that need continuous animation even without user input, add a `time::every` subscription — it delivers frames at a fixed rate and guarantees repaints.

## 11. Subscription Restarts When the Closure "Shape" Changes

**What**: You return a subscription conditionally, and every time the condition flips, the subscription's work restarts (file watcher re-reads, channel re-opens).

**Why**: iced diffs subscriptions by their *identity* — the closure's captures and type. A new closure instance is a new subscription.

**Fix**: Keep the subscription's producer function stable across frames. Move the closure to a named `fn` if you find yourself conditionally capturing:

```rust
fn my_watcher_stream() -> Subscription<Message> {
    Subscription::run(|| { /* ... */ })
}

fn subscription(&self) -> Subscription<Message> {
    if self.watching {
        my_watcher_stream()                  // stable identity
    } else {
        Subscription::none()
    }
}
```

## 12. Multi-Window: Windows Close Unexpectedly

**What**: The app uses multiple windows; closing the main window exits the whole process. You wanted a tray-style lingering app.

**Why**: `iced::application()` exits when its primary window closes. For persistent background apps, use `iced::daemon()` — no primary window, lifecycle tied to explicit `iced::exit()`.

**Fix**: Migrate to `iced::daemon` if the app should outlive a single window. See upstream `multi_window` example.

## 13. Text Wrap Not Kicking In

**What**: Long text draws off the right edge instead of wrapping.

**Why**: `text` wraps within its container, but if the container itself is `Shrink` (default), there's no width to wrap *to*.

**Fix**: Set an explicit width somewhere up the chain:

```rust
// container has no width → text won't wrap
container(text(long_string))

// width set → text wraps
container(text(long_string)).width(Fill)
// or column![..text(long_string)..].width(Fill)
```

## 14. Custom Widgets Need the `advanced` Feature

**What**: Implementing `widget::Widget` trait and the compiler can't find it.

**Why**: The low-level widget trait is gated behind the `advanced` feature.

**Fix**:

```toml
iced = { version = "0.14", features = ["advanced", ...] }
```

Then `use iced::advanced::widget::Widget;`. Most apps never need this — compose built-in widgets first.
