# iced Gotchas: Things That Will Bite You

컴파일 타임에 크래시하지 않지만 혼란스러운 런타임 동작이나 코드 마찰을 일으키는 미묘한 이슈들의 카탈로그이다. 각 항목: what, why, fix.

## 1. 위젯과 기능에 따른 메시지 제약

버튼을 엘리먼트로 변환하려면 메시지를 복제할 수 있어야 한다. 앱을 실행할 때 `time-travel` 기능은 `Clone`을, `debug` 기능은 `Debug`를 요구한다. 두 기능은 Cargo에서 설정한다. 일반 작업은 `std::io::Error`를 그대로 반환할 수 있다.

메시지에 `Clone`이 필요하면 오류를 UI에 필요한 필드로 변환하거나 `Arc`로 원본 오류를 보존한다.

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

**Fix**: view가 소유된 문자열이 필요하다면, `self`에 `self.display_name: String`처럼 소유된 채로 저장하거나, widget 호출 내부에서 소유된 채로 빌드한다:

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

**Fix**: 항상 `Task::perform`을 사용하고 스트리밍은 `Task::sip`을 사용한다:

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

## 5. 네이티브 실행기 선택

네이티브 iced 0.14에서 `tokio`와 `smol`이 모두 켜져 있으면 Tokio를 선택한다. Tokio 의존성이 있다면 기본 렌더러와 플랫폼 기능을 유지한 채 Tokio를 추가한다.

```toml
iced = { version = "0.14", features = ["tokio"] }
```

기본 기능을 끈다면 렌더러와 네이티브 실행기를 선택하고, Linux에서는 `x11`이나 `wayland` 중 하나 이상의 디스플레이 백엔드를 켠다. `cargo tree -e features -i iced`로 실제 활성화된 기능을 확인한다.

## 6. 뷰를 재구성할 때 반복되는 작업

같은 목록을 표시할 때마다 복제하고 정렬한다면 입력이 바뀔 때 정렬해 둔다. 뷰는 저장된 순서대로 표시한다.

```rust
Message::DataLoaded(mut items) => {
    items.sort_by_key(|item| item.priority);
    self.items = items;
    Task::none()
}
```

## 7. 자식 크기에 맞춰진 컨테이너에서 중앙 정렬하기

**What**: 윈도우에서 중앙에 두기 위해 무언가를 `container`로 감쌌다. 좌상단에 자리잡는다.

자식이 차지한 공간에 딱 맞는 컨테이너에는 중앙 정렬에 쓸 여백이 없다. 중앙 정렬할 축에 공간을 할당한다.

**Fix**: 명시적으로 Fill-axis를 준다:

```rust
container(content)
    .center_x(Fill)           // width = Fill, content horizontally centered
    .center_y(Fill)           // height = Fill, content vertically centered
```

또는 `.width(Fill).height(Fill).align_x(Center).align_y(Center)`를 조합. 같은 결과, 더 verbose.

## 8. Forgetting `.on_press` Renders A Disabled Button

**What**: 버튼이 회색으로 보이고 클릭되지 않는다.

**Why**: `.on_press(message)` 없는 `button`은 disabled로 간주되고 그에 따라 스타일이 적용된다.

**Fix**: 항상 `.on_press(...)`를 호출한다. 조건부 활성화:

```rust
button("Submit").on_press_maybe(
    if self.form_valid() { Some(Message::Submit) } else { None }
)
```

## 9. `print!` / `eprintln!` Doesn't Corrupt iced, But Silently Logs

**What**: `println!`의 로그가 보이는 어디에도 나타나지 않는다.

**Why**, 그리고 *문제*가 아닌 이유: ratatui와 달리 iced는 윈도우를 소유하고 터미널을 소유하지 않는다. iced 앱에서 stdout/stderr로 로깅하는 것은 무해하지만 일반적인 사용자 설치에서 앱이 터미널에서 시작되지 않기 때문에 보통 보이지 않는다.

**Fix**: 사후에 필요한 모든 것에 대해 파일 appender와 함께 `tracing`이나 `log`를 사용한다. 실행 중인 GUI에서 stderr를 읽으려고 하지 마라.

## 10. 테마 변경

테마 설정 메시지에서 앱 상태를 갱신하고 빌더의 테마 함수가 그 값을 읽는지 확인한다. 런타임은 메시지를 처리한 후 뷰를 다시 구성한다.

## 11. 조건부 구독이 다시 시작되는 경우

`Subscription::run`은 스트림을 만드는 함수 포인터로 구독을 식별한다. 같은 구독을 계속 반환하면 작업이 유지된다. 구독을 제외하면 작업이 중단되고, 나중에 다시 반환하면 새로 시작한다. 데이터도 식별에 사용하려면 `Subscription::run_with`에 전달한다.

## 12. 모든 창이 닫힌 뒤에도 실행해야 하는 앱

일반 앱은 마지막 창이 닫히면 종료한다. 창 없이 대기하는 트레이 앱은 `iced::daemon`을 사용하고 종료 시점에 `iced::exit()`를 반환한다.

## 13. 텍스트 줄바꿈 너비

긴 텍스트가 줄바꿈되는 위치는 부모가 제공하는 너비와 텍스트의 줄바꿈 설정에 따라 달라진다. 카드 안에서 줄바꿈 너비에 상한을 두려면 컨테이너의 최대 너비를 지정한다.

```rust
container(text(long_string)).max_width(480)
```

## 14. Custom Widgets Need the `advanced` Feature

**What**: `widget::Widget` trait를 구현하고 컴파일러가 그것을 찾을 수 없다.

**Why**: low-level widget trait는 `advanced` feature 뒤에 게이팅되어 있다.

**Fix**:

```toml
iced = { version = "0.14", features = ["advanced", ...] }
```

그런 다음 `use iced::advanced::widget::Widget;`. 대부분의 앱은 이를 절대 필요로 하지 않는다. 먼저 빌트인 widget을 조합한다.
