# iced Gotchas — Things That Will Bite You

컴파일 타임에 크래시하지 않지만 혼란스러운 런타임 동작이나 코드 마찰을 일으키는 미묘한 이슈들의 카탈로그이다. 각 항목: what, why, fix.

## 1. `Message` Must Be `Clone` — And Everything Inside It Too

**What**: 어떤 variant가 `std::io::Error`나 `reqwest::Error` 같은 non-`Clone` 타입을 보유하기 때문에 `Message`에 `#[derive(Debug, Clone)]`이 실패한다.

**Why**: iced 런타임은 이벤트 큐 전반에 걸쳐 메시지를 clone한다. `Clone` 없이는 전체 앱이 컴파일되지 않는다.

**Fix**: 비동기 경계에서 에러를 `String`으로 변환하거나, `Arc<T>`로 감싼다:

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

더 풍부한 에러:

```rust
#[derive(Debug, Clone)]
enum Message {
    Fetched(Result<Data, Arc<anyhow::Error>>),  // Arc<E> is Clone even if E isn't
}
```

## 2. `Element<'a, Message>` Lifetime Confusion

**What**: 소유된 문자열 또는 다른 소유된 데이터에서 `Element`를 빌드하려는 `view` 함수가 borrow-check에 실패한다.

**Why**: `Element<'a, Message>`는 `&self`에서 빌린다. `text(&self.name)` 같은 widget은 App state에 대한 참조를 받고 `Element`의 lifetime은 그 borrow에 묶인다.

**Fix**: view가 소유된 문자열이 필요하다면, `self`에 소유된 채로 저장하거나 (`self.display_name: String`), widget 호출 내부에서 소유된 채로 빌드한다:

```rust
// Borrow — preferred when the data already lives on self
text(&self.name)

// Owned — when the string is computed each render
text(format!("Count: {}", self.counter))
```

이렇게 하지 마라:

```rust
fn view(&self) -> Element<'_, Message> {
    let formatted = format!("Count: {}", self.counter);
    text(&formatted).into()                // BORROW ERROR — formatted dies at return
}
```

이렇게 하라:

```rust
fn view(&self) -> Element<'_, Message> {
    text(format!("Count: {}", self.counter)).into()    // widget owns the String
}
```

## 3. `iced::run` Doesn't Work With Non-`Default` State

**What**: state를 `(u64,)`에서 `Connection`을 보유하는 struct로 전환했고 갑자기 `iced::run`이 컴파일되지 않는다.

**Why**: `iced::run`은 초기 값을 구성할 수 있도록 `State: Default`가 필요하다. non-default 필드를 가진 struct는 `Default`를 impl할 수 없다.

**Fix**: `iced::application()`으로 옮긴다. 거기서 `new` 함수를 제공한다:

```rust
iced::application(App::new, App::update, App::view).run()

impl App {
    fn new() -> (Self, Task<Message>) {
        (Self { conn: Connection::open().unwrap(), ... }, Task::none())
    }
}
```

## 4. `tokio::spawn` Inside `update` Vanishes Into the Void

**What**: `update` 내에서 비동기 task를 `tokio::spawn`하여 완료 시 UI를 업데이트하기를 바란다. 그렇지 않다. UI는 그저 가만히 있다.

**Why**: spawn된 task는 실행되지만, 그 출력은 절대 `Message`가 되지 않는다. 런타임은 그것이 존재하는지 모른다.

**Fix**: 항상 `Task::perform` (또는 스트리밍은 `Task::sip`)을 사용한다:

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

`references/tasks.md` 참조.

## 5. Two Async Runtimes at Once

**What**: 다중 런타임 초기화 에러로 시작 시 앱이 패닉한다.

**Why**: `Cargo.toml`이 `iced`에서 `tokio`와 `smol` 기능을 모두 활성화했거나, 추이적 의존성 중 하나가 다른 것을 가져왔다.

**Fix**: 하나를 선택한다. 명시적으로 추가한다:

```toml
iced = { version = "0.14", features = ["tokio"], default-features = false }
```

그런 다음 `cargo tree | grep -E "tokio|async-std|smol"`로 잔재를 찾는다.

## 6. `view` Re-runs Every Frame — Don't Do Work In It

**What**: UI가 lag를 느끼고 CPU 사용량이 높으며 화면에 사소한 widget만 있는데 framerate가 낮다.

**Why**: `view`는 redraw가 발생하는 매 프레임 (보통 각 message에서) 호출된다. `view`가 처음부터 문자열을 포맷하거나 `Vec`를 정렬하거나 파일을 읽는다면, 그 비용을 framerate로 곱한다.

**Fix**: `update`에서 미리 계산하고 `self`에 캐시한다:

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

**What**: 윈도우에서 중앙에 두기 위해 무언가를 `container`로 감쌌다. 좌상단에 자리잡는다.

**Why**: `container`는 두 축 모두 `Shrink`로 기본값이다 — 자식 크기에 맞춘다. 중앙 정렬은 컨테이너가 *내부에* 중앙 정렬할 공간이 있을 때만 작동한다.

