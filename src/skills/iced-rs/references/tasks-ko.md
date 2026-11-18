# iced Tasks: Async the Right Way

`update`에서 `Task<Message>`를 반환하면 비동기 작업의 결과가 다시 앱에 메시지로 전달된다. UI 스레드에서 블로킹 작업을 수행하지 않는다.

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

이 예시의 `update`는 `Task<Message>`를 반환하므로 모든 분기에서 작업을 반환한다. 실행할 일이 없는 분기는 `Task::none()`을 반환한다. 상태만 바꾸는 `update`는 `()`를 반환할 수 있다.

## Parallel Tasks: `Task::batch`

여러 독립적인 task를 한 번에 발사한다. 그들의 완료는 future가 끝나는 순서대로 별도 메시지로 도착한다:

```rust
Message::Initialize => Task::batch([
    Task::perform(load_user(), Message::UserLoaded),
    Task::perform(load_settings(), Message::SettingsLoaded),
    Task::perform(check_updates(), Message::UpdateChecked),
])
```

`tokio::join!`을 직접 만들지 마라. `Task::batch`는 런타임에 그들을 멀티플렉싱하는 데 필요한 것을 제공한다.

## 저장 결과에 따라 종료하기

저장 작업의 결과를 `update`에 전달한다. 성공하면 종료하고, 실패하면 편집 중인 내용을 유지한 채 오류를 표시한다. 예시는 저장 중에 들어오는 편집과 중복 저장 요청을 무시한다. `app.saving`이 참인 동안 화면에서도 두 입력을 비활성화한다. 저장 중에도 편집을 허용하는 앱은 종료 전에 저장한 리비전과 현재 리비전을 비교한다.

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

iced 0.14에서 `Task::sip`을 쓰려면 `sipper` 기능을 켠다. 이 예시의 다운로드는 진행률과 최종 결과를 전달하는 sipper를 반환한다. `Failed`에 저장된 오류를 표시하고 이 상태에서 재시도를 허용한다.

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

**`abort_on_drop`인 이유**: state가 `Downloading`에서 전환될 때, 예를 들어 사용자 취소 시 `Handle`이 drop되고 in-flight task가 abort된다. 수동 cancel 플래그가 필요 없다.

## Cancellation: `abortable`

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

`abort()` 없이 handle을 drop하는 것은 **취소가 아니다**. task는 백그라운드에서 끝나고 그 완료 Message는 여전히 전달된다. drop = cancel을 원한다면 handle에서 `.abort_on_drop()`을 호출한다.

## Sending Side Effects: `iced::exit`, `window::close`

```rust
// Close the whole app
Message::Quit => iced::exit()

// Close a specific window
Message::CloseSettings(id) => window::close(id)
```

닫기와 종료 작업은 그대로 반환한다. 이 작업은 `Task::then` 콜백에 전달할 메시지를 생성하지 않는다. 다른 창이 닫힌 뒤 남아 있는 창에서 처리할 일이 있으면 `window::close_events()`를 구독한다.

## Command-style side effects without async

future를 spawn하지 않고 "그냥 나중에 메시지를 생성하기"용:

```rust
Task::done(Message::Refresh)           // queue a message for next tick
```

await가 필요 없는 사후 완료 hook에 대해 `.then` 내부에서 유용하다.

## Async Runtime Choice

네이티브 iced 0.14에는 실행기가 필요하다. 기본 기능에는 `thread-pool`이 포함된다. 네이티브 reqwest 비동기 호출처럼 Tokio 런타임이 필요한 작업은 `tokio` 기능을 켠다.

```toml
iced = { version = "0.14", features = ["tokio"] }
```

Cargo가 여러 기능을 활성화하면 기본 네이티브 실행기는 `tokio`, `smol`, `thread-pool` 순으로 선택된다. WebAssembly에서는 브라우저 실행기를 사용한다. 의존성이 활성화한 기능은 `cargo tree -e features -i iced_futures`로 확인한다.

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

상태 검사는 HTTP 4xx와 5xx 응답을 오류 분기로 보낸다. `Task::perform`은 복제할 수 없는 오류도 그대로 전달한다. 위젯이나 시간 이동 기능에 복제 가능한 메시지가 필요하면 호출자가 요구하는 정보에 따라 표시용 문자열, 복제 가능한 오류 타입 또는 `Arc<E>`를 선택한다.
