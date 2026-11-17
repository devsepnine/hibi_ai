use iced::widget::{button, container, text};
use iced::{Border, Color, Theme, color};

pub fn theme(dark: bool) -> Theme {
    Theme::custom(
        if dark { "Hibi Dark" } else { "Hibi Light" },
        iced::theme::Palette {
            background: if dark {
                color!(0x14171c)
            } else {
                color!(0xf5f6f8)
            },
            text: if dark {
                color!(0xe8ecf3)
            } else {
                color!(0x202733)
            },
            primary: if dark {
                color!(0x91b8ff)
            } else {
                color!(0x275ed8)
            },
            success: if dark {
                color!(0x7cd4af)
            } else {
                color!(0x19734e)
            },
            warning: if dark {
                color!(0xf0c478)
            } else {
                color!(0x8b5b0b)
            },
            danger: if dark {
                color!(0xffa5ad)
            } else {
                color!(0xb52d40)
            },
        },
    )
}

pub fn is_dark(theme: &Theme) -> bool {
    theme.palette().background.r < 0.3
}

pub fn muted(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(if is_dark(theme) {
            color!(0xa4afbf)
        } else {
            color!(0x566274)
        }),
    }
}

pub fn surface(theme: &Theme) -> container::Style {
    container::Style {
        background: Some(
            if is_dark(theme) {
                color!(0x1d222a)
            } else {
                Color::WHITE
            }
            .into(),
        ),
        border: Border {
            color: if is_dark(theme) {
                color!(0x323a46)
            } else {
                color!(0xdde2ea)
            },
            width: 1.0,
            radius: 12.0.into(),
        },
        ..Default::default()
    }
}

pub fn sidebar(theme: &Theme) -> container::Style {
    container::Style {
        background: Some(
            if is_dark(theme) {
                color!(0x191d24)
            } else {
                color!(0xecf0f5)
            }
            .into(),
        ),
        border: Border {
            radius: 12.0.into(),
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
                color!(0x303947)
            } else {
                color!(0xe0e7f0)
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

pub fn navigation(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = subtle(theme, status);
    style.background = Some(
        if is_dark(theme) {
            color!(0x293c59)
        } else {
            color!(0xdce7fc)
        }
        .into(),
    );
    style.text_color = theme.palette().primary;
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
                color!(0x2a3240)
            } else {
                color!(0xeaf0f9)
            }
            .into(),
        ),
        text_color: Some(theme.palette().primary),
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
                theme.palette().background,
                if dark { color!(0x1d222a) } else { Color::WHITE },
                if dark {
                    color!(0x293c59)
                } else {
                    color!(0xdce7fc)
                },
                if dark {
                    color!(0x191d24)
                } else {
                    color!(0xecf0f5)
                },
                if dark {
                    color!(0x2a3240)
                } else {
                    color!(0xeaf0f9)
                },
            ];
            for bg in backgrounds {
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
