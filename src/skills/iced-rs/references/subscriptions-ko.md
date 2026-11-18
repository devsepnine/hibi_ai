# iced Subscriptions: Listening to the Outside World

`Subscription<Message>`는 앱의 현재 상태에 필요한 스트림을 선언한다. 스트림은 입력이 끝나거나 앱이 구독을 제거할 때까지 메시지를 전달한다.

## The shape

```rust
.subscription(App::subscription)

fn subscription(&self) -> Subscription<Message> {
    // return the subscriptions you want active *right now*
    Subscription::none()
}
```

state에 따라 다른 subscription을 반환한다. 런타임이 집합을 diff하고 기저 producer를 시작/중지한다. 수동 시작/중지 없음.

## Timers: `time::every`

```rust
use iced::time::{self, Duration};

fn subscription(&self) -> Subscription<Message> {
    time::every(Duration::from_secs(1)).map(|_| Message::Tick)
}
```

네이티브 iced 0.14에서 `time::every`를 쓰려면 `tokio` 또는 `smol` 기능을 켠다. 기본 네이티브 스레드 풀에는 타이머 API가 없다. 브라우저 백엔드는 자체 타이머를 제공한다. 각 틱은 `Instant`를 전달한다. 시각을 저장하는 메시지는 `.map(Message::Tick)`로 연결한다.

조건부로 반환하지 않음으로써 타이머를 중지한다:

```rust
if self.clock_running {
    time::every(Duration::from_secs(1)).map(|_| Message::Tick)
} else {
    Subscription::none()
}
```

다음에 런타임이 활성 구독을 갱신할 때, 반환 목록에서 빠진 타이머가 취소된다.

## 키보드와 창 이벤트

`event::listen()`은 위젯이 처리하지 않고 `Ignored`로 남긴 이벤트를 전달한다. `event::listen_with`는 이벤트, 처리 상태, 창 ID를 받아 앱에 보낼 메시지를 선택한다. 두 함수 모두 창의 다시 그리기 요청을 제외한다. 아래 방향키 단축키는 포커스를 가진 위젯이 키를 처리했으면 반응하지 않는다.

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

독립적인 이벤트 소스를 묶는다. 키 한 번에 앱 동작이 한 번 발생하도록 단축키별 처리 경로를 하나만 둔다.

```rust
fn subscription(&self) -> Subscription<Message> {
    Subscription::batch([
        time::every(Duration::from_millis(500)).map(|_| Message::Tick),
        shortcuts(),
    ])
}
```

## 채널 작업

`Subscription::run`은 인수가 없는 함수 포인터를 받는다. 경로를 사용하는 작업에는 `run_with`를 쓰면 빌더와 전달한 데이터가 구독의 식별자가 된다. 아래 예시는 Tokio로 파일을 한 번 읽는다. iced의 `tokio` 기능과 앱이 직접 의존하는 Tokio의 `fs`, `io-util` 기능을 켠다.

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

스트림은 파일 끝에서 종료하고, 읽기 오류를 전달하며, 수신기가 닫히면 전송을 멈춘다. 같은 식별자를 다시 반환하면 기존 작업이 유지된다. 파일 끝에 도달한 후에도 같은 식별자를 계속 선언하면 파일을 다시 열지 않는다. 다시 시작하려면 식별자를 바꾸거나 구독을 한 번 제거한 다음 다시 추가한다. 경로가 고정돼 있으면 외부 값을 캡처하지 않는 `Subscription::run(|| read_lines(PathBuf::from("app.log")))`를 사용할 수 있다.

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

조건부로 빌드되어 batch되는 `Vec<Subscription<Message>>` 패턴은 `Subscription::none()`을 반환하는 중첩된 `if`/`else`보다 더 잘 확장된다.

## 창 이벤트

닫기 전에 확인하려면 앱 빌더에 `.exit_on_close_request(false)`를 설정한다. 별도로 여는 창은 열 때 `window::Settings::exit_on_close_request`를 `false`로 설정한다. 닫기 요청을 구독해 확인창을 띄우고, 사용자가 확인했을 때만 `window::close(id)`를 반환한다. 취소하면 창을 계속 열어 둔다.

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

텍스트 검색은 입력 위젯이 보내는 변경 메시지를 `update`에서 디바운스한다. 아래 타이머에는 Tokio를 사용한다.

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

- 구독 선언은 가볍게 유지한다. 파일 열기와 이벤트 수신은 스트림 내부에서 수행해 작업 수명이 구독을 따르게 한다. `update`가 요청하는 일회성 작업은 `Task::perform`을 사용할 수 있다.
- **subscription 순서에 의존하지 마라.** `Subscription::batch`는 병합한다; producer 간 순서는 보장되지 않는다. 순서가 필요하면 메시지에 시퀀스 번호를 보낸다.
