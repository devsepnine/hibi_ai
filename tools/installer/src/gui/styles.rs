use iced::widget::{button, checkbox, container, rule, text, text_input};
use iced::{Border, Color, Theme, color};

pub fn theme(dark: bool) -> Theme {
    Theme::custom(
        if dark { "Hibi Dark" } else { "Hibi Light" },
        iced::theme::Palette {
            background: if dark {
                color!(0x1e1e2e)
            } else {
                color!(0xf5f5fa)
            },
            text: if dark {
                color!(0xd6d8ee)
            } else {
                color!(0x292938)
            },
            primary: if dark {
                color!(0xc4b5fd)
            } else {
                color!(0x6550a5)
            },
            success: if dark {
                color!(0xa3cfb3)
            } else {
                color!(0x286c49)
            },
            warning: if dark {
                color!(0xe5c890)
            } else {
                color!(0x805f16)
            },
            danger: if dark {
                color!(0xf2a6b9)
            } else {
                color!(0xa33a56)
            },
        },
    )
}

pub fn is_dark(theme: &Theme) -> bool {
    theme.palette().background.r < 0.3
}

pub fn divider(theme: &Theme) -> rule::Style {
    rule::Style {
        color: surface(theme).border.color,
        ..rule::default(theme)
    }
}

pub fn muted(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(if is_dark(theme) {
            color!(0xaaaac1)
        } else {
            color!(0x626277)
        }),
    }
}

pub fn surface(theme: &Theme) -> container::Style {
    container::Style {
        background: Some(
            if is_dark(theme) {
                color!(0x242437)
            } else {
                Color::WHITE
            }
            .into(),
        ),
        border: Border {
            color: if is_dark(theme) {
                color!(0x3b3b54)
            } else {
                color!(0xe0dfeb)
            },
            width: 1.0,
            radius: 10.0.into(),
        },
        ..Default::default()
    }
}

pub fn sidebar(theme: &Theme) -> container::Style {
    container::Style {
        background: Some(
            if is_dark(theme) {
                color!(0x27273b)
            } else {
                color!(0xeeeef6)
            }
            .into(),
        ),
        border: Border {
            radius: 10.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

pub fn subtle(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = button::text(theme, status);
    style.border = Border {
        radius: 7.0.into(),
        ..Default::default()
    };
    style.text_color = theme.palette().text;
    style.background = match status {
        button::Status::Hovered | button::Status::Pressed => Some(
            if is_dark(theme) {
                color!(0x303047)
            } else {
                color!(0xe8e6f2)
            }
            .into(),
        ),
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

pub fn secondary(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = subtle(theme, status);
    let surface = surface(theme);
    style.border = surface.border;
    style.border.radius = 8.0.into();
    if matches!(status, button::Status::Active) {
        style.background = surface.background;
    }
    if status == button::Status::Disabled {
        style.border.color = style.border.color.scale_alpha(0.4);
    }
    style
}

pub fn primary(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = button::primary(theme, status);
    style.border.radius = 8.0.into();
    style
}

pub fn destructive(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = button::danger(theme, status);
    style.border.radius = 8.0.into();
    style
}

pub fn navigation(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = subtle(theme, status);
    style.background = Some(
        if is_dark(theme) {
            color!(0x35334c)
        } else {
            color!(0xebe7f5)
        }
        .into(),
    );
    style.text_color = theme.palette().text;
    style
}

pub fn focused_row(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = secondary(theme, status);
    if status == button::Status::Active {
        style.background = Some(
            if is_dark(theme) {
                color!(0x2f2d43)
            } else {
                color!(0xf1eef8)
            }
            .into(),
        );
    }
    style.border.color = theme.palette().primary.scale_alpha(0.35);
    style
}

pub fn input(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let mut style = text_input::default(theme, status);
    style.background = theme.palette().background.into();
    style.placeholder = muted(theme).color.unwrap_or(theme.palette().text);
    style.border = surface(theme).border;
    style.border.radius = 8.0.into();
    if matches!(status, text_input::Status::Focused { .. }) {
        style.border.color = theme.palette().primary;
    }
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
        style.background = if is_dark(theme) {
            color!(0x47405f)
        } else {
            color!(0xe5ddf5)
        }
        .into();
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
        background: Some(
            if is_dark(theme) {
                color!(0x303047)
            } else {
                color!(0xefedf6)
            }
            .into(),
        ),
        text_color: muted(theme).color,
        border: Border {
            radius: 5.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn body_and_secondary_copy_meet_aa_contrast_in_both_themes() {
        for dark in [false, true] {
            let theme = theme(dark);
            let backgrounds = [
                Some(theme.palette().background.into()),
                surface(&theme).background,
                sidebar(&theme).background,
                navigation(&theme, button::Status::Active).background,
                focused_row(&theme, button::Status::Active).background,
                badge(&theme).background,
            ];
            for bg in backgrounds {
                let Some(iced::Background::Color(bg)) = bg else {
                    panic!("Expected a solid surface color");
                };
                for fg in [
                    theme.palette().text,
                    muted(&theme).color.unwrap(),
                    theme.palette().primary,
                    theme.palette().danger,
                ] {
                    let (a, b) = (luminance(fg), luminance(bg));
                    let ratio = (a.max(b) + 0.05) / (a.min(b) + 0.05);
                    assert!(ratio >= 4.5, "dark={dark}, color={fg:?}, contrast={ratio}");
                }
            }
        }
    }
}
