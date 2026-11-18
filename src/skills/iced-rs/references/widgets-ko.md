# iced 0.14 위젯

빌트인 widget과 그 관용적 사용법. 모든 예제는 적절히 `use iced::widget::*;`를 가정한다.

## text

```rust
use iced::widget::text;

text("Hello").size(24)                    // plain
text(counter)                             // u64
text(value.to_string())                   // custom Display type
text!("User: {}", name)                   // formatted (macro)
text("Warn").style(text::warning)         // styled via built-in function
```

**동적 스타일링**: `text("x").style(|theme| text::Style { color: Some(Color::BLACK) })`.

## button

```rust
use iced::widget::{button, text};

button("Save")
    .on_press(Message::Save)              // required for the button to be enabled
    .padding(10)
    .style(button::primary)
```

- `.on_press(...)` 없이는 button이 회색의 disabled로 렌더된다.
- 조건부 활성화: `.on_press_maybe(Some(msg_if_ready))`, `None`이면 disabled.
- 빌트인 styler: `button::primary / secondary / success / danger / text`. 마지막은 링크처럼 보이게 한다.

**자식으로 widget 전달**, 문자열뿐만 아니라:

```rust
button(text("Save").size(18)).on_press(Message::Save)
```

## column! and row!

heterogeneous 자식을 위한 매크로이며 일반적인 경우이다:

```rust
column![
    text("Title"),
    button("Go").on_press(Message::Go),
    text("Footer"),
]
.spacing(10)
.padding(20)
.align_x(Alignment::Center)
```

동적 자식의 함수 형태:

```rust
use iced::widget::Column;

let rows: Vec<Element<_>> = items.iter()
    .map(|i| row![text(&i.name), text(&i.value)].into())
    .collect();
Column::with_children(rows).spacing(8)
```

**Axis methods**:

| Column | Row | Meaning |
|---|---|---|
| `align_x(Center)` | `align_y(Center)` | Cross-axis alignment |
| `spacing(n)` | `spacing(n)` | 자식 간 간격 |
| `padding(n)` | `padding(n)` | 외부 padding |
| `width(Fill)` | `width(Fill)` | Length primitive |

## container

정렬, padding, 배경, 스타일링과 함께 단일 자식을 wrap한다. "무언가 주위의 box"라고 생각하라.

```rust
use iced::{Fill, Element};
use iced::widget::container;

container(text("Centered"))
    .center_x(Fill)          // center horizontally, take full width
    .center_y(Fill)           // center vertically, take full height
    .padding(20)
    .style(container::rounded_box)
```

**Common uses**:
- 중앙 정렬: `container(x).center_x(Fill).center_y(Fill)`
- Cards: `container(content).style(container::rounded_box).padding(16)`
- 배경: `container(content).style(|theme| container::Style { background: Some(...), ..Default::default() })`

## scrollable

뷰포트를 초과할 수 있는 어떤 콘텐츠든 wrap한다:

```rust
use iced::widget::scrollable;

scrollable(column(rows).spacing(4))
    .height(Fill)
    .width(Fill)
```

방향 제어: `.direction(scrollable::Direction::Vertical(Default::default()))`. `Horizontal`과 `Both`도 있다. 무한 스크롤이나 가상화를 구현하기 위해 `.on_scroll(Message::Scrolled)`로 스크롤 오프셋을 구독한다.

## text_input

```rust
use iced::widget::text_input;

text_input("Type here...", &state.value)
    .on_input(Message::Changed)           // called on every keystroke
    .on_submit(Message::Submit)           // called on Enter
    .padding(10)
    .size(18)
```

- `on_input`을 생략하면 입력 위젯이 비활성화된다.
- 여러 줄을 편집하려면 앱 상태에 `text_editor::Content`를 둔다.
- 비밀번호: `.secure(true)`가 문자를 마스킹한다.

## checkbox / toggler / radio

```rust
checkbox(state.subscribed)
    .label("Subscribe")
    .on_toggle(Message::SubscribeToggled);

toggler(state.dark_mode)
    .label("Dark mode")
    .on_toggle(Message::DarkModeToggled);

radio("Option A", Choice::A, Some(state.choice), Message::ChoiceChanged);
radio("Option B", Choice::B, Some(state.choice), Message::ChoiceChanged);
```

