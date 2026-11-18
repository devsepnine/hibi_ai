# iced Gotchas: Things That Will Bite You

A catalog of the subtle issues that don't crash at compile time but produce confusing runtime behavior or friction in the code. Each entry: what, why, fix.

## 1. Message bounds depend on the widgets and features

A button converted to an element needs a clonable message. The `time-travel` feature also requires `Clone` when the app runs, while the `debug` feature requires `Debug`. These are Cargo features. A plain task can return `std::io::Error` directly.

If the message must implement `Clone`, convert the error into the fields the UI needs or use `Arc` to retain the original error.

```rust
async fn fetch(url: &str) -> Result<String, String> {
    reqwest::get(url).await
        .map_err(|e| e.to_string())?
        .error_for_status().map_err(|e| e.to_string())?
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

**Fix**: If the view needs an owned string, either store it owned on `self`, as in `self.display_name: String`, or build it as owned inside the widget call:

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

**Fix**: Always use `Task::perform`, or `Task::sip` for streaming:

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

## 5. Selecting the native executor

Native iced 0.14 selects Tokio when both `tokio` and `smol` are enabled. For a Tokio dependency, keep the default renderer and platform features and add Tokio.

```toml
iced = { version = "0.14", features = ["tokio"] }
```

If defaults are disabled, select a renderer, a native executor, and on Linux at least one display backend such as `x11` or `wayland`. Check the resolved features with `cargo tree -e features -i iced`.

## 6. Repeated work when rebuilding a view

If each rebuild clones and sorts the same list, sort when the input changes and render the stored order.

```rust
Message::DataLoaded(mut items) => {
    items.sort_by_key(|item| item.priority);
    self.items = items;
    Task::none()
}
```

## 7. Centering when a container shrinks to its child

**What**: You wrap something in `container` expecting it to center in the window. It sits in the top-left corner.

When a container shrinks to fit its child, there is no spare space on that axis for centering. Give the container room on the axis you want to center.

**Fix**: Give it a Fill-axis explicitly:

```rust
container(content)
    .center_x(Fill)           // width = Fill, content horizontally centered
    .center_y(Fill)           // height = Fill, content vertically centered
```

Or compose `.width(Fill).height(Fill).align_x(Center).align_y(Center)`. Same result, more verbose.

## 8. Forgetting `.on_press` Renders A Disabled Button

**What**: A button looks grayed out and doesn't click.

**Why**: A `button` without `.on_press(message)` is considered disabled and styled accordingly.

**Fix**: Always call `.on_press(...)`. For conditional enablement:

```rust
button("Submit").on_press_maybe(
    if self.form_valid() { Some(Message::Submit) } else { None }
)
```

## 9. `print!` / `eprintln!` Doesn't Corrupt iced, But Silently Logs

**What**: Logs from `println!` don't show up anywhere visible.

**Why**, and why it is *not* a problem: unlike ratatui, iced owns the window, not the terminal. Logging to stdout/stderr from an iced app is harmless but usually invisible because the app isn't launched from a terminal on a typical user install.

**Fix**: Use `tracing` or `log` with a file appender for anything you need post-hoc. Don't try to read stderr from a running GUI.

## 10. Applying a theme change

Update the stored theme in the message handler and have the builder's theme function read that value. The runtime rebuilds the view after processing messages.

## 11. Restarting a conditional subscription

`Subscription::run` identifies the stream by its builder function pointer. Returning the same subscription keeps the worker alive. Omitting it stops the worker, and returning it later starts a new one. Use `Subscription::run_with` to include data in the identity.

## 12. Keeping an app alive after all windows close

A regular application exits when its last window closes. For a tray app that waits without a window, use `iced::daemon` and return `iced::exit()` when the app should quit.

## 13. Text wrapping width

Where a long line wraps depends on the width available from its parent and the text’s wrapping mode. To cap the line width in a card, set the container’s maximum width.

```rust
container(text(long_string)).max_width(480)
```

## 14. Custom Widgets Need the `advanced` Feature

**What**: Implementing `widget::Widget` trait and the compiler can't find it.

**Why**: The low-level widget trait is gated behind the `advanced` feature.

**Fix**:

```toml
iced = { version = "0.14", features = ["advanced", ...] }
```

Then `use iced::advanced::widget::Widget;`. Most apps never need this. Compose built-in widgets first.
