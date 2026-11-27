use iced::theme::palette::{mix, readable};
use iced::widget::{button, checkbox, container, overlay::menu, pick_list, rule, text, text_input};
use iced::{Border, Color, Degrees, Theme, gradient};

const PANEL_RADIUS: f32 = 18.0;
const CONTROL_RADIUS: f32 = 10.0;

pub(super) fn glass_theme(source: &Theme) -> Theme {
    let mut palette = source.palette();
    let dark = source.extended_palette().is_dark;
    let reference = mix(
        palette.background,
        if dark { Color::WHITE } else { Color::BLACK },
        if dark { 0.065 } else { 0.14 },
    );
    for ink in [
        &mut palette.text,
        &mut palette.primary,
        &mut palette.success,
        &mut palette.warning,
        &mut palette.danger,
    ] {
        *ink = readable(reference, *ink);
    }
    Theme::custom(source.to_string(), palette)
}

fn surface_color(theme: &Theme) -> Color {
    let base = theme.palette().background;
    if theme.extended_palette().is_dark {
        mix(base, Color::BLACK, 0.22)
    } else {
        mix(base, Color::WHITE, 0.70)
    }
}

fn glass_tint(theme: &Theme) -> Color {
    let p = theme.palette();
    let amount = if theme.extended_palette().is_dark {
        0.11
    } else {
        0.21
    };
    mix(p.background, p.primary, amount)
}

pub fn backdrop(theme: &Theme) -> container::Style {
    let p = theme.palette();
    let dark = theme.extended_palette().is_dark;
    let start = if dark {
        mix(p.background, p.primary, 0.075)
    } else {
        mix(p.background, Color::WHITE, 0.75)
    };
    let end = if dark {
        mix(p.background, Color::BLACK, 0.48)
    } else {
        glass_tint(theme)
    };
    container::Style::default().background(
        gradient::Linear::new(Degrees(126.0))
            .add_stop(0.0, start)
            .add_stop(0.48, p.background)
            .add_stop(1.0, end),
    )
}

pub fn divider(theme: &Theme) -> rule::Style {
    rule::Style {
        color: surface(theme).border.color,
        ..rule::default(theme)
    }
}

pub fn muted(theme: &Theme) -> text::Style {
    let palette = theme.palette();
    let (reference, background_weight) = if theme.extended_palette().is_dark {
        (mix(palette.background, palette.primary, 0.14), 0.24)
    } else {
        (theme.extended_palette().background.neutral.color, 0.06)
    };
    let ink = mix(palette.text, palette.background, background_weight);
    text::Style {
        color: Some(readable(reference, ink)),
    }
}

pub fn surface(theme: &Theme) -> container::Style {
    container::Style {
        background: Some(surface_color(theme).into()),
        border: Border {
            color: theme.extended_palette().background.weak.color,
            width: 1.0,
            radius: PANEL_RADIUS.into(),
        },
        ..Default::default()
    }
}

fn glass_fill(theme: &Theme, base: Color) -> gradient::Linear {
    let dark = theme.extended_palette().is_dark;
    let highlight = if dark {
        glass_tint(theme)
    } else {
        mix(base, Color::WHITE, 0.94)
    };
    gradient::Linear::new(Degrees(153.0))
        .add_stop(0.0, highlight.scale_alpha(0.90))
        .add_stop(0.36, base.scale_alpha(0.52))
        .add_stop(
            1.0,
            mix(
                base,
                theme.palette().primary,
                if dark { 0.065 } else { 0.15 },
            )
            .scale_alpha(0.78),
        )
}

pub fn glass(theme: &Theme) -> container::Style {
    let fill = glass_fill(theme, theme.palette().background);
    container::Style {
        background: Some(fill.into()),
        border: Border {
            color: mix(theme.palette().primary, Color::WHITE, 0.45).scale_alpha(0.55),
            width: 1.0,
            radius: PANEL_RADIUS.into(),
        },
        ..Default::default()
    }
}

pub fn popup(theme: &Theme) -> container::Style {
    let mut style = surface(theme);
    style.border.radius = CONTROL_RADIUS.into();
    style
}

pub fn subtle(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = button::text(theme, status);
    style.border = Border {
        radius: CONTROL_RADIUS.into(),
        ..Default::default()
    };
    style.text_color = theme.palette().text;
    style.background = match status {
        button::Status::Hovered => Some(hover_fill(theme, 0.65).into()),
        button::Status::Pressed => Some(hover_fill(theme, 1.0).into()),
        _ => None,
    };
    if status == button::Status::Disabled {
        style.text_color = muted(theme)
            .color
            .unwrap_or(theme.palette().text)
            .scale_alpha(0.55);
    }
    style
}

fn hover_fill(theme: &Theme, alpha: f32) -> Color {
    theme
        .extended_palette()
        .background
        .weak
        .color
        .scale_alpha(alpha)
}

