use super::{Message, actions::Action, runtime::Runtime};
use crate::app::{App, View};
use crossterm::event::KeyCode;
use iced::{Subscription, Task};
use std::time::Duration;

pub(super) fn scroll_after_key(app: &App, key: KeyCode) -> Task<Message> {
    use iced::widget::operation::{self, AbsoluteOffset, RelativeOffset};
    match app.current_view {
        View::List
            if matches!(
                key,
                KeyCode::Up
                    | KeyCode::Down
                    | KeyCode::Left
                    | KeyCode::Right
                    | KeyCode::Home
                    | KeyCode::Char('j' | 'k' | 'h' | 'l' | 'E' | 'f' | 'H')
            ) =>
        {
            let (index, count) = match app.tab {
                crate::app::Tab::McpServers => (
                    app.visible_mcp_servers()
                        .iter()
                        .position(|(index, _)| *index == app.mcp_index)
                        .unwrap_or(0),
                    app.visible_mcp_servers().len(),
                ),
                crate::app::Tab::Plugins => (
                    app.visible_plugins()
                        .iter()
                        .position(|(index, _)| *index == app.plugin_index)
                        .unwrap_or(0),
                    app.visible_plugins().len(),
                ),
                _ => app
                    .tree_views
                    .get(&app.tab)
                    .map(|tree| (tree.cursor, tree.visible_indices.len()))
                    .unwrap_or_default(),
            };
            let y = if count > 1 {
                index as f32 / (count - 1) as f32
            } else {
                0.0
            };
            operation::snap_to("component-list", RelativeOffset { x: 0.0, y })
        }
        View::Diff | View::Help
            if matches!(key, KeyCode::Up | KeyCode::Down | KeyCode::Char('j' | 'k')) =>
        {
            let y = if matches!(key, KeyCode::Up | KeyCode::Char('k')) {
                -32.0
            } else {
                32.0
            };
            operation::scroll_by(
                if app.current_view == View::Diff {
                    "diff"
                } else {
                    "help"
                },
                AbsoluteOffset { x: 0.0, y },
            )
        }
        _ => Task::none(),
    }
}

pub(super) fn subscription(state: &Runtime) -> Subscription<Message> {
    let events = iced::event::listen_with(|event, status, _| {
        if matches!(
            event,
            iced::Event::Window(iced::window::Event::CloseRequested)
        ) {
            return Some(Message::Action(Action::RequestClose));
        }
        if let iced::Event::Window(iced::window::Event::Resized(size)) = event {
            return Some(Message::Resized(size));
        }
        if let iced::Event::Keyboard(iced::keyboard::Event::KeyPressed { key, modifiers, .. }) =
            &event
        {
            if modifiers.command()
                && matches!(key, iced::keyboard::Key::Character(value) if value.eq_ignore_ascii_case("f"))
            {
                return Some(Message::FocusSearch);
            }
            if status == iced::event::Status::Captured
                && *key == iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape)
            {
                return Some(Message::SearchEscape);
            }
        }
        if status == iced::event::Status::Captured {
            return None;
        }
        match event {
            iced::Event::Keyboard(iced::keyboard::Event::KeyPressed { key, modifiers, .. }) => {
                keyboard_key(key, modifiers).map(|key| Message::Action(Action::Key(key)))
            }
            _ => None,
        }
    });
    if state.busy() {
        Subscription::batch([
            events,
            iced::time::every(Duration::from_millis(50)).map(|_| Message::Tick),
        ])
    } else {
        events
    }
}

fn keyboard_key(key: iced::keyboard::Key, modifiers: iced::keyboard::Modifiers) -> Option<KeyCode> {
    use iced::keyboard::{Key, key::Named};
    if modifiers.control() || modifiers.logo() || modifiers.alt() {
        return None;
    }
    Some(match key {
        Key::Named(Named::Escape) => KeyCode::Esc,
        Key::Named(Named::Enter) => KeyCode::Enter,
        Key::Named(Named::ArrowUp) => KeyCode::Up,
        Key::Named(Named::ArrowDown) => KeyCode::Down,
        Key::Named(Named::ArrowLeft) => KeyCode::Left,
        Key::Named(Named::ArrowRight) => KeyCode::Right,
        Key::Named(Named::Space) => KeyCode::Char(' '),
        Key::Named(Named::Tab) if modifiers.shift() => KeyCode::BackTab,
        Key::Named(Named::Tab) => KeyCode::Tab,
        Key::Character(value) if value.chars().count() == 1 => KeyCode::Char(value.chars().next()?),
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shortcuts_retain_case_and_do_not_turn_multi_character_input_into_commands() {
        use iced::keyboard::{Key, Modifiers};
        assert_eq!(
            keyboard_key(Key::Character("X".into()), Modifiers::SHIFT),
            Some(KeyCode::Char('X'))
        );
        assert_eq!(
            keyboard_key(Key::Character("한글".into()), Modifiers::NONE),
            None
        );
        assert_eq!(keyboard_key(Key::Unidentified, Modifiers::NONE), None);
    }

    #[test]
    fn editing_shortcuts_never_become_list_actions_when_the_input_is_unfocused() {
        use iced::keyboard::{Key, Modifiers};
        for modifiers in [Modifiers::CTRL, Modifiers::LOGO, Modifiers::ALT] {
            for letter in ["a", "c", "x", "v", "i", "r"] {
                assert_eq!(keyboard_key(Key::Character(letter.into()), modifiers), None);
            }
        }
    }
}
