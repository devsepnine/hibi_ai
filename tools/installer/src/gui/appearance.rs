use std::sync::LazyLock;

use iced::widget::{Space, container, pick_list, row, text, tooltip};
use iced::{Color, Element, Theme, color};

use super::{Message, actions::Action, runtime::Runtime, styles};
use crate::fs::installer::preferences::{Accent, Appearance};

static PALETTES: LazyLock<Vec<(String, Theme)>> = LazyLock::new(|| {
    Theme::ALL
        .iter()
        .map(|theme| (theme.to_string(), styles::glass_theme(theme)))
        .collect()
});

impl Accent {
    const ALL: [Self; 6] = [
        Self::Blue,
        Self::Lavender,
        Self::Mint,
        Self::Peach,
        Self::Rose,
        Self::Red,
    ];

    fn swatch(self, dark: bool) -> (&'static str, Color) {
        let (name, light, night) = match self {
            Self::Blue => ("Blue", color!(0x166284), color!(0x80cede)),
            Self::Lavender => ("Lavender", color!(0x6b518f), color!(0xb6a5d4)),
            Self::Mint => ("Mint", color!(0x166c60), color!(0x8bc6b7)),
            Self::Peach => ("Peach", color!(0x93552e), color!(0xd7ab83)),
            Self::Rose => ("Rose", color!(0x974563), color!(0xd3a0b5)),
            Self::Red => ("Red", color!(0xa33229), color!(0xed8978)),
        };
        (name, if dark { night } else { light })
    }
}

static WORKSPACE_THEMES: LazyLock<[[Theme; 6]; 2]> = LazyLock::new(|| {
    [false, true].map(|dark| {
        Accent::ALL.map(|accent| {
            let (name, bg, ink, base) = if dark {
                (
                    "Dark",
                    color!(0x201f2a),
                    color!(0xe1dce0),
                    Theme::CatppuccinMocha,
                )
            } else {
                (
                    "Light",
                    color!(0xedeae2),
                    color!(0x363341),
                    Theme::CatppuccinLatte,
                )
            };
            let palette = iced::theme::Palette {
                background: bg,
                text: ink,
                primary: accent.swatch(dark).1,
                ..base.palette()
            };
            styles::glass_theme(&Theme::custom(name, palette))
        })
    })
});

fn workspace_theme(dark: bool, accent: Accent) -> Theme {
    WORKSPACE_THEMES[usize::from(dark)][accent as usize].clone()
}

#[cfg(test)]
pub(super) fn workspace_palettes() -> impl Iterator<Item = Theme> {
    WORKSPACE_THEMES.iter().flatten().cloned()
}

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

pub(super) fn window_theme(choice: &Appearance, accent: Accent) -> Option<Theme> {
    match palette_name(choice)? {
        "Light" => Some(workspace_theme(false, accent)),
        "Dark" => Some(workspace_theme(true, accent)),
        name => named_theme(name),
    }
}

pub(super) fn resolve(choice: &Appearance, system: iced::theme::Mode, accent: Accent) -> Theme {
    window_theme(choice, accent)
        .unwrap_or_else(|| workspace_theme(system == iced::theme::Mode::Dark, accent))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AccentOption {
    value: Accent,
    label: &'static str,
}

impl std::fmt::Display for AccentOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label)
    }
}

pub(super) fn accents(state: &Runtime) -> Element<'static, Message> {
    if matches!(state.theme_choice, Appearance::Palette(ref name) if named_theme(name).is_some() && name != "Light" && name != "Dark")
    {
        return Space::new().into();
    }
    let dark = super::theme(state).extended_palette().is_dark;
    let options = Accent::ALL.map(|value| AccentOption {
        value,
        label: state.language.text(value.swatch(dark).0),
    });
    let selected = options[state.accent as usize];
    let swatch = state.accent.swatch(dark).1;
    let dot = container(Space::new())
        .width(10)
        .height(10)
        .style(move |_| {
            container::Style::default()
                .background(swatch)
                .border(iced::border::rounded(5))
        });
    let picker = pick_list(options, Some(selected), |choice| {
        Message::Action(Action::SetAccent(choice.value))
    })
    .text_size(12)
    .padding([5, 10])
    .style(styles::selector)
    .menu_style(styles::selector_menu);
    tooltip(
        row![dot, picker].spacing(6).align_y(iced::Center),
        text(state.language.text("Accent color")).size(12),
        tooltip::Position::Bottom,
    )
    .style(styles::popup)
    .into()
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
    fn each_accent_changes_the_workspace_highlight_in_both_modes() {
        for mode in [Mode::Light, Mode::Dark] {
            let mut colors = Vec::new();
            for accent in Accent::ALL {
                let theme = resolve(&Appearance::System, mode, accent);
                let primary = theme.palette().primary;
                assert!(!colors.contains(&primary));
                colors.push(primary);
                assert_eq!(theme.extended_palette().is_dark, mode == Mode::Dark);
                let nord = Appearance::Palette("Nord".into());
                assert_eq!(resolve(&nord, mode, accent), named_theme("Nord").unwrap());
            }
        }
    }

    #[test]
    fn saved_brightness_and_named_palettes_match_the_system_defaults() {
        for (choice, name, mode) in [
            (Appearance::Light, "Light", Mode::Light),
            (Appearance::Dark, "Dark", Mode::Dark),
        ] {
            let named = Appearance::Palette(name.to_owned());
            assert_eq!(palette_name(&choice), palette_name(&named));
            assert_eq!(
                resolve(&Appearance::System, mode, Accent::Blue),
                window_theme(&named, Accent::Blue).unwrap()
            );
            assert_eq!(
                window_theme(&choice, Accent::Blue).unwrap().to_string(),
                name
            );
        }
    }
}
