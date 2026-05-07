# iced Canvas — Custom 2D Drawing

시각화, 게임, 플롯, iced의 빌트인 widget으로 표현할 수 없는 어떤 UI든. `canvas` feature 필요:

```toml
iced = { version = "0.14", features = ["canvas"] }
```

모델: 타입에 대해 `canvas::Program<Message>`를 구현하고, iced가 페인팅할 `Frame`을 주는 `draw` 메서드를 호출한다. `Cache`는 입력이 변경되지 않을 때 프레임 간에 페인팅된 geometry를 메모이즈한다.

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

어떤 `&impl canvas::Program<Message>`든 전달한다 — 보통 `self`를 전달한다. Canvas는 `&self`를 받으므로 (not `&mut`), draw 중에 mutation은 허용되지 않는다. `draw`가 `&self`를 받는 것으로 강제된다.

### `type State`

`canvas::Program`은 iced가 앱 state와 독립적으로 사용자 대신 보관하는 연관 `State`를 가진다. 메인 `App`에 누출되지 않아야 하는 *interaction* state(hover target, drag start, 드래그 중 ghost shape)에 사용한다. 정적 그리기에는 `type State = ()`.

### `Cache`

`Cache::draw(renderer, size, closure)`는 캐시가 dirty할 때만 closure를 실행한다; 그렇지 않으면 메모이즈된 geometry를 반환한다. 입력 변경 시 명시적으로 invalidate한다:

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

`Cache` 없이는 draw가 매 프레임 실행된다 — 간단한 그림에는 괜찮지만 무거운 그림에는 비싸다.

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

`with_save`는 transform 스택을 push/pop하므로 외부 그리기에 영향을 주지 않는다.

## Interaction — `update` / `mouse_interaction`

인터랙티브 캔버스의 경우 `update`와 (선택적으로) `mouse_interaction`을 구현한다:

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

이벤트를 처리할 때 **`Status::Captured`를 반환**하여 iced가 다른 widget으로 라우팅하지 않도록 한다. bubble하도록 하려면 `Status::Ignored`를 반환한다.

## Multiple Layers

별도 캐시가 필요한 합성 그리기를 위해 여러 `Geometry` 값을 반환한다:

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

입력이 변경된 캐시만 clear한다. 이는 사소하지 않은 캔버스의 주된 성능 패턴이다.

## Performance Checklist

- [ ] 정적 레이어당 하나의 `Cache`; 변경 시에만 invalidate
- [ ] 매 draw마다 재구축되지 않고 state에 미리 계산된 long-lived `Path` 객체
- [ ] Text `size`를 적당히 유지; 매우 큰 래스터화된 텍스트는 느리다
- [ ] `draw`에서 할당을 피한다 (매 프레임 `format!`) — `update`에서 문자열을 미리 계산
- [ ] 정말 무거운 그리기 (10만 포인트 차트)의 경우, `update`에서 다운샘플링하고 결과를 그리는 것을 고려

## When NOT to use Canvas

- 정적 이미지 — `image`/`svg` widget을 사용한다.
- 동일한 widget의 grid — 실제 widget의 `column!`/`row!`를 사용한다; a11y와 focus를 무료로 얻는다.
- 3D — 캔버스는 2D 전용. 3D는 `wgpu`로 직접 떨어져 커스텀 widget으로 wrap한다.
