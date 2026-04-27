# iced Tasks — Async the Right Way

A `Task<Message>` is iced's Elm-style "command": something the runtime performs and whose outcome returns as another `Message`. The rule: **never block or `tokio::spawn` inside `update`**. Issue a Task instead — iced clones messages through the runtime and keeps ordering sane.

## Basic `Task::perform`

```rust
use iced::Task;

fn update(app: &mut App, msg: Message) -> Task<Message> {
    match msg {
        Message::Load => {
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

**Return `Task::none()`** from branches that don't issue work — iced requires every arm to return a `Task`, even if empty.

## Parallel Tasks — `Task::batch`

Fire multiple independent tasks at once. Their completions arrive as separate messages, in whatever order the futures finish:

```rust
Message::Initialize => Task::batch([
    Task::perform(load_user(), Message::UserLoaded),
    Task::perform(load_settings(), Message::SettingsLoaded),
    Task::perform(check_updates(), Message::UpdateChecked),
])
```

Don't hand-craft `tokio::join!` — `Task::batch` hands the runtime what it needs to multiplex them.

## Sequencing — `.then(...)`

When step B depends on step A's result, chain:

```rust
Message::SaveAndExit => {
    Task::perform(save(state.clone()), Message::Saved)
        .then(|_| iced::exit())
}
```

`.then` receives the future's output and returns a new `Task`. Use this sparingly — prefer a Message round-trip if the result needs to touch state.

## Stream Progress — `Task::sip`

When a long-running task should report progress during its run (downloads, indexing), use `Task::sip`. It takes a stream-producing future, a progress mapper, and a completion mapper:

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
}

fn update(app: &mut App, msg: Message) -> Task<Message> {
    match msg {
        Message::StartDownload => {
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
        Message::Finished(Err(e)) => { app.error = Some(e); Task::none() }
    }
}
```

**Why `abort_on_drop`**: when state transitions away from `Downloading` (e.g., user cancels), the `Handle` is dropped and the in-flight task is aborted. No manual cancel flag needed.

## Cancellation — `abortable`

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

Dropping the handle without `abort()` does **not** cancel — the task finishes in the background and its completion Message is still delivered. If you want drop = cancel, call `.abort_on_drop()` on the handle.

## Sending Side Effects — `iced::exit`, `window::close`

```rust
// Close the whole app
Message::Quit => iced::exit()

// Close a specific window
Message::CloseSettings(id) => window::close(id)
```

Both return a `Task` so they chain with `.then`.

## Command-style side effects without async

For "just produce a message later" without spawning a future:

```rust
Task::done(Message::Refresh)           // queue a message for next tick
```

Useful inside `.then` for post-completion hooks that don't need await.

## Async Runtime Choice

iced requires **exactly one** of `tokio` or `smol`:

```toml
iced = { version = "0.14", features = ["tokio"] }     # or "smol"
```

- **tokio**: pick this if the app uses `reqwest`, `sqlx`, `tokio::fs`, or anything in the tokio ecosystem.
- **smol**: lighter, fine for self-contained apps. `async-std` crates work.

Enabling both triggers a runtime conflict at startup that doesn't surface until runtime. Pick one, commit, move on.

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
        .text().await
        .map_err(|e| e.to_string())
}

#[derive(Debug, Clone)]
enum Message {
    Fetch,
    Fetched(Result<String, String>),   // Result is Clone, carries the error
}
```

Using `String` for the error makes `Clone` trivial (required since `Message: Clone`). For richer errors, define your own `Clone + Debug` error type or wrap in `Arc<anyhow::Error>`.
