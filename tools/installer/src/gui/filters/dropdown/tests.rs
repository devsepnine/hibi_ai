use super::*;
use iced::advanced::renderer::{Headless, Renderer as _};
use iced::widget::{Space, container};
use iced::{Color, Font, Pixels};

#[test]
fn recreated_overlay_draws_with_the_cached_layout() {
    let mut renderer = Renderer::new(Font::default(), Pixels(16.0));
    let viewport = Rectangle::with_size(Size::new(400.0, 300.0));
    let mut widget = view(
        Space::new().width(180).height(36).into(),
        container(Space::new().width(260).height(160))
            .width(260)
            .style(|_| container::Style::default().background(Color::from_rgb(1.0, 0.0, 0.0)))
            .into(),
        true,
    );
    let mut tree = Tree::new(widget.as_widget());
    let node = widget.as_widget_mut().layout(
        &mut tree,
        &renderer,
        &layout::Limits::new(Size::ZERO, viewport.size()),
    );
    let overlay_layout = {
        let mut overlay = widget
            .as_widget_mut()
            .overlay(
                &mut tree,
                Layout::new(&node),
                &renderer,
                &viewport,
                Vector::ZERO,
            )
            .unwrap();
        overlay.as_overlay_mut().layout(&renderer, viewport.size())
    };
    renderer.reset(viewport);
    let overlay = widget
        .as_widget_mut()
        .overlay(
            &mut tree,
            Layout::new(&node),
            &renderer,
            &viewport,
            Vector::ZERO,
        )
        .unwrap();
    overlay.as_overlay().draw(
        &mut renderer,
        &Theme::Dark,
        &renderer::Style {
            text_color: Color::WHITE,
        },
        Layout::new(&overlay_layout),
        mouse::Cursor::Unavailable,
    );
    let pixels = renderer.screenshot(Size::new(400, 300), 1.0, Color::BLACK);
    let red = pixels
        .chunks_exact(4)
        .filter(|pixel| pixel[0] > 240 && pixel[1] < 15 && pixel[2] < 15)
        .count();
    let expected = (overlay_layout.size().width * overlay_layout.size().height) as usize;
    assert_eq!(red, expected, "The entire menu background must be visible");
}
