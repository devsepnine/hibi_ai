# iced Subscriptions: Listening to the Outside World

A `Subscription<Message>` declares which streams the application needs in its current state. A stream produces messages until it ends or the application removes its subscription.

## The shape

```rust
.subscription(App::subscription)

fn subscription(&self) -> Subscription<Message> {
    // return the subscriptions you want active *right now*
    Subscription::none()
}
```

Return different subscriptions depending on state. The runtime diffs the set and starts/stops the underlying producers for you. No manual start/stop.

## Timers: `time::every`

```rust
use iced::time::{self, Duration};

fn subscription(&self) -> Subscription<Message> {
    time::every(Duration::from_secs(1)).map(|_| Message::Tick)
}
```

In native iced 0.14, enable `tokio` or `smol` for `time::every`. The default native thread pool has no timer API. The browser backend provides its own timer. Each tick carries an `Instant`. Use `.map(Message::Tick)` when the message stores that timestamp.

Stop the timer by conditionally not returning it:

```rust
if self.clock_running {
    time::every(Duration::from_secs(1)).map(|_| Message::Tick)
} else {
    Subscription::none()
}
```

Removing the subscription cancels its producer when the runtime next reconciles the active set.

## Keyboard and window interactions

`event::listen()` delivers interactions that widgets left `Ignored`. Use `event::listen_with` to receive the event, capture status, and window ID and choose which events reach the app. Both exclude window redraw requests. The arrow shortcut below respects focused widgets that consume arrow keys.

```rust
use iced::{Event, Subscription, event, keyboard, window};

fn arrow_shortcut(event: Event, status: event::Status, _: window::Id) -> Option<Message> {
    let Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) = event else {
        return None;
    };
    if status == event::Status::Captured {
        return None;
    }

    match key {
        keyboard::Key::Named(keyboard::key::Named::ArrowUp) => Some(Message::Up),
        keyboard::Key::Named(keyboard::key::Named::ArrowDown) => Some(Message::Down),
        _ => None,
    }
}

fn shortcuts() -> Subscription<Message> {
    event::listen_with(arrow_shortcut)
}
```

Batch independent producers. Give each shortcut one handler so a key press produces one application action.

```rust
fn subscription(&self) -> Subscription<Message> {
    Subscription::batch([
        time::every(Duration::from_millis(500)).map(|_| Message::Tick),
        shortcuts(),
    ])
}
```

## Channel workers

`Subscription::run` takes a function pointer with no arguments. For a worker that depends on a path, `run_with` uses the builder and the supplied data as its identity. This example reads a file once with Tokio. Enable `tokio` on iced and the `fs` and `io-util` features on the app’s Tokio dependency.

```rust
use iced::{Subscription, stream};
use iced::futures::{SinkExt, Stream, StreamExt};
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, BufReader};

fn read_lines(path: PathBuf) -> impl Stream<Item = Message> {
    stream::try_channel(16, async move |mut output| {
        let file = tokio::fs::File::open(path).await?;
        let mut lines = BufReader::new(file).lines();
        while let Some(line) = lines.next_line().await? {
            if output.send(line).await.is_err() {
                break;
            }
        }
        Ok::<(), std::io::Error>(())
    })
    .map(|result| Message::LogLine(result.map_err(|error| error.to_string())))
}

fn log_subscription(path: PathBuf) -> Subscription<Message> {
    Subscription::run_with(path, |path| read_lines(path.clone()))
}
```

The stream ends at EOF, reports a read error, and stops sending when the receiver closes. Returning the same identity keeps the existing worker. Even after EOF, declaring that identity again does not reopen the file. To restart, change the identity or remove the subscription for an update before adding it again. For a fixed source, `Subscription::run(|| read_lines(PathBuf::from("app.log")))` uses a noncapturing builder.

## Conditional / dynamic

```rust
fn subscription(&self) -> Subscription<Message> {
    let mut subs = vec![event::listen().map(Message::Event)];

    if let Some(download_id) = self.active_download {
        subs.push(download_progress(download_id));
    }

    if self.clock_running {
        subs.push(time::every(Duration::from_secs(1)).map(|_| Message::Tick));
    }

    Subscription::batch(subs)
}
```

This pattern, a `Vec<Subscription<Message>>` built conditionally and batched, scales better than nested `if`/`else` returning `Subscription::none()`.

## Window events

To confirm before closing, configure the application with `.exit_on_close_request(false)`. For a separately opened window, set `window::Settings::exit_on_close_request` to `false` when opening it. Subscribe to close requests, show the prompt, and issue `window::close(id)` only after confirmation. On cancellation, keep the window open.

```rust
fn window_events() -> Subscription<Message> {
    Subscription::batch([
        window::resize_events().map(|(id, size)| Message::WindowResized(id, size)),
        window::close_requests().map(Message::WindowClosing),
    ])
}
```

## Common Patterns

### Debouncing

For text search, debounce the input widget’s change message in `update`. The timer below uses Tokio.

```rust
Message::QueryChanged(query) => {
    self.query = query;
    self.pending_search = Some(Instant::now());
    Task::perform(
        async {
            tokio::time::sleep(Duration::from_millis(300)).await;
        },
        |_| Message::SearchTimerElapsed,
    )
}
Message::SearchTimerElapsed => {
    // Only run the search if no newer input came in
    if let Some(t) = self.pending_search {
        if t.elapsed() >= Duration::from_millis(300) {
            self.pending_search = None;
            return self.run_search();
        }
    }
    Task::none()
}
```

### Tick + pause

```rust
fn subscription(&self) -> Subscription<Message> {
    if self.paused {
        Subscription::none()
    } else {
        time::every(Duration::from_millis(16)).map(|_| Message::Frame)  // ~60Hz
    }
}
```

## What NOT to do

- Keep the subscription declaration cheap. Open files and receive events inside the worker stream so I/O follows its lifetime. A one time operation requested by `update` can use `Task::perform`.
- **Don't rely on subscription order.** `Subscription::batch` merges; ordering across producers isn't guaranteed. If you need ordering, send a seq number in the message.
