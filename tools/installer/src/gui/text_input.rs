use iced::advanced::widget::{Operation, Tree, tree};
use iced::advanced::{Clipboard, Layout, Shell, Widget, layout, mouse, overlay, renderer};
use iced::keyboard::{self, Modifiers};
use iced::widget::TextInput;
use iced::{Element, Event, Length, Rectangle, Renderer, Size, Theme, Vector};

pub(super) fn search_input<'a, Message: Clone + 'a>(
    input: TextInput<'a, Message>,
    search_shortcut: bool,
) -> Element<'a, Message> {
    Element::new(SearchInput {
        input,
        search_shortcut,
    })
}

type InputState =
    iced::widget::text_input::State<<Renderer as iced::advanced::text::Renderer>::Paragraph>;

struct SearchInput<'a, Message> {
    input: TextInput<'a, Message>,
    search_shortcut: bool,
}

impl<Message: Clone> Widget<Message, Theme, Renderer> for SearchInput<'_, Message> {
    fn size(&self) -> Size<Length> {
        Widget::size(&self.input)
    }

    fn size_hint(&self) -> Size<Length> {
        Widget::size_hint(&self.input)
    }

    fn tag(&self) -> tree::Tag {
        Widget::tag(&self.input)
    }

    fn state(&self) -> tree::State {
        Widget::state(&self.input)
    }

    fn children(&self) -> Vec<Tree> {
        Widget::children(&self.input)
    }

    fn diff(&self, tree: &mut Tree) {
        Widget::diff(&self.input, tree);
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        Widget::layout(&mut self.input, tree, renderer, limits)
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
        Widget::draw(
            &self.input,
            tree,
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
        Widget::operate(&mut self.input, tree, layout, renderer, operation);
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
        if let Event::Keyboard(keyboard::Event::KeyPressed {
            key,
            physical_key,
            modifiers,
            ..
        }) = event
        {
            // Focus before further key events in the same batch reach the list.
            if self.search_shortcut
                && modifiers.command()
                && key.to_latin(*physical_key) == Some('f')
            {
                tree.state.downcast_mut::<InputState>().focus();
                shell.capture_event();
                shell.request_redraw();
                return;
            }
            if *key == keyboard::Key::Named(keyboard::key::Named::Tab)
                && !modifiers.intersects(Modifiers::CTRL | Modifiers::LOGO | Modifiers::ALT)
            {
                let state = tree.state.downcast_mut::<InputState>();
                if state.is_focused() {
                    state.unfocus();
                    shell.request_redraw();
                }
            }
            // Native TextInput reads its modifier cache, which can be stale after focus changes.
            let editing_command =
                matches!(key.to_latin(*physical_key), Some('a' | 'c' | 'x' | 'v'));
            let modifiers =
                if editing_command && !modifiers.alt() && (modifiers.control() || modifiers.logo())
                {
                    modifiers.difference(Modifiers::CTRL | Modifiers::LOGO) | Modifiers::COMMAND
                } else {
                    *modifiers
                };
            Widget::update(
                &mut self.input,
                tree,
                &Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)),
                layout,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
        }
        Widget::update(
            &mut self.input,
            tree,
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
        if shell.event_status() == iced::event::Status::Captured
            && matches!(
                event,
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(keyboard::key::Named::Enter),
                    ..
                })
            )
        {
            // A deferred unfocus could undo a later Find shortcut in the same batch.
            tree.state.downcast_mut::<InputState>().unfocus();
            shell.request_redraw();
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        Widget::mouse_interaction(&self.input, tree, layout, cursor, viewport, renderer)
    }

    fn overlay<'a>(
        &'a mut self,
        tree: &'a mut Tree,
        layout: Layout<'a>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'a, Message, Theme, Renderer>> {
        Widget::overlay(
            &mut self.input,
            tree,
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::advanced::clipboard;
    use iced::keyboard::{Key, Location, key};
    use iced::widget::text_input::Value;
    use iced::{Font, Pixels, Point};

    #[derive(Default)]
    struct FakeClipboard(Option<String>);

    impl Clipboard for FakeClipboard {
        fn read(&self, kind: clipboard::Kind) -> Option<String> {
            (kind == clipboard::Kind::Standard)
                .then(|| self.0.clone())
                .flatten()
        }

        fn write(&mut self, kind: clipboard::Kind, value: String) {
            if kind == clipboard::Kind::Standard {
                self.0 = Some(value);
            }
        }
    }

    struct Harness {
        wrapped: bool,
        value: String,
        tree: Tree,
        renderer: Renderer,
        clipboard: FakeClipboard,
    }

    #[derive(Clone, Debug, PartialEq)]
    enum Edit {
        SetValue(String),
        Submit,
    }

    fn element(value: &str, wrapped: bool) -> Element<'static, Edit> {
        let input = TextInput::new("Search", value)
            .id("search-test")
            .on_input(Edit::SetValue)
            .on_submit(Edit::Submit);
        if wrapped {
            search_input(input, true)
        } else {
            input.into()
        }
    }

    impl Harness {
        fn new(value: &str, wrapped: bool) -> Self {
            let input = element(value, wrapped);
            let mut harness = Self {
                wrapped,
                value: value.into(),
                tree: Tree::new(input.as_widget()),
                renderer: Renderer::new(Font::default(), Pixels(16.0)),
                clipboard: FakeClipboard::default(),
            };
            harness.focus();
            harness
        }

        fn focus(&mut self) {
            let mut input = element(&self.value, self.wrapped);
            self.tree.diff(input.as_widget());
            let node = input.as_widget_mut().layout(
                &mut self.tree,
                &self.renderer,
                &layout::Limits::new(Size::ZERO, Size::new(400.0, 80.0)),
            );
            let mut focus =
                iced::advanced::widget::operation::focusable::focus::<()>("search-test".into());
            input.as_widget_mut().operate(
                &mut self.tree,
                Layout::new(&node),
                &self.renderer,
                &mut focus,
            );
        }

        fn send(&mut self, event: Event) -> (iced::event::Status, Vec<Edit>) {
            let mut input = element(&self.value, self.wrapped);
            self.tree.diff(input.as_widget());
            let node = input.as_widget_mut().layout(
                &mut self.tree,
                &self.renderer,
                &layout::Limits::new(Size::ZERO, Size::new(400.0, 80.0)),
            );
            let mut messages = Vec::new();
            let mut shell = Shell::new(&mut messages);
            input.as_widget_mut().update(
                &mut self.tree,
                &event,
                Layout::new(&node),
                mouse::Cursor::Unavailable,
                &self.renderer,
                &mut self.clipboard,
                &mut shell,
                &Rectangle::new(Point::ORIGIN, Size::new(400.0, 80.0)),
            );
            let status = shell.event_status();
            if let Some(Edit::SetValue(value)) = messages.last() {
                self.value = value.clone();
            }
            (status, messages)
        }

        fn selection(&self) -> Option<(usize, usize)> {
            self.tree
                .state
                .downcast_ref::<InputState>()
                .cursor()
                .selection(&Value::new(&self.value))
        }
    }

    fn press(character: &str, modifiers: Modifiers) -> Event {
        let code = match character {
            "a" => key::Code::KeyA,
            "c" => key::Code::KeyC,
            "x" => key::Code::KeyX,
            "v" => key::Code::KeyV,
            "f" => key::Code::KeyF,
            "r" => key::Code::KeyR,
            _ => key::Code::KeyB,
        };
        Event::Keyboard(keyboard::Event::KeyPressed {
            key: Key::Character(character.into()),
            modified_key: Key::Character(character.into()),
            physical_key: key::Physical::Code(code),
            location: Location::Standard,
            modifiers,
            text: Some(character.into()),
            repeat: false,
        })
    }

    fn escape(modifiers: Modifiers) -> Event {
        named_key(key::Named::Escape, key::Code::Escape, modifiers)
    }

    fn named_key(name: key::Named, code: key::Code, modifiers: Modifiers) -> Event {
        Event::Keyboard(keyboard::Event::KeyPressed {
            key: Key::Named(name),
            modified_key: Key::Named(name),
            physical_key: key::Physical::Code(code),
            location: Location::Standard,
            modifiers,
            text: None,
            repeat: false,
        })
    }

    #[test]
    fn baseline_native_input_loses_held_command_after_escape_and_refocus() {
        let mut native = Harness::new("architect", false);
        native.send(Event::Keyboard(keyboard::Event::ModifiersChanged(
            Modifiers::COMMAND,
        )));
        native.send(escape(Modifiers::COMMAND));
        native.focus();
        native.send(press("a", Modifiers::COMMAND));
        assert_eq!(native.selection(), None);
        assert_eq!(native.value, "architecta");
    }

    #[test]
    fn held_ctrl_or_logo_selects_whole_query_after_escape_refocus_and_native_rebuild() {
        for modifier in [Modifiers::CTRL, Modifiers::LOGO] {
            let mut input = Harness::new("architect 한국어", true);
            input.send(Event::Keyboard(keyboard::Event::ModifiersChanged(modifier)));
            input.send(escape(modifier));
            input.focus();
            let (status, messages) = input.send(press("a", modifier));
            assert_eq!(status, iced::event::Status::Captured);
            assert!(messages.is_empty());
            assert_eq!(input.value, "architect 한국어");
            assert_eq!(input.selection(), Some((0, Value::new(&input.value).len())));
            input.send(Event::InputMethod(
                iced::advanced::input_method::Event::Commit("검토".into()),
            ));
            assert_eq!(input.value, "검토");
        }
    }

    #[cfg(not(target_os = "macos"))]
    #[test]
    fn baseline_native_logo_a_inserts_literal_a_while_wrapper_selects_without_extra_text() {
        let mut native = Harness::new("architect", false);
        native.send(Event::Keyboard(keyboard::Event::ModifiersChanged(
            Modifiers::LOGO,
        )));
        native.send(press("a", Modifiers::LOGO));
        assert_eq!(native.value, "architecta");
        assert_eq!(native.selection(), None);
        let mut fixed = Harness::new("architect", true);
        fixed.send(press("a", Modifiers::LOGO));
        assert_eq!(fixed.value, "architect");
        assert_eq!(fixed.selection(), Some((0, 9)));
    }

    #[test]
    fn ctrl_and_logo_copy_cut_and_paste_use_native_clipboard_and_selection() {
        for modifier in [Modifiers::CTRL, Modifiers::LOGO] {
            let mut input = Harness::new("검색할 내용", true);
            input.send(press("a", modifier));
            let (_, messages) = input.send(press("c", modifier));
            assert!(messages.is_empty());
            assert_eq!(input.clipboard.0.as_deref(), Some("검색할 내용"));
            assert_eq!(input.value, "검색할 내용");
            input.send(press("x", modifier));
            assert!(input.value.is_empty());
            assert_eq!(input.clipboard.0.as_deref(), Some("검색할 내용"));
            input.send(press("v", modifier));
            assert_eq!(input.value, "검색할 내용");
        }
    }

    #[test]
    fn next_plain_a_resets_stale_command_cache_and_types_normally() {
        let mut input = Harness::new("architect", true);
        input.send(press("a", Modifiers::LOGO));
        input.send(press("a", Modifiers::NONE));
        assert_eq!(input.value, "a");
        input.send(press("b", Modifiers::NONE));
        assert_eq!(input.value, "ab");
    }

    #[test]
    fn find_shortcut_captures_the_next_typed_key_without_waiting_for_an_app_update() {
        let mut input = Harness::new("", true);
        input.send(escape(Modifiers::NONE));
        let (status, messages) = input.send(press("f", Modifiers::COMMAND));
        assert_eq!(status, iced::event::Status::Captured);
        assert!(messages.is_empty());
        let (status, _) = input.send(press("r", Modifiers::NONE));
        assert_eq!(status, iced::event::Status::Captured);
        assert_eq!(input.value, "r");
    }

    #[test]
    fn submitting_search_returns_the_next_key_to_results_without_a_deferred_focus_change() {
        let mut input = Harness::new("architect", true);
        let (status, messages) = input.send(named_key(
            key::Named::Enter,
            key::Code::Enter,
            Modifiers::NONE,
        ));
        assert_eq!(status, iced::event::Status::Captured);
        assert_eq!(messages, [Edit::Submit]);
        let (status, messages) = input.send(press("r", Modifiers::NONE));
        assert_eq!(status, iced::event::Status::Ignored);
        assert!(messages.is_empty());

        input.send(press("f", Modifiers::COMMAND));
        let (status, _) = input.send(press("r", Modifiers::NONE));
        assert_eq!(status, iced::event::Status::Captured);
        assert_eq!(input.value, "architectr");
    }

    #[test]
    fn tab_releases_the_search_before_the_pane_navigation_message() {
        let mut input = Harness::new("architect", true);
        let (status, messages) =
            input.send(named_key(key::Named::Tab, key::Code::Tab, Modifiers::SHIFT));
        assert_eq!(status, iced::event::Status::Ignored);
        assert!(messages.is_empty());
        let (status, messages) = input.send(press("r", Modifiers::NONE));
        assert_eq!(status, iced::event::Status::Ignored);
        assert!(messages.is_empty());
        assert_eq!(input.value, "architect");
    }

    #[test]
    fn unfocused_search_does_not_capture_editing_commands_or_change_clipboard() {
        let mut input = Harness::new("architect", true);
        input.send(escape(Modifiers::NONE));
        for character in ["a", "c", "x", "v"] {
            let (status, messages) = input.send(press(character, Modifiers::LOGO));
            assert_eq!(status, iced::event::Status::Ignored);
            assert!(messages.is_empty());
        }
        assert_eq!(input.value, "architect");
        assert!(input.clipboard.0.is_none());
    }

    #[test]
    fn alt_commands_and_other_logo_letters_keep_native_behavior() {
        let nonnative = if cfg!(target_os = "macos") {
            Modifiers::CTRL
        } else {
            Modifiers::LOGO
        };
        let mut input = Harness::new("query", true);
        input.send(press("a", nonnative | Modifiers::ALT));
        assert_eq!(input.selection(), None);
        assert_eq!(input.value, "querya");
        input.send(press("b", nonnative));
        assert_eq!(input.value, "queryab");
    }
}