fn control_border(theme: &Theme) -> Border {
    let tint = glass_tint(theme);
    let dark = theme.extended_palette().is_dark;
    let end = mix(
        theme.palette().primary,
        if dark { Color::WHITE } else { Color::BLACK },
        0.65,
    );
    // Leave room for antialiasing at the brightest dark or deepest light glass edge.
    let base = tint.relative_luminance();
    let target = if dark {
        (base + 0.05) * 3.6 - 0.05
    } else {
        (base + 0.05) / 3.6 - 0.05
    };
    let strength = (target - base) / (end.relative_luminance() - base);
    Border {
        color: mix(tint, end, strength),
        width: 1.5,
        radius: CONTROL_RADIUS.into(),
    }
}

pub fn secondary(theme: &Theme, status: button::Status) -> button::Style {
    let palette = &theme.extended_palette().background;
    let pair = match status {
        button::Status::Active | button::Status::Disabled => palette.weakest,
        button::Status::Hovered => palette.weak,
        button::Status::Pressed => palette.neutral,
    };
    let mut style = button::Style {
        background: Some(glass_fill(theme, pair.color).into()),
        text_color: pair.text,
        border: control_border(theme),
        ..Default::default()
    };
    if status == button::Status::Disabled {
        style.background = Some(pair.color.scale_alpha(0.4).into());
        style.text_color = style.text_color.scale_alpha(0.45);
        style.border.color = style.border.color.scale_alpha(0.4);
    }
    style
}

pub fn glass_button(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = secondary(theme, status);
    style.border.radius = PANEL_RADIUS.into();
    if status == button::Status::Active {
        style.background = glass(theme).background;
    }
    style
}

pub fn focused_glass_button(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = glass_button(theme, status);
    style.border = focus_border(theme, style.border.radius);
    style
}

pub fn focused_secondary(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = secondary(theme, status);
    style.border = focus_border(theme, style.border.radius);
    style
}

pub fn primary(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = button::primary(theme, status);
    style.border.radius = CONTROL_RADIUS.into();
    if status == button::Status::Hovered {
        style.text_color = theme.extended_palette().primary.strong.text;
    }
    style
}

pub fn destructive(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = button::danger(theme, status);
    style.border.radius = CONTROL_RADIUS.into();
    style
}

pub fn navigation(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = subtle(theme, status);
    let palette = theme.palette();
    style.background = Some(match status {
        button::Status::Hovered | button::Status::Pressed => hover_fill(theme, 1.0).into(),
        _ => mix(palette.background, palette.primary, 0.08).into(),
    });
    style.border = control_border(theme);
    style
}

pub fn focused_row(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = navigation(theme, status);
    style.border = focus_border(theme, style.border.radius);
    style
}

fn focus_border(theme: &Theme, radius: impl Into<iced::border::Radius>) -> Border {
    Border {
        color: theme.palette().primary,
        width: 2.0,
        radius: radius.into(),
    }
}

pub fn focus_ring(theme: &Theme) -> container::Style {
    container::Style {
        border: focus_border(theme, CONTROL_RADIUS),
        ..Default::default()
    }
}

pub fn input(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let mut style = text_input::default(theme, status);
    style.background = surface_color(theme).into();
    style.placeholder = muted(theme).color.unwrap_or(theme.palette().text);
    style.border = control_border(theme);
    if matches!(status, text_input::Status::Focused { .. }) {
        style.border = focus_border(theme, CONTROL_RADIUS);
    }
    style
}

pub fn selector(theme: &Theme, status: pick_list::Status) -> pick_list::Style {
    let mut style = pick_list::default(theme, status);
    style.background = if status == pick_list::Status::Active {
        surface_color(theme).into()
    } else {
        hover_fill(theme, 1.0).into()
    };
    style.border = control_border(theme);
    style
}

pub fn selector_menu(theme: &Theme) -> menu::Style {
    let mut style = menu::default(theme);
    style.border.radius = CONTROL_RADIUS.into();
    style
}

pub fn check(theme: &Theme, status: checkbox::Status) -> checkbox::Style {
    let mut style = checkbox::primary(theme, status);
    style.border.radius = 4.0.into();
    let checked = match status {
        checkbox::Status::Active { is_checked }
        | checkbox::Status::Hovered { is_checked }
        | checkbox::Status::Disabled { is_checked } => is_checked,
    };
    if checked {
        style.background = mix(theme.palette().background, theme.palette().primary, 0.10).into();
        style.border.color = theme.palette().primary.scale_alpha(0.6);
        style.icon_color = theme.palette().primary;
    }
    style
}

pub fn danger(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = subtle(theme, status);
    style.text_color = if status == button::Status::Disabled {
        muted(theme).color.unwrap_or(theme.palette().text)
    } else {
        theme.palette().danger
    };
    style
}

