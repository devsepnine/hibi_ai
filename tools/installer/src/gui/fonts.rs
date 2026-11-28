use iced::Font;

pub(super) const REGULAR: Font = Font::with_name("JetBrainsMonoHangul Nerd Font Mono");
pub(super) const BOLD: Font = Font {
    weight: iced::font::Weight::Bold,
    ..REGULAR
};
pub(super) const COLLECTION: &[u8] =
    include_bytes!("../../assets/fonts/JetBrainsMonoHangulNerdFontMono.ttc");

#[cfg(test)]
mod tests {
    use super::*;
    use cosmic_text::fontdb::{Database, Query};
    use iced::advanced::graphics::text::{cosmic_text, to_attributes};

    #[test]
    fn both_interface_faces_render_hangul_latin_and_nerd_symbols_without_system_fonts() {
        let mut db = Database::new();
        db.load_font_data(COLLECTION.to_vec());
        for font in [REGULAR, BOLD] {
            let attrs = to_attributes(font);
            let id = db
                .query(&Query {
                    families: &[attrs.family],
                    weight: attrs.weight,
                    stretch: attrs.stretch,
                    style: attrs.style,
                })
                .unwrap();
            assert_eq!(db.face(id).unwrap().weight, attrs.weight);
            let face = cosmic_text::Font::new(&db, id, attrs.weight).unwrap();
            let glyphs = face.as_swash().charmap();
            for ch in ['H', '0', '가', '힣', 'ㄱ', '\u{f120}'] {
                assert_ne!(glyphs.map(ch), 0, "{font:?} lacks {ch}");
            }
        }
    }
}
