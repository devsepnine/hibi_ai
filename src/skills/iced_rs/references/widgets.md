# iced Widgets (0.13 / 0.14)

Built-in widgets and their idiomatic usage. All examples assume `use iced::widget::*;` where relevant.

## text

```rust
use iced::widget::text;

text("Hello").size(24)                    // plain
text(counter)                             // any Display
text!("User: {}", name)                   // formatted (macro)
text("Warn").style(text::warning)         // styled via built-in function
```

**Dynamic styling**: `text("x").style(|theme| text::Style { color: Some(Color::BLACK) })`.

## button

```rust
use iced::widget::{button, text};

button("Save")
    .on_press(Message::Save)              // required for the button to be enabled
    .padding(10)
    .style(button::primary)
```

- Without `.on_press(...)`, the button renders disabled (grayed out).
- Conditional enabling: `.on_press_maybe(Some(msg_if_ready))` — `None` disables.
- Built-in stylers: `button::primary / secondary / success / danger / text`. The last one makes it look like a link.

**Passing a widget as child** (not just a string):

```rust
button(text("Save").size(18)).on_press(Message::Save)
```

## column! and row!

Macros for heterogeneous children — the common case:

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

Function form for dynamic children:

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
| `spacing(n)` | `spacing(n)` | Gap between children |
| `padding(n)` | `padding(n)` | Outer padding |
| `width(Fill)` | `width(Fill)` | Length primitive |

## container

Wraps a single child with alignment, padding, background, and styling. Think "the box around something".

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
- Centering: `container(x).center_x(Fill).center_y(Fill)`
- Cards: `container(content).style(container::rounded_box).padding(16)`
- Backgrounds: `container(content).style(|theme| container::Style { background: Some(...), ..Default::default() })`

## scrollable

Wrap any content that might exceed the viewport:

```rust
use iced::widget::scrollable;

scrollable(column(rows).spacing(4))
    .height(Fill)
    .width(Fill)
```

Control direction: `.direction(scrollable::Direction::Vertical(Default::default()))` — also `Horizontal` and `Both`. Subscribe to scroll offsets via `.on_scroll(Message::Scrolled)` to implement infinite scroll or virtualization.

## text_input

```rust
use iced::widget::text_input;

text_input("Type here...", &state.value)
    .on_input(Message::Changed)           // called on every keystroke
    .on_submit(Message::Submit)           // called on Enter
    .padding(10)
    .size(18)
```

- `on_input` disabled form: omit the call to make the input read-only.
- Multi-line: use `text_editor` instead — it has a proper `Content` type with undo/redo.
- Password: `.secure(true)` masks characters.

## checkbox / toggler / radio

```rust
checkbox("Subscribe", state.subscribed)
    .on_toggle(Message::SubscribeToggled);

toggler(state.dark_mode)
    .label("Dark mode")
    .on_toggle(Message::DarkModeToggled);

radio("Option A", Choice::A, Some(state.choice), Message::ChoiceChanged);
radio("Option B", Choice::B, Some(state.choice), Message::ChoiceChanged);
```

For a group of radios, build them in a `column!` and share the `on_change` handler.

## slider

```rust
use iced::widget::slider;

slider(0.0..=100.0, state.volume, Message::VolumeChanged)
    .step(1.0)
    .width(Fill)
```

- `Message::VolumeChanged(f32)` — you get the new value directly.
- `.step(n)` quantizes; omit for continuous.
- Vertical form: `vertical_slider(...)`.

## pick_list

Dropdown from a slice of `Clone + Display + Eq` options:

```rust
use iced::widget::pick_list;

pick_list(
    &["Option A", "Option B", "Option C"][..],
    state.selected.as_deref(),
    Message::Selected,
)
.placeholder("Choose one...");
```

For enum lists, implement `Display` and expose an `ALL: &[Self]` constant:

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
    .height(20.0)
```

## image / svg

```rust
use iced::widget::{image, svg};

image("logo.png").width(200);                 // requires `image` feature
svg("icon.svg").width(32);                    // requires `svg` feature
```

Sources can also be bytes: `image(image::Handle::from_bytes(include_bytes!("logo.png").to_vec()))`.

## Stack (Z-layered overlays)

```rust
use iced::widget::stack;

stack![
    content_behind,
    container(overlay).center_x(Fill).center_y(Fill),
]
```

For modal dialogs you usually want `stack!` with a dimmed background container on the lower layer and a card container on top.

## space

Pure spacing when `spacing()` doesn't express the layout cleanly:

```rust
use iced::widget::{Space, horizontal_space, vertical_space};

horizontal_space()          // fills remaining horizontal
vertical_space().height(16) // fixed height gap
Space::new(20, 0)           // explicit WxH
```

Use `horizontal_space()` inside a `row!` to push siblings apart ("spacer pattern").

## Length Primitives

Use these on every `.width(...)` / `.height(...)`:

```rust
use iced::{Fill, Shrink, FillPortion};
// Bare f32 also works: .width(200.0)

.width(Fill)             // take all remaining
.width(Shrink)            // size to content (default for most)
.width(FillPortion(2))    // proportional — two siblings with 2 and 1 = 2/3 and 1/3
.width(300.0)             // fixed pixels
```

**Default rule**: most widgets default to `Shrink` (width) and `Shrink` (height). `container` defaults to `Shrink` too — meaning by default it doesn't take remaining space. Set `.width(Fill)` or `.center_x(Fill)` explicitly when you want "fill the parent".

## text_editor (multi-line)

For multi-line text with undo/redo, selection, keyboard nav:

```rust
use iced::widget::text_editor;

text_editor(&state.content)
    .on_action(Message::Edit)            // text_editor::Action payload
    .height(200)
```

`state.content: text_editor::Content`. In `update`:

```rust
Message::Edit(action) => state.content.perform(action)
```

This delegates to the widget's internal edit semantics — you don't hand-roll character mutations.

## Related

- Styling each widget: `references/theming.md`
- Custom widgets via `Widget` trait (needs `advanced` feature): outside this skill's scope — see upstream `custom_widget` example.
