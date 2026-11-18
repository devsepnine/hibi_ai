# iced Tasks: Async the Right Way

Return a `Task<Message>` from `update` to start asynchronous work and route its output back to the application. Keep blocking operations off the UI thread.

## Basic `Task::perform`

```rust
use iced::Task;

fn update(app: &mut App, msg: Message) -> Task<Message> {
    match msg {
        Message::Load if app.loading => Task::none(),
        Message::Load => {
            app.error = None;
            app.loading = true;
            Task::perform(fetch_data(), Message::Loaded)
        }
        Message::Loaded(Ok(data)) => {
            app.loading = false;
            app.data = Some(data);
            Task::none()
        }
        Message::Loaded(Err(e)) => {
            app.loading = false;
            app.error = Some(e);
            Task::none()
        }
    }
}

async fn fetch_data() -> Result<Data, String> { /* ... */ }
```

This `update` returns a `Task<Message>`, so each branch must return a task. Use `Task::none()` for branches with no work. An update that only changes state can return `()`.

## Parallel Tasks: `Task::batch`

Fire multiple independent tasks at once. Their completions arrive as separate messages, in whatever order the futures finish:

```rust
Message::Initialize => Task::batch([
    Task::perform(load_user(), Message::UserLoaded),
    Task::perform(load_settings(), Message::SettingsLoaded),
    Task::perform(check_updates(), Message::UpdateChecked),
])
```

Don't hand-craft `tokio::join!`. `Task::batch` hands the runtime what it needs to multiplex them.

## Exit after a successful save

Return the save result to `update`. Exit on success and keep the draft on screen with an error on failure. The example ignores edits and duplicate save requests until the save finishes. Disable both controls in the view while `app.saving` is true. If editing stays enabled during the save, compare the saved revision with the current revision before exiting.

```rust
Message::SaveAndExit | Message::Edited(_) if app.saving => Task::none(),
Message::Edited(draft) => {
    app.draft = draft;
    Task::none()
}
Message::SaveAndExit => {
    app.saving = true;
    app.error = None;
    Task::perform(save(app.draft.clone()), Message::Saved)
}
Message::Saved(Ok(())) => iced::exit(),
Message::Saved(Err(error)) => {
    app.saving = false;
    app.error = Some(error);
    Task::none()
}
```

## Stream Progress: `Task::sip`

Enable the `sipper` feature of iced 0.14 for `Task::sip`. The download in this example returns a sipper with progress values and a final result. Display the error stored in `Failed` and allow a retry from that state.

```rust
use iced::{Task, task};

#[derive(Debug, Clone)]
enum Message {
    StartDownload,
    Progress(f32),
    Finished(Result<(), String>),
}

enum State {
    Idle,
    Downloading { _handle: task::Handle },
    Done,
    Failed(String),
}

fn update(app: &mut App, msg: Message) -> Task<Message> {
    match msg {
        Message::StartDownload if matches!(app.state, State::Downloading { .. }) => Task::none(),
        Message::StartDownload => {
            app.progress = 0.0;
            let (task, handle) = Task::sip(
                download_stream("https://example.com/big.zip"),
                Message::Progress,
                Message::Finished,
            )
            .abortable();

            app.state = State::Downloading {
                _handle: handle.abort_on_drop(),
            };
            task
        }
        Message::Progress(p) => { app.progress = p; Task::none() }
        Message::Finished(Ok(())) => { app.state = State::Done; Task::none() }
        Message::Finished(Err(e)) => { app.state = State::Failed(e); Task::none() }
    }
}
```

**Why `abort_on_drop`**: when state transitions away from `Downloading`, for example when the user cancels, the `Handle` is dropped and the in-flight task is aborted. No manual cancel flag needed.

## Cancellation: `abortable`

Any task can be made abortable:

```rust
let (task, handle) = Task::perform(long_work(), Message::Done).abortable();
app.current = Some(handle);                 // keep the handle alive
task
```

To cancel:

```rust
Message::Cancel => {
    if let Some(handle) = app.current.take() {
        handle.abort();
    }
    Task::none()
}
```

Dropping the handle without `abort()` does **not** cancel. The task finishes in the background and its completion Message is still delivered. If you want drop = cancel, call `.abort_on_drop()` on the handle.

## Sending Side Effects: `iced::exit`, `window::close`

```rust
// Close the whole app
Message::Quit => iced::exit()

// Close a specific window
Message::CloseSettings(id) => window::close(id)
```

Return the close or exit task directly. These effects emit no message for a `Task::then` callback. When a surviving window needs to react to another window closing, subscribe to `window::close_events()`.

## Command-style side effects without async

For "just produce a message later" without spawning a future:

```rust
Task::done(Message::Refresh)           // queue a message for next tick
```

Useful inside `.then` for post-completion hooks that don't need await.

## Async Runtime Choice

Native iced 0.14 needs an executor. Its default features include `thread-pool`. Enable `tokio` for work that requires a Tokio runtime, such as native asynchronous reqwest calls.

```toml
iced = { version = "0.14", features = ["tokio"] }
```

If Cargo unifies multiple features, the default native executor uses `tokio`, then `smol`, then `thread-pool` in that order. WebAssembly uses the browser executor. Inspect `cargo tree -e features -i iced_futures` to see the features enabled by dependencies.

## What NOT to do

**Don't `tokio::spawn` inside `update`:**

```rust
// WRONG — breaks message plumbing
Message::Start => {
    tokio::spawn(async move { /* ... */ });
    Task::none()
}
```

The spawned task runs, but its output can never become a `Message`. Your UI won't know when it finishes. Use `Task::perform` instead.

**Don't block in `update`:**

```rust
// WRONG — freezes the UI until this returns
Message::LoadFile => {
    app.content = std::fs::read_to_string(path).ok();
    Task::none()
}
```

Even small reads can block. Wrap in `Task::perform`:

```rust
Message::LoadFile => Task::perform(
    async move { tokio::fs::read_to_string(path).await.map_err(|e| e.to_string()) },
    Message::FileLoaded,
)
```

## Error Handling Pattern

Wrap the async result in `Result` and let `update` pattern-match:

```rust
async fn fetch(url: String) -> Result<String, String> {
    reqwest::get(&url).await
        .map_err(|e| e.to_string())?
        .error_for_status().map_err(|e| e.to_string())?
        .text().await
        .map_err(|e| e.to_string())
}

#[derive(Debug, Clone)]
enum Message {
    Fetch,
    Fetched(Result<String, String>),
}
```

The status check sends HTTP 4xx and 5xx responses to the error branch. `Task::perform` can carry a nonclonable error directly. When a widget or time travel requires a clonable message, choose a display string, a clonable error type, or `Arc<E>` according to the details callers need.
