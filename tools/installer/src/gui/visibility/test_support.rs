use crate::app::{App, HiddenVisibility, test_support::fresh_app};
use crate::gui::runtime::Runtime;
use crate::tree::test_support::make_component;
use iced::advanced::widget::{Operation, Tree};
use iced::advanced::{Layout, Shell, clipboard, layout, mouse};
use iced::{Event, Font, Pixels, Point, Rectangle, Renderer, Size, Vector};
use std::collections::BTreeSet;
use std::path::PathBuf;

pub(super) struct Desktop {
    pub(super) state: Runtime,
    tree: Tree,
    renderer: Renderer,
    directory: PathBuf,
}

impl Drop for Desktop {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.directory).unwrap();
    }
}

#[derive(Default)]
pub(super) struct Labels(pub(super) Vec<(String, Rectangle)>);

impl Operation for Labels {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn text(&mut self, _: Option<&iced::widget::Id>, bounds: Rectangle, text: &str) {
        self.0.push((text.into(), bounds));
    }

    fn container(&mut self, id: Option<&iced::widget::Id>, bounds: Rectangle) {
        for (name, label) in [
            ("filters-menu", "[filters-menu]"),
            ("visibility-visible", "Visible only"),
            ("visibility-include", "Include hidden"),
            ("visibility-only", "Hidden only"),
        ] {
            if id == Some(&iced::widget::Id::new(name)) {
                self.0.push((label.into(), bounds));
            }
        }
    }
}

impl Desktop {
    pub(super) fn new() -> Self {
        let directory = std::env::temp_dir().join(format!(
            "hibi-bulk-visibility-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&directory).unwrap();
        let mut app = fresh_app();
        app.dest_dir = directory.join(".codex");
        app.visibility_path = directory.join("visibility.json");
        let hidden = BTreeSet::from([
            app.dest_dir.join("skills/hidden"),
            app.dest_dir.join("skills/other-hidden"),
        ]);
        crate::fs::installer::visibility::save(&app.visibility_path, &BTreeSet::new(), &hidden)
            .unwrap();
        let mut components = [
            "visible/a.md",
            "hidden/a.md",
            "other-hidden/a.md",
            "unchecked/a.md",
        ]
        .map(make_component);
        components[3].selected = false;
        app.hidden_visibility = HiddenVisibility::Include;
        app.finish_loading(components.into(), vec![], vec![], vec![]);
        let mut state = Runtime::empty();
        state.app = Some(app);
        let tree = Tree::new(crate::gui::view(&state).as_widget());
        Self {
            state,
            tree,
            renderer: Renderer::new(Font::default(), Pixels(16.0)),
            directory,
        }
    }

    pub(super) fn app(&self) -> &App {
        self.state.app.as_ref().unwrap()
    }

    pub(super) fn saved(&self) -> BTreeSet<PathBuf> {
        crate::fs::installer::visibility::load(&self.app().visibility_path).unwrap()
    }

    pub(super) fn labels(&mut self) -> Labels {
        let mut root = crate::gui::view(&self.state);
        self.tree.diff(root.as_widget());
        let node = root.as_widget_mut().layout(
            &mut self.tree,
            &self.renderer,
            &layout::Limits::new(Size::ZERO, self.state.window_size),
        );
        let mut labels = Labels::default();
        root.as_widget_mut().operate(
            &mut self.tree,
            Layout::new(&node),
            &self.renderer,
            &mut labels,
        );
        if let Some(mut overlay) = root.as_widget_mut().overlay(
            &mut self.tree,
            Layout::new(&node),
            &self.renderer,
            &Rectangle::new(Point::ORIGIN, self.state.window_size),
            Vector::ZERO,
        ) {
            let node = overlay
                .as_overlay_mut()
                .layout(&self.renderer, self.state.window_size);
            overlay
                .as_overlay_mut()
                .operate(Layout::new(&node), &self.renderer, &mut labels);
        }
        labels
    }

    fn event(&mut self, event: Event, position: Point) {
        let mut root = crate::gui::view(&self.state);
        self.tree.diff(root.as_widget());
        let node = root.as_widget_mut().layout(
            &mut self.tree,
            &self.renderer,
            &layout::Limits::new(Size::ZERO, self.state.window_size),
        );
        let mut messages = Vec::new();
        let mut shell = Shell::new(&mut messages);
        if let Some(mut overlay) = root.as_widget_mut().overlay(
            &mut self.tree,
            Layout::new(&node),
            &self.renderer,
            &Rectangle::new(Point::ORIGIN, self.state.window_size),
            Vector::ZERO,
        ) {
            let node = overlay
                .as_overlay_mut()
                .layout(&self.renderer, self.state.window_size);
            overlay.as_overlay_mut().update(
                &event,
                Layout::new(&node),
                mouse::Cursor::Available(position),
                &self.renderer,
                &mut clipboard::Null,
                &mut shell,
            );
        }
        if shell.event_status() == iced::event::Status::Ignored {
            root.as_widget_mut().update(
                &mut self.tree,
                &event,
                Layout::new(&node),
                mouse::Cursor::Available(position),
                &self.renderer,
                &mut clipboard::Null,
                &mut shell,
                &Rectangle::new(Point::ORIGIN, self.state.window_size),
            );
        }
        drop(root);
        for message in messages {
            let _ = crate::gui::update(&mut self.state, message);
        }
    }

    pub(super) fn click(&mut self, label: &str) {
        let labels = self.labels();
        let bounds = labels
            .0
            .iter()
            .find(|(text, _)| text == label)
            .unwrap_or_else(|| panic!("Missing button {label:?}: {:?}", labels.0))
            .1;
        let position = bounds.center();
        self.event(
            Event::Mouse(mouse::Event::CursorMoved { position }),
            position,
        );
        self.event(
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
            position,
        );
        self.event(
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
            position,
        );
    }

    pub(super) fn filters_label(&mut self) -> String {
        self.labels()
            .0
            .into_iter()
            .find(|(label, _)| label.starts_with("Filters · "))
            .unwrap()
            .0
    }

    pub(super) fn open_filters(&mut self) {
        if !self.state.filter_menu_open {
            let label = self.filters_label();
            self.click(&label);
        }
    }

    pub(super) fn close_filters(&mut self) {
        if self.state.filter_menu_open {
            let label = self.filters_label();
            self.click(&label);
        }
    }

    pub(super) fn choose_visibility(&mut self, label: &str) {
        self.open_filters();
        self.click(label);
        self.close_filters();
    }
}
