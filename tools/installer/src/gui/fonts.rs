use std::sync::OnceLock;

use iced::Font;
use iced::advanced::graphics::text::font_system;

pub(super) fn interface() -> Font {
    static FONT: OnceLock<Font> = OnceLock::new();
    *FONT.get_or_init(|| {
        installed_font(
            &[
                "Noto Sans CJK KR",
                "Noto Sans KR",
                "Malgun Gothic",
                "Apple SD Gothic Neo",
                "NanumGothic",
                "Noto Sans CJK JP",
                "Yu Gothic UI",
                "Hiragino Sans",
            ],
            Font::DEFAULT,
        )
    })
}

pub(super) fn display() -> Font {
    Font {
        weight: iced::font::Weight::Bold,
        stretch: iced::font::Stretch::Condensed,
        style: iced::font::Style::Italic,
        ..interface()
    }
}

pub(super) fn monospace() -> Font {
    static FONT: OnceLock<Font> = OnceLock::new();
    *FONT.get_or_init(|| {
        installed_font(
            &["Noto Sans Mono CJK KR", "Sarasa Mono K", "D2Coding"],
            Font::MONOSPACE,
        )
    })
}

fn installed_font(families: &[&'static str], fallback: Font) -> Font {
    let mut system = font_system().write().expect("Write font system");
    let database = system.raw().db();
    families
        .iter()
        .find(|name| {
            database
                .faces()
                .any(|face| face.families.iter().any(|(family, _)| family == **name))
        })
        .map_or(fallback, |name| Font::with_name(name))
}