**Fix**: 명시적으로 Fill-axis를 준다:

```rust
container(content)
    .center_x(Fill)           // width = Fill, content horizontally centered
    .center_y(Fill)           // height = Fill, content vertically centered
```

또는 `.width(Fill).height(Fill).align_x(Center).align_y(Center)`를 조합 — 같은 결과, 더 verbose.

## 8. Forgetting `.on_press` Renders A Disabled Button

**What**: 버튼이 회색으로 보이고 클릭되지 않는다.

**Why**: `.on_press(message)` 없는 `button`은 disabled로 간주되고 그에 따라 스타일이 적용된다.

**Fix**: 항상 `.on_press(...)`를 호출한다. 조건부 활성화:

```rust
button("Submit").on_press_maybe(
    if self.form_valid() { Some(Message::Submit) } else { None }
)
```

## 9. `print!` / `eprintln!` Doesn't Corrupt iced (but silently logs)

**What**: `println!`의 로그가 보이는 어디에도 나타나지 않는다.

**Why** (그리고 *문제*가 아님): ratatui와 달리 iced는 윈도우를 소유하고 터미널을 소유하지 않는다 — iced 앱에서 stdout/stderr로 로깅하는 것은 무해하지만 일반적인 사용자 설치에서 앱이 터미널에서 시작되지 않기 때문에 보통 보이지 않는다.

**Fix**: 사후에 필요한 모든 것에 대해 파일 appender와 함께 `tracing`이나 `log`를 사용한다. 실행 중인 GUI에서 stderr를 읽으려고 하지 마라.

## 10. Changing Theme in `update` Doesn't Repaint Until Next Message

**What**: Theme-switch 메시지가 발사되지만, 다음 키 입력이나 마우스 이동까지 UI가 시각적으로 업데이트되지 않는다.

**Why**: iced는 *메시지에 의해 트리거된* state 변경 시 redraw한다. `theme(&self)` 함수는 매 프레임마다 호출되지만, 프레임은 이벤트에서만 발생한다.

**Fix**: 수정은 "redraw를 강제하기 위해 follow-up 메시지를 emit"이며, iced가 자동으로 처리한다 — 멈춘 프레임이 보인다면, `Message` mutation에 묶이지 않은 state가 있을 수 있다. theme switch가 *실제로* `update`를 통과하는지 확인한다. 그렇다면, repaint된다.

사용자 입력 없이도 연속 애니메이션이 필요한 앱은 `time::every` subscription을 추가한다 — 고정 비율로 프레임을 전달하고 repaint를 보장한다.

## 11. Subscription Restarts When the Closure "Shape" Changes

**What**: subscription을 조건부로 반환하고, 조건이 뒤집힐 때마다 subscription의 작업이 재시작된다 (file watcher가 재읽기, channel이 재오픈).

**Why**: iced는 *identity*로 subscription을 diff한다 — closure의 captures와 type. 새 closure 인스턴스는 새 subscription이다.

**Fix**: subscription의 producer 함수를 프레임 간에 안정적으로 유지한다. 조건부로 캡처하고 있다면 closure를 명명된 `fn`으로 옮긴다:

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

**What**: 앱이 여러 윈도우를 사용하고; 메인 윈도우를 닫으면 전체 프로세스가 종료된다. 트레이 스타일의 lingering 앱을 원했다.

**Why**: `iced::application()`은 primary 윈도우가 닫힐 때 종료된다. 영구 background 앱은 `iced::daemon()`을 사용한다 — primary 윈도우 없음, 명시적인 `iced::exit()`에 묶인 lifecycle.

**Fix**: 앱이 단일 윈도우보다 오래 살아야 한다면 `iced::daemon`으로 마이그레이션한다. upstream `multi_window` 예제 참조.

## 13. Text Wrap Not Kicking In

**What**: 긴 텍스트가 wrap되지 않고 오른쪽 가장자리를 벗어나 그려진다.

**Why**: `text`는 컨테이너 내에서 wrap되지만, 컨테이너 자체가 `Shrink`(기본값)라면 wrap *할* 너비가 없다.

**Fix**: 체인 위 어딘가에 명시적 너비를 설정한다:

```rust
// container has no width → text won't wrap
container(text(long_string))

// width set → text wraps
container(text(long_string)).width(Fill)
// or column![..text(long_string)..].width(Fill)
```

## 14. Custom Widgets Need the `advanced` Feature

**What**: `widget::Widget` trait를 구현하고 컴파일러가 그것을 찾을 수 없다.

**Why**: low-level widget trait는 `advanced` feature 뒤에 게이팅되어 있다.

**Fix**:

```toml
iced = { version = "0.14", features = ["advanced", ...] }
```

그런 다음 `use iced::advanced::widget::Widget;`. 대부분의 앱은 이를 절대 필요로 하지 않는다 — 먼저 빌트인 widget을 조합한다.
