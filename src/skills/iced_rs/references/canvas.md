# iced Canvas — Custom 2D Drawing

For visualizations, games, plots, or any UI iced's built-in widgets don't express. Requires the `canvas` feature:

```toml
iced = { version = "0.14", features = ["canvas"] }
```

The model: you implement `canvas::Program<Message>` for a type, and iced calls your `draw` method with a `Frame` you paint into. A `Cache` memoizes the painted geometry across frames when inputs don't change.

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
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            let palette = theme.extended_palette();
            let center = frame.center();
            let radius = frame.width().min(frame.height()) / 4.0;

            let circle = Path::circle(center, radius);
            frame.fill(&circle, palette.primary.strong.color);

            let diameter = Path::line(
                Point::new(center.x - radius, center.y),
                Point::new(center.x + radius, center.y),
            );
            frame.stroke(
                &diameter,
                Stroke::default()
                    .with_width(3.0)
                    .with_color(palette.background.base.color),
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

Pass any `&impl canvas::Program<Message>` — you usually pass `self`. Canvas takes `&self` (not `&mut`), so mutation during draw is not allowed. That's enforced by `draw` taking `&self`.

### `type State`

`canvas::Program` has an associated `State` that iced keeps on your behalf, independent of your app state. Use it for *interaction* state that shouldn't leak into the main `App` (hover target, drag start, ghost shape while dragging). For static drawing, `type State = ()`.

### `Cache`

`Cache::draw(renderer, size, closure)` runs the closure only if the cache is dirty; otherwise it returns the memoized geometry. Invalidate explicitly when inputs change:

```rust
// In your update or a callback on the canvas
fn update(&mut self, msg: Message) {
    match msg {
        Message::DataChanged => {
            self.data = new_data;
            self.cache.clear();             // force repaint next frame
        }
        _ => {}
    }
}
```

Without a `Cache`, draw runs on every frame — fine for simple drawings, expensive for heavy ones.

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
    horizontal_alignment: alignment::Horizontal::Left,
    vertical_alignment: alignment::Vertical::Top,
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
    frame.scale(1.5);
    // draw here — transform is popped after with_save returns
    frame.fill(&Path::rectangle(Point::ORIGIN, Size::new(50.0, 50.0)), Color::BLACK);
});
```

`with_save` pushes/pops the transform stack so outer drawing isn't affected.

## Interaction — `update` / `mouse_interaction`

For interactive canvases, implement `update` and (optionally) `mouse_interaction`:

```rust
impl canvas::Program<Message> for Board {
    type State = InteractionState;

    fn update(
        &self,
        state: &mut Self::State,
        event: canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> (canvas::event::Status, Option<Message>) {
        use canvas::event::Status;
        use mouse::Event as Mouse;

        let Some(pos) = cursor.position_in(bounds) else {
            return (Status::Ignored, None);
        };

        match event {
            canvas::Event::Mouse(Mouse::ButtonPressed(mouse::Button::Left)) => {
                state.drag_start = Some(pos);
                (Status::Captured, Some(Message::DragStarted(pos)))
            }
            canvas::Event::Mouse(Mouse::CursorMoved { .. }) if state.drag_start.is_some() => {
                (Status::Captured, Some(Message::Dragging(pos)))
            }
            canvas::Event::Mouse(Mouse::ButtonReleased(mouse::Button::Left)) => {
                state.drag_start = None;
                (Status::Captured, Some(Message::DragEnded(pos)))
            }
            _ => (Status::Ignored, None),
        }
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

**Return `Status::Captured`** when you handle an event so iced doesn't route it to other widgets. Return `Status::Ignored` to let it bubble.

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
- [ ] Avoid `draw` doing allocation (`format!` every frame) — pre-compute strings in `update`
- [ ] For really heavy drawing (charts with 100k points), consider downsampling in `update` and drawing the result

## When NOT to use Canvas

- Static images — use `image`/`svg` widget.
- Grids of the same widget — use `column!`/`row!` of actual widgets; they get a11y and focus for free.
- 3D — canvas is 2D only. For 3D, drop down to `wgpu` directly and wrap a custom widget.
