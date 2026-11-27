use iced::widget::{button, checkbox, container, pick_list, rule, text, text_input};
use iced::{Border, Color, Degrees, Theme, color, gradient};

const PANEL_RADIUS: f32 = 18.0;
const PILL_RADIUS: f32 = 999.0;

pub fn theme(dark: bool) -> Theme {
    use iced::theme::Palette;

    let (name, palette) = if dark {
        (
            "Hibi Dark",
            Palette {
                background: color!(0x232639),
                text: color!(0xf3f2fd),
                primary: color!(0xd1bfff),
                success: color!(0xace0c1),
                warning: color!(0xf3d396),
                danger: color!(0xffb1c7),
            },
        )
    } else {
        (
            "Hibi Light",
            Palette {
                background: color!(0xe9eaf8),
                text: color!(0x22243a),
                primary: color!(0x584099),
                success: color!(0x226148),
                warning: color!(0x74500e),
                danger: color!(0x922c51),
            },
        )
    };
    Theme::custom(name, palette)
}

fn tone(theme: &Theme, light: Color, dark: Color) -> Color {
    if theme.extended_palette().is_dark {
        dark
    } else {
        light
    }
}

pub fn backdrop(theme: &Theme) -> container::Style {
    container::Style::default().background(
        gradient::Linear::new(Degrees(126.0))
            .add_stop(0.0, tone(theme, color!(0xf7f8ff), color!(0x383451)))
            .add_stop(0.48, theme.palette().background)
            .add_stop(1.0, tone(theme, color!(0xd7dbf4), color!(0x181b2a))),
    )
}

pub fn divider(theme: &Theme) -> rule::Style {
    rule::Style {
        color: surface(theme).border.color,
        ..rule::default(theme)
    }
}

pub fn muted(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(tone(theme, color!(0x50546c), color!(0xc6c8dc))),
    }
}

pub fn surface(theme: &Theme) -> container::Style {
    container::Style {
        background: Some(tone(theme, color!(0xfbfaff), color!(0x1e2131)).into()),
        border: Border {
            color: tone(theme, color!(0xd8daeb), color!(0x3b4058)),
            width: 1.0,
            radius: PANEL_RADIUS.into(),
        },
        ..Default::default()
    }
}

pub fn glass(theme: &Theme) -> container::Style {
    // The software renderer paints shadows outside the damage clip on partial redraws.
    let highlight = tone(theme, Color::WHITE, color!(0x555071));
    let base = tone(theme, color!(0xf3f2ff), color!(0x21263b));
    let fill = gradient::Linear::new(Degrees(154.0))
        .add_stop(0.0, highlight.scale_alpha(0.78))
        .add_stop(0.42, base.scale_alpha(0.88))
        .add_stop(1.0, base.scale_alpha(0.64));
    container::Style {
        background: Some(fill.into()),
        border: Border {
            color: tone(theme, Color::WHITE, color!(0x9d94c4)).scale_alpha(0.55),
            width: 1.0,
            radius: PANEL_RADIUS.into(),
        },
        ..Default::default()
    }
}

pub fn subtle(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = button::text(theme, status);
    style.border = Border {
        radius: 10.0.into(),
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
    tone(theme, color!(0xe2e1f6), color!(0x373b56)).scale_alpha(alpha)
}

pub fn secondary(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = subtle(theme, status);
    let surface = surface(theme);
    style.border = surface.border;
    style.border.radius = PILL_RADIUS.into();
    if matches!(status, button::Status::Active) {
        style.background = surface.background;
    }
    if status == button::Status::Disabled {
        style.border.color = style.border.color.scale_alpha(0.4);
    }
    style
}

pub fn glass_button(theme: &Theme, status: button::Status) -> button::Style {
    let mut material = glass(theme);
    if matches!(status, button::Status::Hovered | button::Status::Pressed) {
        material.background = Some(hover_fill(theme, 0.9).into());
        material.border.color = theme.palette().primary.scale_alpha(0.65);
    }
    button::Style {
        background: material.background,
        border: material.border,
        ..subtle(theme, status)
    }
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
    style.border.radius = PILL_RADIUS.into();
    if status != button::Status::Disabled {
        let fill = match status {
            button::Status::Hovered => tone(theme, color!(0x6245ae), color!(0xe3d7ff)),
            button::Status::Pressed => tone(theme, color!(0x4f348f), color!(0xbba2f5)),
            _ => theme.palette().primary,
        };
        style.background = Some(fill.into());
        style.text_color = tone(theme, Color::WHITE, color!(0x241b38));
        style.border.color = Color::WHITE.scale_alpha(0.35);
        style.border.width = 1.0;
    }
    style
}

pub fn destructive(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = button::danger(theme, status);
    style.border.radius = PILL_RADIUS.into();
    style
}

pub fn navigation(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = subtle(theme, status);
    style.background = Some(match status {
        button::Status::Hovered | button::Status::Pressed => hover_fill(theme, 1.0).into(),
        _ => tone(theme, color!(0xf0ecff), color!(0x38324e)).into(),
    });
    style.border = Border {
        color: theme.palette().primary.scale_alpha(0.24),
        width: 1.0,
        radius: 10.0.into(),
    };
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
        border: focus_border(theme, 8.0),
        ..Default::default()
    }
}

pub fn input(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let mut style = text_input::default(theme, status);
    style.background = tone(theme, color!(0xfbfaff), color!(0x202438)).into();
    style.placeholder = muted(theme).color.unwrap_or(theme.palette().text);
    style.border = surface(theme).border;
    style.border.radius = PILL_RADIUS.into();
    if matches!(status, text_input::Status::Focused { .. }) {
        style.border = focus_border(theme, PILL_RADIUS);
    }
    style
}

pub fn selector(theme: &Theme, status: pick_list::Status) -> pick_list::Style {
    let mut style = pick_list::default(theme, status);
    style.background = if status == pick_list::Status::Active {
        tone(theme, color!(0xfbfaff), color!(0x202438)).into()
    } else {
        hover_fill(theme, 1.0).into()
    };
    style.border = surface(theme).border;
    style.border.radius = PILL_RADIUS.into();
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
        style.background = tone(theme, color!(0xe7dffb), color!(0x494061)).into();
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
        background: Some(tone(theme, color!(0xeceaf8), color!(0x30344a)).into()),
        text_color: muted(theme).color,
        border: Border {
            radius: 9.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::Background;

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

    fn assert_contrast(foreground: Color, background: Color, minimum: f32) {
        let (a, b) = (luminance(foreground), luminance(background));
        let ratio = (a.max(b) + 0.05) / (a.min(b) + 0.05);
        assert!(
            ratio >= minimum,
            "{foreground:?} over {background:?} has contrast {ratio}"
        );
    }

    #[test]
    fn body_and_secondary_copy_meet_aa_contrast_in_both_themes() {
        for dark in [false, true] {
            let theme = theme(dark);
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

    #[test]
    fn focus_indicators_have_three_to_one_contrast_in_both_themes() {
        for dark in [false, true] {
            let theme = theme(dark);
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
