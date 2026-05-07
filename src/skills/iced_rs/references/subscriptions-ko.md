# iced Subscriptions — Listening to the Outside World

`Subscription<Message>`은 "이것이 반환되는 동안 이 메시지들을 전달한다"는 선언적 표현이다. 타이머, 글로벌 이벤트, 채널 — 런타임이 시간 경과에 따라 멀티플렉싱하는 어떤 것이든. `Task`(fire-and-forget, one outcome)와 달리, Subscription은 반환을 멈출 때까지 메시지 생성을 계속한다.

## The shape

```rust
.subscription(App::subscription)

fn subscription(&self) -> Subscription<Message> {
    // return the subscriptions you want active *right now*
    Subscription::none()
}
```

state에 따라 다른 subscription을 반환한다 — 런타임이 집합을 diff하고 기저 producer를 시작/중지한다. 수동 시작/중지 없음.

## Timers — `time::every`

```rust
use iced::time::{self, Duration};

fn subscription(&self) -> Subscription<Message> {
    time::every(Duration::from_secs(1)).map(|_| Message::Tick)
}
```

tick은 `Message::Tick` variant로 도착한다. `time::every`는 `Instant`를 준다 — timestamp를 원하면 `.map(Message::Tick)`로 유지하고, 그렇지 않으면 `_ => Message::Tick`.

조건부로 반환하지 않음으로써 타이머를 중지한다:

```rust
if self.clock_running {
    time::every(Duration::from_secs(1)).map(|_| Message::Tick)
} else {
    Subscription::none()
}
```

런타임이 변경을 감지하고 프레임 사이에 타이머를 종료한다.

## Global Events — `event::listen`

모든 런타임 이벤트 (keyboard, mouse, window, touch):

```rust
use iced::event::{self, Event};

fn subscription(&self) -> Subscription<Message> {
    event::listen().map(Message::Event)
}
```

그런 다음 `update`에서 이벤트를 패턴 매칭한다:

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

**subscription에서 필터링**: 특정 이벤트만 신경 쓴다면, 필터 closure를 전달하는 `event::listen_with`를 사용한다. 같은 것이지만, 할당을 절약하고 더 좁은 메시지 타입을 emit할 수 있게 한다.

## Keyboard-specific

포커스된 keypress subscription (앱 윈도우가 포커스를 가졌을 때만 발화):

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

무시하려면 `None`을 반환한다 — `update`의 catch-all match보다 깔끔하다.

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

활성 subscription이 1개를 초과할 때마다 batch를 반환한다. 내부 순서는 중요하지 않다.

## Channel-based — `Subscription::run`

메시지의 *stream*을 생성하는 background task (file watcher, 장시간 실행 비동기 worker)는 `Subscription::run`을 사용한다:

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

subscription의 identity는 함수 포인터 / closure 캡처이다 — 다음 프레임에서 같은 `run`이 반환되면, 런타임이 기존 stream을 살려둔다. 재시작하려면 반환되는 것을 변경한다.

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

이 패턴 — 조건부로 빌드되어 batch되는 `Vec<Subscription<Message>>` — 은 `Subscription::none()`을 반환하는 중첩된 `if`/`else`보다 더 잘 확장된다.

## Window-level events

```rust
use iced::window;

window::resize_events().map(|(id, size)| Message::WindowResized(id, size))
window::close_requests().map(|id| Message::WindowClosing(id))
```

`window::close`로 실제로 윈도우를 닫기 전에 ("save changes?"를 prompt하기 위해) close를 가로채려면 `close_requests`를 사용한다.

## Common Patterns

### Debouncing

subscription은 끊임없이 발화한다; `update`에서 debounce한다:

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

- **subscription closure 내부에서 사이드 이펙트를 일으키지 마라.** closure는 매 프레임 실행된다; 어떤 I/O든 시스템을 두드린다. 사이드 이펙트는 `Task::perform`에 속한다.
- **subscription 순서에 의존하지 마라.** `Subscription::batch`는 병합한다; producer 간 순서는 보장되지 않는다. 순서가 필요하면 메시지에 시퀀스 번호를 보낸다.
