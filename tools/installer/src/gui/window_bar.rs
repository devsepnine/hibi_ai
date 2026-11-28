use iced::advanced::{
    Clipboard, Layout, Shell, Widget, layout, mouse, renderer,
    widget::{Tree, tree},
};
use iced::widget::{Space, row};
use iced::{Element, Event, Fill, Length, Rectangle, Renderer, Size, Task, Theme, window};

use super::{Message, runtime::Runtime, theme_button};

pub(super) const HEIGHT: f32 = 52.0;

pub(super) fn settings() -> window::Settings {
    window::Settings {
        size: Size::new(1120.0, 780.0),
        min_size: Some(Size::new(760.0, 560.0)),
        #[cfg(not(target_os = "macos"))]
        icon: {
            const PIXELS: &[u8; 64 * 64 * 4] = include_bytes!("../../assets/hibi-window.rgba");
            Some(window::icon::from_rgba(PIXELS.to_vec(), 64, 64).expect("valid Hibi window icon"))
        },
        #[cfg(target_os = "linux")]
        platform_specific: window::settings::PlatformSpecific {
            application_id: "hibi".into(),
            ..Default::default()
        },
        exit_on_close_request: false,
        decorations: cfg!(target_os = "macos"),
        #[cfg(target_os = "macos")]
        platform_specific: window::settings::PlatformSpecific {
            title_hidden: true,
            titlebar_transparent: true,
            fullsize_content_view: true,
        },
        ..Default::default()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Command {
    Drag,
    #[cfg(target_os = "windows")]
    TouchMove(iced::Point),
    Maximize,
    #[cfg(not(target_os = "macos"))]
    Minimize,
    #[cfg(not(target_os = "macos"))]
    Resize(window::Direction),
}

impl Command {
    pub(super) fn run(self) -> Task<Message> {
        window::oldest().and_then(move |id| match self {
            Self::Drag => window::drag(id),
            #[cfg(target_os = "windows")]
            Self::TouchMove(point) => window::is_maximized(id).then(move |maximized| {
                if maximized {
                    Task::none()
                } else {
                    window::move_to(id, point)
                }
            }),
            Self::Maximize => window::toggle_maximize(id),
            #[cfg(not(target_os = "macos"))]
            Self::Minimize => window::minimize(id, true),
            #[cfg(not(target_os = "macos"))]
            Self::Resize(direction) => window::drag_resize(id, direction),
        })
    }
}

pub(super) fn view(state: &Runtime) -> Element<'_, Message> {
    #[cfg(target_os = "macos")]
    let controls: Element<'_, Message> = Space::new().width(80).into();
    #[cfg(not(target_os = "macos"))]
    let controls: Element<'_, Message> = {
        use super::{actions::Action, styles};
        use iced::widget::{button, svg, text, tooltip};
        let control =
            |image: &'static [u8], label: &'static str, action| -> Element<'static, Message> {
                let icon = svg(svg::Handle::from_memory(image))
                    .width(12)
                    .height(12)
                    .style(|theme: &Theme, _| svg::Style {
                        color: Some(theme.palette().text),
                    });
                tooltip(
                    button(icon)
                        .width(32)
                        .height(32)
                        .padding(10)
                        .style(styles::subtle)
                        .on_press(action),
                    text(state.language.text(label)).size(12),
                    tooltip::Position::Bottom,
                )
                .style(styles::popup)
                .into()
            };
        row![
            control(
                include_bytes!("../../assets/window-close.svg"),
                "Close application",
                Message::Action(Action::RequestClose)
            ),
            control(
                include_bytes!("../../assets/window-minimize.svg"),
                "Minimize",
                Message::Window(Command::Minimize)
            ),
            control(
                include_bytes!("../../assets/window-maximize.svg"),
                "Maximize or restore",
                Message::Window(Command::Maximize)
            ),
        ]
        .into()
    };
    row![
        controls,
        iced::widget::container(
            iced::widget::text("HIBI AI")
                .font(super::fonts::display())
                .size(16)
        )
        .padding(iced::Padding {
            top: 3.0,
            right: 24.0,
            bottom: 3.0,
            left: 12.0
        })
        .style(super::styles::wordmark),
        grip(Fill, Fill, Command::Drag),
        super::appearance::accents(state),
        Space::new().width(10),
        super::appearance::picker(state),
        Space::new().width(8),
        theme_button(state)
    ]
    .padding([0, 8])
    .height(HEIGHT)
    .align_y(iced::Center)
    .into()
}

// Windows' native mouse drag loop waits for a mouse release. A touch release
// is a separate event and must not start that loop.
#[derive(Default)]
struct Gesture {
    click: Option<mouse::Click>,
    #[cfg(target_os = "windows")]
    touch: Option<(iced::touch::Finger, iced::Point)>,
}

struct Grip {
    width: Length,
    height: Length,
    command: Command,
}

fn grip(width: Length, height: Length, command: Command) -> Element<'static, Message> {
    Element::new(Grip {
        width,
        height,
        command,
    })
}

