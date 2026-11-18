# iced Canvas: Custom 2D Drawing

For visualizations, games, plots, or any UI iced's built-in widgets don't express. Requires the `canvas` feature:

```toml
iced = { version = "0.14", features = ["canvas"] }
```

iced calls `canvas::Program::draw` with the renderer and bounds. The program creates a frame or draws into the frame provided by a cache.

## Minimal Canvas

```rust
use iced::widget::canvas::{self, Canvas, Cache, Geometry, Path, Stroke};
use iced::widget::container;
use iced::{Element, Fill, Point, Rectangle, Renderer, Theme};
use iced::mouse;

struct Diagram {
    cache: Cache,
}

impl Diagram {
    fn new() -> Self {
        Self { cache: Cache::new() }
    }

    fn view(&self) -> Element<'_, Message> {
        container(
            Canvas::new(self)
                .width(Fill)
                .height(Fill)
        )
        .padding(10)
        .into()
    }
}

impl canvas::Program<Message> for Diagram {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            let center = frame.center();
            let radius = frame.width().min(frame.height()) / 4.0;

            let circle = Path::circle(center, radius);
            frame.fill(&circle, iced::Color::from_rgb(0.2, 0.5, 1.0));

            let diameter = Path::line(
                Point::new(center.x - radius, center.y),
                Point::new(center.x + radius, center.y),
            );
            frame.stroke(
                &diameter,
                Stroke::default()
                    .with_width(3.0)
                    .with_color(iced::Color::BLACK),
            );
        });
        vec![geometry]
    }
}

#[derive(Debug, Clone)]
enum Message {}
```

## Key Concepts

### `Canvas::new(program)`

Pass the program with `Canvas::new(self)` when the view borrows it. The draw method borrows the program, and the cache uses interior mutability to store geometry.

### `type State`

`canvas::Program` has an associated `State` that iced keeps on your behalf, independent of your app state. Use it for *interaction* state that shouldn't leak into the main `App`, such as hover target, drag start, or ghost shape while dragging. For static drawing, `type State = ()`.

### `Cache`

`Cache::draw(renderer, size, closure)` runs the closure only if the cache is dirty; otherwise it returns the memoized geometry. Size changes invalidate the cached geometry. Clear it when other drawing inputs change, including data and theme colors.

```rust
// In your update or a callback on the canvas
fn update(&mut self, msg: Message) {
    match msg {
        Message::DataChanged(new_data) => {
            self.data = new_data;
            self.cache.clear();             // force repaint next frame
        }
        _ => {}
    }
}
```

Without a cache, each call to `draw` rebuilds the geometry. For a simple shape this can cost less than maintaining a cache.

## The `Frame` API

### Filling a path

```rust
let path = Path::circle(Point::new(100.0, 100.0), 40.0);
frame.fill(&path, Color::from_rgb(0.2, 0.5, 1.0));

// Or with a gradient:
use iced::widget::canvas::{gradient, Fill};
let gradient = gradient::Linear::new(Point::ORIGIN, Point::new(200.0, 0.0))
    .add_stop(0.0, Color::BLACK)
    .add_stop(1.0, Color::WHITE);
frame.fill(&path, Fill::from(gradient));
```

### Stroking a path

```rust
let path = Path::line(Point::new(0.0, 0.0), Point::new(200.0, 200.0));
frame.stroke(
    &path,
    Stroke::default()
        .with_width(2.0)
        .with_color(Color::BLACK)
        .with_line_cap(canvas::LineCap::Round),
);
```

### Text

```rust
frame.fill_text(canvas::Text {
    content: "Label".to_string(),
    position: Point::new(50.0, 50.0),
    color: Color::BLACK,
    size: 18.0.into(),
    font: iced::Font::MONOSPACE,
    align_x: iced::widget::text::Alignment::Left,
    align_y: iced::alignment::Vertical::Top,
    ..canvas::Text::default()
});
```

### Building complex paths

```rust
let path = Path::new(|b| {
    b.move_to(Point::new(0.0, 0.0));
    b.line_to(Point::new(100.0, 0.0));
    b.quadratic_curve_to(Point::new(150.0, 50.0), Point::new(100.0, 100.0));
    b.close();
});
frame.fill(&path, Color::from_rgb(0.4, 0.7, 0.4));
```

