# iced Tasks — Async the Right Way

`Task<Message>`는 iced의 Elm 스타일 "command"이다: 런타임이 수행하고 그 결과가 다른 `Message`로 반환되는 것. 규칙: **`update` 내부에서 절대 block하거나 `tokio::spawn`하지 마라**. 대신 Task를 발행하라 — iced는 런타임을 통해 메시지를 clone하고 순서를 sane하게 유지한다.

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

작업을 발행하지 않는 분기에서 **`Task::none()`을 반환**한다 — iced는 모든 arm이 `Task`를 반환하도록 요구한다, 비어있더라도.

## Parallel Tasks — `Task::batch`

여러 독립적인 task를 한 번에 발사한다. 그들의 완료는 future가 끝나는 순서대로 별도 메시지로 도착한다:

```rust
Message::Initialize => Task::batch([
    Task::perform(load_user(), Message::UserLoaded),
    Task::perform(load_settings(), Message::SettingsLoaded),
    Task::perform(check_updates(), Message::UpdateChecked),
])
```

`tokio::join!`을 직접 만들지 마라 — `Task::batch`는 런타임에 그들을 멀티플렉싱하는 데 필요한 것을 제공한다.

## Sequencing — `.then(...)`

step B가 step A의 결과에 의존할 때, chain한다:

```rust
Message::SaveAndExit => {
    Task::perform(save(state.clone()), Message::Saved)
        .then(|_| iced::exit())
}
```

`.then`은 future의 출력을 받고 새 `Task`를 반환한다. 이를 신중히 사용하라 — 결과가 state에 닿아야 한다면 Message round-trip을 선호한다.

## Stream Progress — `Task::sip`

장시간 실행 task가 실행 중에 진행을 보고해야 할 때 (다운로드, 인덱싱), `Task::sip`을 사용한다. stream을 생성하는 future, 진행 매퍼, 완료 매퍼를 받는다:

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

**`abort_on_drop`인 이유**: state가 `Downloading`에서 전환될 때 (예: 사용자 취소), `Handle`이 drop되고 in-flight task가 abort된다. 수동 cancel 플래그가 필요 없다.

## Cancellation — `abortable`

어떤 task든 abortable로 만들 수 있다:

```rust
let (task, handle) = Task::perform(long_work(), Message::Done).abortable();
app.current = Some(handle);                 // keep the handle alive
task
```

취소하려면:

```rust
Message::Cancel => {
    if let Some(handle) = app.current.take() {
        handle.abort();
    }
    Task::none()
}
```

`abort()` 없이 handle을 drop하는 것은 **취소가 아니다** — task는 백그라운드에서 끝나고 그 완료 Message는 여전히 전달된다. drop = cancel을 원한다면 handle에서 `.abort_on_drop()`을 호출한다.

## Sending Side Effects — `iced::exit`, `window::close`

```rust
// Close the whole app
Message::Quit => iced::exit()

// Close a specific window
Message::CloseSettings(id) => window::close(id)
```

둘 다 `Task`를 반환하므로 `.then`과 chain된다.

## Command-style side effects without async

future를 spawn하지 않고 "그냥 나중에 메시지를 생성하기"용:

```rust
Task::done(Message::Refresh)           // queue a message for next tick
```

await가 필요 없는 사후 완료 hook에 대해 `.then` 내부에서 유용하다.

## Async Runtime Choice

iced는 `tokio` 또는 `smol` 중 **정확히 하나**를 요구한다:

```toml
iced = { version = "0.14", features = ["tokio"] }     # or "smol"
```

- **tokio**: 앱이 `reqwest`, `sqlx`, `tokio::fs`나 tokio 생태계의 무엇이든 사용한다면 이를 선택.
- **smol**: 더 가벼움, 자기 완결적인 앱에 적합. `async-std` crate가 작동.

둘 다 활성화하면 시작 시 런타임 충돌이 트리거되며 런타임에서야 표면화된다. 하나를 선택하고, 커밋하고, 진행하라.

## What NOT to do

**`update` 내부에서 `tokio::spawn`하지 마라:**

```rust
// WRONG — breaks message plumbing
Message::Start => {
    tokio::spawn(async move { /* ... */ });
    Task::none()
}
```

spawn된 task는 실행되지만, 그 출력은 결코 `Message`가 될 수 없다. UI는 언제 끝나는지 모른다. 대신 `Task::perform`을 사용한다.

**`update`에서 block하지 마라:**

```rust
// WRONG — freezes the UI until this returns
Message::LoadFile => {
    app.content = std::fs::read_to_string(path).ok();
    Task::none()
}
```

작은 read도 block할 수 있다. `Task::perform`으로 wrap한다:

```rust
Message::LoadFile => Task::perform(
    async move { tokio::fs::read_to_string(path).await.map_err(|e| e.to_string()) },
    Message::FileLoaded,
)
```

## Error Handling Pattern

비동기 결과를 `Result`로 wrap하고 `update`가 패턴 매칭하도록 한다:

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

에러에 `String`을 사용하면 `Clone`이 사소해진다 (`Message: Clone`이므로 필수). 더 풍부한 에러는 자체 `Clone + Debug` 에러 타입을 정의하거나 `Arc<anyhow::Error>`로 wrap한다.