impl Widget<Message, Theme, Renderer> for Grip {
    fn size(&self) -> Size<Length> {
        Size::new(self.width, self.height)
    }
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<Gesture>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(Gesture::default())
    }
    fn layout(&mut self, _: &mut Tree, _: &Renderer, limits: &layout::Limits) -> layout::Node {
        layout::atomic(limits, self.width, self.height)
    }
    fn draw(
        &self,
        _: &Tree,
        _: &mut Renderer,
        _: &Theme,
        _: &renderer::Style,
        _: Layout<'_>,
        _: mouse::Cursor,
        _: &Rectangle,
    ) {
    }
    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _: &Renderer,
        _: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _: &Rectangle,
    ) {
        let gesture = tree.state.downcast_mut::<Gesture>();
        #[cfg(target_os = "windows")]
        if let Event::Touch(touch) = event {
            use iced::touch::Event::*;
            match *touch {
                FingerPressed { id, position }
                    if matches!(self.command, Command::Drag)
                        && layout.bounds().contains(position) =>
                {
                    gesture.touch = Some((id, position));
                }
                FingerMoved { id, position }
                    if gesture.touch.is_some_and(|(finger, _)| finger == id) =>
                {
                    if let Some((_, anchor)) = gesture.touch {
                        shell.publish(Message::TouchWindowDrag(position - anchor));
                    }
                }
                FingerLifted { id, .. } | FingerLost { id, .. }
                    if gesture.touch.is_some_and(|(finger, _)| finger == id) =>
                {
                    gesture.touch = None;
                }
                _ => return,
            }
            shell.capture_event();
            return;
        }
        let position = match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => cursor.position(),
            #[cfg(not(target_os = "windows"))]
            Event::Touch(iced::touch::Event::FingerPressed { position, .. }) => Some(*position),
            _ => None,
        };
        if let Some(point) = position.filter(|point| layout.bounds().contains(*point)) {
            let click = mouse::Click::new(point, mouse::Button::Left, gesture.click);
            gesture.click = Some(click);
            let command = if matches!(self.command, Command::Drag)
                && click.kind() == mouse::click::Kind::Double
            {
                Command::Maximize
            } else {
                self.command
            };
            shell.publish(Message::Window(command));
            shell.capture_event();
        }
    }
    #[cfg(not(target_os = "macos"))]
    fn mouse_interaction(
        &self,
        _: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _: &Rectangle,
        _: &Renderer,
    ) -> mouse::Interaction {
        use window::Direction::*;
        if !cursor.is_over(layout.bounds()) {
            return mouse::Interaction::None;
        }
        match self.command {
            Command::Resize(East | West) => mouse::Interaction::ResizingHorizontally,
            Command::Resize(North | South) => mouse::Interaction::ResizingVertically,
            Command::Resize(NorthEast | SouthWest) => mouse::Interaction::ResizingDiagonallyUp,
            Command::Resize(NorthWest | SouthEast) => mouse::Interaction::ResizingDiagonallyDown,
            _ => mouse::Interaction::None,
        }
    }
}

#[cfg(not(target_os = "macos"))]
pub(super) fn resizable(body: Element<'_, Message>) -> Element<'_, Message> {
    use iced::widget::column;
    use window::Direction::*;
    let edge = Length::Fixed(4.0);
    let corner = Length::Fixed(12.0);
    let strip = |left, middle, right| {
        row![
            grip(corner, edge, Command::Resize(left)),
            grip(Fill, edge, Command::Resize(middle)),
            grip(corner, edge, Command::Resize(right)),
        ]
    };
    column![
        strip(NorthWest, North, NorthEast),
        row![
            grip(edge, Fill, Command::Resize(West)),
            body,
            grip(edge, Fill, Command::Resize(East))
        ]
        .height(Fill),
        strip(SouthWest, South, SouthEast),
    ]
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gui::actions::Action;
    use iced::advanced::{clipboard, mouse::Button};
    use iced::{Font, Pixels, Point};

    #[test]
    fn theme_button_and_drag_area_route_clicks_separately() {
        let mut state = Runtime::empty();
        state.app = Some(crate::app::test_support::fresh_app());
        let mut bar = view(&state);
        let widget = bar.as_widget_mut();
        let mut tree = Tree::new(&*widget);
        let renderer = Renderer::new(Font::default(), Pixels(16.0));
        let viewport = Rectangle::with_size(Size::new(760.0, HEIGHT));
        let node = widget.layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::ZERO, viewport.size()),
        );
        let mut click = |x, y| {
            let mut messages = Vec::new();
            for event in [
                mouse::Event::ButtonPressed(Button::Left),
                mouse::Event::ButtonReleased(Button::Left),
            ] {
                widget.update(
                    &mut tree,
                    &Event::Mouse(event),
                    Layout::new(&node),
                    mouse::Cursor::Available(Point::new(x, y)),
                    &renderer,
                    &mut clipboard::Null,
                    &mut Shell::new(&mut messages),
                    &viewport,
                );
            }
            messages
        };
        assert!(click(734.0, 7.0).is_empty());
        #[cfg(not(target_os = "macos"))]
        {
            assert!(matches!(
                click(24.0, HEIGHT / 2.0).as_slice(),
                [Message::Action(Action::RequestClose)]
            ));
            assert!(matches!(
                click(56.0, HEIGHT / 2.0).as_slice(),
                [Message::Window(Command::Minimize)]
            ));
            assert!(matches!(
                click(88.0, HEIGHT / 2.0).as_slice(),
                [Message::Window(Command::Maximize)]
            ));
        }
        assert!(matches!(
            click(734.0, HEIGHT / 2.0).as_slice(),
            [Message::Action(Action::ToggleTheme)]
        ));
        assert!(matches!(
            click(380.0, HEIGHT / 2.0).as_slice(),
            [Message::Window(Command::Drag)]
        ));
        assert!(matches!(
            click(380.0, HEIGHT / 2.0).as_slice(),
            [Message::Window(Command::Maximize)]
        ));
    }
}