pub fn badge(theme: &Theme) -> container::Style {
    container::Style {
        background: Some(mix(theme.palette().background, theme.palette().primary, 0.05).into()),
        text_color: muted(theme).color,
        border: Border {
            radius: CONTROL_RADIUS.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::Background;

    #[test]
    fn compact_popups_and_focus_rings_match_their_controls() {
        let theme = glass_theme(&Theme::Light);
        let radius = secondary(&theme, button::Status::Active).border.radius;
        for border in [
            navigation(&theme, button::Status::Active).border,
            focus_ring(&theme).border,
            badge(&theme).border,
            popup(&theme).border,
            selector(&theme, pick_list::Status::Active).border,
            selector_menu(&theme).border,
        ] {
            assert_eq!(border.radius, radius);
        }
    }

    fn luminance(color: Color) -> f32 {
        fn linear(v: f32) -> f32 {
            if v <= 0.04045 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055).powf(2.4)
            }
        }
        0.2126 * linear(color.r) + 0.7152 * linear(color.g) + 0.0722 * linear(color.b)
    }

    fn colors(background: Background) -> Vec<Color> {
        match background {
            Background::Color(color) => vec![color],
            Background::Gradient(gradient::Gradient::Linear(linear)) => linear
                .stops
                .iter()
                .flatten()
                .map(|stop| stop.color)
                .collect(),
        }
    }

    fn composite(foreground: Color, background: Color) -> Color {
        let mix = |front, back| foreground.a * front + (1.0 - foreground.a) * back;
        Color::from_rgb(
            mix(foreground.r, background.r),
            mix(foreground.g, background.g),
            mix(foreground.b, background.b),
        )
    }

    #[track_caller]
    fn assert_contrast(foreground: Color, background: Color, minimum: f32) {
        let (a, b) = (
            luminance(composite(foreground, background)),
            luminance(background),
        );
        let ratio = (a.max(b) + 0.05) / (a.min(b) + 0.05);
        assert!(
            ratio >= minimum,
            "{foreground:?} over {background:?} has contrast {ratio}"
        );
    }

    #[test]
    fn body_and_secondary_copy_meet_aa_contrast_in_all_palettes() {
        for theme in Theme::ALL.iter().map(glass_theme) {
            let backgrounds = [
                backdrop(&theme).background,
                surface(&theme).background,
                glass(&theme).background,
                navigation(&theme, button::Status::Active).background,
                focused_row(&theme, button::Status::Active).background,
                badge(&theme).background,
            ];
            let backdrop_colors = colors(backdrop(&theme).background.unwrap());
            let panel_colors = backgrounds
                .into_iter()
                .flat_map(|fill| colors(fill.unwrap()));
            for (fill, base) in panel_colors.flat_map(|fill| {
                backdrop_colors
                    .iter()
                    .copied()
                    .map(move |base| (fill, base))
            }) {
                for fg in [
                    theme.palette().text,
                    muted(&theme).color.unwrap(),
                    theme.palette().primary,
                    theme.palette().danger,
                ] {
                    assert_contrast(fg, composite(fill, base), 4.5);
                }
            }
            for status in [
                button::Status::Active,
                button::Status::Hovered,
                button::Status::Pressed,
            ] {
                let style = primary(&theme, status);
                for color in colors(style.background.unwrap()) {
                    assert_contrast(style.text_color, color, 4.5);
                }
            }
        }
    }

    fn button_backdrops(theme: &Theme) -> Vec<Color> {
        let mut parents = Vec::new();
        for base in colors(backdrop(theme).background.unwrap()) {
            parents.push(base);
            for layer in [surface(theme), glass(theme)] {
                for fill in colors(layer.background.unwrap()) {
                    parents.push(composite(fill, base));
                }
            }
        }
        parents
    }

    #[test]
    fn enabled_buttons_stay_visible_on_every_backdrop() {
        for theme in Theme::ALL.iter().map(glass_theme) {
            let parents = button_backdrops(&theme);
            for status in [
                button::Status::Active,
                button::Status::Hovered,
                button::Status::Pressed,
            ] {
                for &parent in &parents {
                    for fill in colors(primary(&theme, status).background.unwrap()) {
                        assert_contrast(fill, parent, 3.0);
                    }
                    for style in [
                        secondary,
                        glass_button,
                        navigation,
                        focused_secondary,
                        focused_glass_button,
                    ] {
                        let button = style(&theme, status);
                        assert_contrast(button.border.color, parent, 3.0);
                        for fill in colors(button.background.unwrap()) {
                            let inside = composite(fill, parent);
                            assert_contrast(button.text_color, inside, 4.5);
                            assert_contrast(muted(&theme).color.unwrap(), inside, 4.5);
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn focus_indicators_have_three_to_one_contrast_in_all_palettes() {
        for theme in Theme::ALL.iter().map(glass_theme) {
            let status = button::Status::Active;
            let focus = [
                focus_ring(&theme).border,
                focused_row(&theme, status).border,
                focused_glass_button(&theme, status).border,
                focused_secondary(&theme, status).border,
            ];
            let backdrop_colors = colors(backdrop(&theme).background.unwrap());
            let fills = [
                backdrop(&theme).background,
                surface(&theme).background,
                glass(&theme).background,
                focused_row(&theme, status).background,
            ];
            for border in focus {
                assert_eq!(border.width, 2.0);
                assert_eq!(border.color.a, 1.0);
                for fill in fills.into_iter().flat_map(|fill| colors(fill.unwrap())) {
                    for &base in &backdrop_colors {
                        assert_contrast(border.color, composite(fill, base), 3.0);
                    }
                }
            }
        }
    }
}
