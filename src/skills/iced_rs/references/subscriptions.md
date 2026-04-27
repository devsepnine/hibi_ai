# iced Subscriptions — Listening to the Outside World

A `Subscription<Message>` is a declarative "while this is returned, deliver these messages." Timers, global events, channels — anything the runtime multiplexes over time. Unlike a `Task` (fire-and-forget, one outcome), a Subscription keeps producing messages until you stop returning it.

## The shape

```rust
.subscription(App::subscription)

fn subscription(&self) -> Subscription<Message> {
    // return the subscriptions you want active *right now*
    Subscription::none()
}
```

Return different subscriptions depending on state — the runtime diffs the set and starts/stops the underlying producers for you. No manual start/stop.

## Timers — `time::every`

```rust
use iced::time::{self, Duration};

fn subscription(&self) -> Subscription<Message> {
    time::every(Duration::from_secs(1)).map(|_| Message::Tick)
}
```

The ticks arrive as the `Message::Tick` variant. `time::every` gives you an `Instant` — keep it with `.map(Message::Tick)` if you want the timestamp, or `_ => Message::Tick` if you don't.

Stop the timer by conditionally not returning it:

```rust
if self.clock_running {
    time::every(Duration::from_secs(1)).map(|_| Message::Tick)
} else {
    Subscription::none()
}
```

The runtime detects the change and shuts the timer down between frames.

## Global Events — `event::listen`

All runtime events (keyboard, mouse, window, touch):

```rust
use iced::event::{self, Event};

fn subscription(&self) -> Subscription<Message> {
    event::listen().map(Message::Event)
}
```

Then in `update`, pattern-match the event:

```rust
Message::Event(Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. })) => {
    match key.as_ref() {
        keyboard::Key::Named(keyboard::key::Named::Escape) => /* dismiss */,
        keyboard::Key::Character("s") if modifiers.control() => /* save */,
        _ => {}
    }
    Task::none()
}
```

**Filtering at the subscription**: if you only care about specific events, use `event::listen_with` which passes a filter closure. It's the same thing, just saves an allocation and lets you emit a narrower message type.

## Keyboard-specific

Focused keypress subscriptions (only fire when your app's window has focus):

```rust
use iced::keyboard;

keyboard::on_key_press(|key, modifiers| {
    Some(match key.as_ref() {
        keyboard::Key::Named(keyboard::key::Named::ArrowUp) => Message::Up,
        keyboard::Key::Named(keyboard::key::Named::ArrowDown) => Message::Down,
        _ => return None,                // `None` drops the event
    })
})
```

Return `None` to ignore — cleaner than a catch-all match in `update`.

## Mixing Multiple Subscriptions — `Subscription::batch`

```rust
fn subscription(&self) -> Subscription<Message> {
    Subscription::batch([
        time::every(Duration::from_millis(500)).map(|_| Message::Tick),
        event::listen().map(Message::Event),
        keyboard::on_key_press(key_handler),
    ])
}
```

Return a batch whenever you have >1 subscription active. Order inside doesn't matter.

## Channel-based — `Subscription::run`

For background tasks that produce a *stream* of messages (file watcher, long-running async worker), use `Subscription::run`:

```rust
use iced::{Subscription, stream};
use futures::stream::StreamExt;

fn subscription(&self) -> Subscription<Message> {
    Subscription::run(|| stream::channel(100, |mut output| async move {
        // long-running work; push into `output` as events happen
        loop {
            let event = watcher.next().await;
            let _ = output.send(Message::FileChanged(event)).await;
        }
    }))
}
```

The subscription's identity is the function pointer / closure captures — if the same `run` is returned on the next frame, the runtime keeps the existing stream alive. Change what's returned to restart.

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

This pattern — a `Vec<Subscription<Message>>` built conditionally and batched — scales better than nested `if`/`else` returning `Subscription::none()`.

## Window-level events

```rust
use iced::window;

window::resize_events().map(|(id, size)| Message::WindowResized(id, size))
window::close_requests().map(|id| Message::WindowClosing(id))
```

Use `close_requests` to intercept close (to prompt "save changes?") before actually closing the window with `window::close`.

## Common Patterns

### Debouncing

A subscription fires constantly; debounce in `update`:

```rust
Message::Event(Event::Keyboard(ev)) => {
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

- **Don't do side effects inside a subscription closure.** The closure runs every frame; any I/O will hammer the system. Side effects belong in `Task::perform`.
- **Don't rely on subscription order.** `Subscription::batch` merges; ordering across producers isn't guaranteed. If you need ordering, send a seq number in the message.
