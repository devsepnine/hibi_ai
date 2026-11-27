use std::sync::LazyLock;

use iced::widget::pick_list;
use iced::{Element, Theme};

use super::{Message, actions::Action, runtime::Runtime, styles};
use crate::fs::installer::preferences::Appearance;

static PALETTES: LazyLock<Vec<(String, Theme)>> = LazyLock::new(|| {
    Theme::ALL
        .iter()
        .map(|theme| (theme.to_string(), styles::glass_theme(theme)))
        .collect()
});

fn palette_name(choice: &Appearance) -> Option<&str> {
    match choice {
        Appearance::System => None,
        Appearance::Light => Some("Light"),
        Appearance::Dark => Some("Dark"),
        Appearance::Palette(name) => Some(name),
    }
}

fn named_theme(name: &str) -> Option<Theme> {
    PALETTES
        .iter()
        .find(|(label, _)| label == name)
        .map(|(_, theme)| theme.clone())
}

pub(super) fn window_theme(choice: &Appearance) -> Option<Theme> {
    palette_name(choice).and_then(named_theme)
}

pub(super) fn resolve(choice: &Appearance, system: iced::theme::Mode) -> Theme {
    window_theme(choice).unwrap_or_else(|| {
        let name = if system == iced::theme::Mode::Dark {
            "Dark"
        } else {
            "Light"
        };
        named_theme(name).expect("iced includes the Light and Dark palettes")
    })
}

pub(super) fn picker(state: &Runtime) -> Element<'static, Message> {
    let system = state.language.text("System theme");
    let options: Vec<_> = std::iter::once(system.to_owned())
        .chain(PALETTES.iter().map(|(name, _)| name.clone()))
        .collect();
    let selected = palette_name(&state.theme_choice)
        .and_then(|name| options.iter().find(|option| option.as_str() == name))
        .cloned()
        .unwrap_or_else(|| system.to_owned());
    pick_list(options, Some(selected), move |label: String| {
        let choice = match label.as_str() {
            value if value == system => Appearance::System,
            "Light" => Appearance::Light,
            "Dark" => Appearance::Dark,
            _ => Appearance::Palette(label),
        };
        Message::Action(Action::SetAppearance(choice))
    })
    .text_size(12)
    .padding([5, 10])
    .style(styles::selector)
    .menu_style(styles::selector_menu)
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::theme::Mode;

    #[test]
    fn saved_brightness_and_named_palettes_match_the_system_defaults() {
        for (choice, name, mode) in [
            (Appearance::Light, "Light", Mode::Light),
            (Appearance::Dark, "Dark", Mode::Dark),
        ] {
            let named = Appearance::Palette(name.to_owned());
            assert_eq!(palette_name(&choice), palette_name(&named));
            assert_eq!(
                resolve(&Appearance::System, mode),
                window_theme(&named).unwrap()
            );
            assert_eq!(window_theme(&choice).unwrap().to_string(), name);
        }
    }
}