### Transforms

```rust
frame.with_save(|frame| {
    frame.translate(iced::Vector::new(100.0, 100.0));
    frame.rotate(std::f32::consts::FRAC_PI_4);
    frame.scale(1.5_f32);
    // draw here — transform is popped after with_save returns
    frame.fill(&Path::rectangle(Point::ORIGIN, Size::new(50.0, 50.0)), Color::BLACK);
});
```

`with_save` pushes/pops the transform stack so outer drawing isn't affected.

## Interaction: `update` / `mouse_interaction`

For interactive canvases, implement `update` and, optionally, `mouse_interaction`:

```rust
use iced::{Point, Rectangle, Renderer, Theme, mouse};
use iced::widget::Action;
use iced::widget::canvas::{self, Geometry, Path, Stroke};

struct Board;

#[derive(Default)]
struct InteractionState {
    drag_start: Option<Point>,
}

#[derive(Debug, PartialEq)]
enum Message {
    DragStarted(Point),
    Dragging(Point),
    DragEnded(Point),
    DragCancelled,
}

impl canvas::Program<Message> for Board {
    type State = InteractionState;

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        if let (Some(start), Some(end)) = (state.drag_start, cursor.position_in(bounds)) {
            frame.stroke(&Path::line(start, end), Stroke::default());
        }
        vec![frame.into_geometry()]
    }

    fn update(
        &self,
        state: &mut Self::State,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Action<Message>> {
        use mouse::Event as Mouse;

        let message = match event {
            canvas::Event::Mouse(Mouse::ButtonPressed(mouse::Button::Left)) => {
                let pos = cursor.position_in(bounds)?;
                state.drag_start = Some(pos);
                Message::DragStarted(pos)
            }
            canvas::Event::Mouse(Mouse::CursorMoved { .. }) if state.drag_start.is_some() => {
                Message::Dragging(cursor.position_in(bounds)?)
            }
            canvas::Event::Mouse(Mouse::ButtonReleased(mouse::Button::Left)) => {
                state.drag_start.take()?;
                cursor.position_in(bounds).map_or(Message::DragCancelled, Message::DragEnded)
            }
            canvas::Event::Window(iced::window::Event::Unfocused) => {
                state.drag_start.take()?;
                return Some(Action::publish(Message::DragCancelled));
            }
            _ => return None,
        };
        Some(Action::publish(message).and_capture())
    }

    fn mouse_interaction(
        &self,
        state: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if state.drag_start.is_some() {
            mouse::Interaction::Grabbing
        } else if cursor.is_over(bounds) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }
}
```

In iced 0.14, `update` borrows the event and returns `Option<Action<Message>>`. Apply `.and_capture()` to handled mouse events. This example cancels a drag on release outside the canvas or loss of window focus, and ignores releases without an active drag.

## Multiple Layers

Return multiple `Geometry` values for composite drawings that need separate caches:

```rust
fn draw(&self, state: &Self::State, renderer: &Renderer, theme: &Theme, bounds: Rectangle, cursor: mouse::Cursor) -> Vec<Geometry> {
    let background = self.background_cache.draw(renderer, bounds.size(), |frame| {
        // expensive static grid
    });
    let foreground = self.foreground_cache.draw(renderer, bounds.size(), |frame| {
        // cheap dynamic cursor, annotations, etc.
    });
    vec![background, foreground]
}
```

Clear only the cache whose inputs changed. This is the main performance pattern for non-trivial canvases.

## Performance Checklist

- [ ] One `Cache` per static layer; invalidate only on change
- [ ] Long-lived `Path` objects precomputed in state, not re-built each draw
- [ ] Text `size` kept modest; very large rasterized text is slow
- [ ] Avoid `draw` doing allocation, such as `format!` every frame; pre-compute strings in `update`
- [ ] For really heavy drawing such as charts with 100k points, consider downsampling in `update` and drawing the result

## When NOT to use Canvas

- Static images: use `image`/`svg` widget.
- Grids of the same widget: use `column!`/`row!` of actual widgets; they get a11y and focus for free.
- 3D: canvas is 2D only. For 3D, drop down to `wgpu` directly and wrap a custom widget.
