use iced::advanced::widget::{Operation, Tree, tree};
use iced::advanced::{Clipboard, Layout, Shell, Widget, layout, mouse, overlay, renderer};
use iced::{Element, Event, Length, Point, Rectangle, Renderer, Size, Theme, Vector};

use crate::gui::{Message, actions::Action};

#[cfg(test)]
mod tests;

pub(super) fn view<'a>(
    anchor: Element<'a, Message>,
    menu: Element<'a, Message>,
    open: bool,
) -> Element<'a, Message> {
    Element::new(Dropdown { anchor, menu, open })
}

struct Dropdown<'a> {
    anchor: Element<'a, Message>,
    menu: Element<'a, Message>,
    open: bool,
}

struct State;

impl Widget<Message, Theme, Renderer> for Dropdown<'_> {
    fn size(&self) -> Size<Length> {
        self.anchor.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.anchor.as_widget().size_hint()
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State)
    }

    fn children(&self) -> Vec<Tree> {
        vec![
            Tree::new(self.anchor.as_widget()),
            Tree::new(self.menu.as_widget()),
        ]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.children[0].diff(self.anchor.as_widget());
        tree.children[1].diff(self.menu.as_widget());
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.anchor
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.anchor.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        self.anchor
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.anchor.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.anchor.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn overlay<'a>(
        &'a mut self,
        tree: &'a mut Tree,
        layout: Layout<'a>,
        _: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'a, Message, Theme, Renderer>> {
        if !self.open {
            return None;
        }
        let anchor = Rectangle::new(layout.position() + translation, layout.bounds().size());
        Some(overlay::Element::new(Box::new(Menu {
            content: &mut self.menu,
            tree: &mut tree.children[1],
            anchor,
            viewport: *viewport,
        })))
    }
}

struct Menu<'a, 'b> {
    content: &'a mut Element<'b, Message>,
    tree: &'a mut Tree,
    anchor: Rectangle,
    viewport: Rectangle,
}

impl overlay::Overlay<Message, Theme, Renderer> for Menu<'_, '_> {
    fn layout(&mut self, renderer: &Renderer, bounds: Size) -> layout::Node {
        const GAP: f32 = 4.0;
        let below = (bounds.height - self.anchor.y - self.anchor.height - GAP).max(0.0);
        let above = (self.anchor.y - GAP).max(0.0);
        let width = match self.content.as_widget().size().width {
            Length::Fixed(width) => width,
            _ => self.anchor.width,
        }
        .min(bounds.width);
        let node = self.content.as_widget_mut().layout(
            self.tree,
            renderer,
            &layout::Limits::new(Size::ZERO, Size::new(width, below.max(above))).width(width),
        );
        let x = if self.anchor.x + node.size().width > bounds.width {
            self.anchor.x + self.anchor.width - node.size().width
        } else {
            self.anchor.x
        }
        .clamp(0.0, (bounds.width - node.size().width).max(0.0));
        let y = if node.size().height <= below {
            self.anchor.y + self.anchor.height + GAP
        } else {
            (self.anchor.y - node.size().height - GAP).max(0.0)
        };
        node.move_to(Point::new(x, y))
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        self.content.as_widget().draw(
            self.tree,
            renderer,
            theme,
            style,
            layout,
            cursor,
            &self.viewport,
        );
    }

    fn operate(&mut self, layout: Layout<'_>, renderer: &Renderer, operation: &mut dyn Operation) {
        self.content
            .as_widget_mut()
            .operate(self.tree, layout, renderer, operation);
    }

    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) {
        let pressed = matches!(
            event,
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                | Event::Touch(iced::touch::Event::FingerPressed { .. })
        );
        let position = match event {
            Event::Touch(iced::touch::Event::FingerPressed { position, .. }) => Some(*position),
            _ => cursor.position(),
        };
        if pressed
            && position.is_some_and(|position| {
                !layout.bounds().contains(position) && !self.anchor.contains(position)
            })
        {
            shell.publish(Message::Action(Action::CloseFilterMenu));
            shell.capture_event();
            return;
        }
        self.content.as_widget_mut().update(
            self.tree,
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            &self.viewport,
        );
        if cursor.is_over(layout.bounds()) && matches!(event, Event::Mouse(_) | Event::Touch(_)) {
            shell.capture_event();
        }
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            self.tree,
            layout,
            cursor,
            &self.viewport,
            renderer,
        )
    }
}