라디오 그룹의 경우, `column!`에 빌드하고 `on_change` 핸들러를 공유한다.

## slider

```rust
use iced::widget::slider;

slider(0.0..=100.0, state.volume, Message::VolumeChanged)
    .step(1.0)
    .width(Fill)
```

- `Message::VolumeChanged(f32)`: 새 값을 직접 받는다.
- 기본 간격은 1이다. `0.0..=1.0`에서 소수 값을 선택하려면 `.step(0.01)`처럼 더 작은 간격을 지정한다.
- 수직 형태: `vertical_slider(...)`.

## pick_list

`Clone + PartialEq + ToString` 옵션의 슬라이스로부터 드롭다운:

```rust
use iced::widget::pick_list;

pick_list(
    &["Option A", "Option B", "Option C"][..],
    state.selected.as_deref(),
    Message::Selected,
)
.placeholder("Choose one...");
```

enum 리스트의 경우, `Display`를 구현하고 `ALL: &[Self]` 상수를 노출한다:

```rust
pick_list(Theme::ALL, Some(&state.theme), Message::ThemeChanged)
```

## tooltip

```rust
use iced::widget::{tooltip, button};

tooltip(
    button("Delete").on_press(Message::Delete),
    "Permanently remove this item",
    tooltip::Position::Bottom,
)
.gap(4)
.style(container::rounded_box)
```

## progress_bar

```rust
use iced::widget::progress_bar;

progress_bar(0.0..=1.0, state.progress)       // expects value in the given range
    .girth(20.0)
```

## image / svg

```rust
use iced::widget::{image, svg};

image("logo.png").width(200);                 // requires `image` feature
svg("icon.svg").width(32);                    // requires `svg` feature
```

소스는 bytes일 수도 있다: `image(image::Handle::from_bytes(include_bytes!("logo.png").to_vec()))`.

## Stack: Z-layered overlays

```rust
use iced::widget::stack;

stack![
    content_behind,
    container(overlay).center_x(Fill).center_y(Fill),
]
```

모달 대화상자의 경우 보통 하위 레이어에 어둡게 처리된 배경 컨테이너와 상단에 카드 컨테이너로 `stack!`을 원한다.

## space

`spacing()`이 레이아웃을 깔끔하게 표현하지 못할 때 순수 spacing:

```rust
use iced::widget::{space, Space};

let horizontal_gap = space::horizontal();
let vertical_gap = space::vertical().height(16);
let fixed_gap = Space::new().width(20).height(0);
```

0.14에서 `row!`의 남은 가로 공간을 채우려면 `space::horizontal()`을 사용한다. 0.13에서는 `horizontal_space()`를 사용한다.

## Length Primitives

모든 `.width(...)` / `.height(...)`에 이를 사용한다:

```rust
use iced::{Fill, Shrink, FillPortion};
// Bare f32 also works: .width(200.0)

.width(Fill)             // take all remaining
.width(Shrink)            // size to content (default for most)
.width(FillPortion(2))    // proportional — two siblings with 2 and 1 = 2/3 and 1/3
.width(300.0)             // fixed pixels
```

컨테이너는 축마다 자식의 크기 힌트를 확인한다. 힌트가 `Fill` 또는 `FillPortion`이면 기본값은 `Fill`이고 나머지는 `Shrink`다.

## text_editor, multi-line

여러 줄 입력, 선택, 키보드 이동을 지원하는 편집기:

```rust
use iced::widget::text_editor;

text_editor(&state.content)
    .on_action(Message::Edit)            // text_editor::Action payload
    .height(200)
```

`state.content: text_editor::Content`. `update`에서:

```rust
Message::Edit(action) => state.content.perform(action)
```

`Content::perform`으로 편집 동작을 적용한다. 실행 취소와 다시 실행이 필요하면 앱에서 편집 이력을 관리한다.

## Related

- 각 widget 스타일링: `references/theming.md`
- `Widget` trait를 통한 커스텀 widget은 `advanced` feature가 필요하다. 이 skill의 범위 밖이므로 upstream `custom_widget` 예제 참조.
