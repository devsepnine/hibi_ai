use ratatui::layout::Rect;
use ratatui::text::Span;

/// Terminal columns `s` occupies once painted.
///
/// Deferred to ratatui rather than measured here, because the number that
/// matters is the one the painter will use — a budget computed by any other
/// measure can only agree with the screen by coincidence. Both obvious
/// shortcuts drift, and in opposite directions: `str::len` counts bytes, so a
/// three-byte `→` over-charges by two, while `chars().count()` counts
/// codepoints, so a double-width CJK glyph under-charges by one.
pub fn columns(s: &str) -> usize {
    Span::raw(s).width()
}

/// Centre a box of at most `w` x `h` in `area`, shrinking rather than
/// overflowing.
///
/// Overlays size themselves from their content, which can outgrow the terminal
/// — the keybinding table is taller than a 24-row window and the confirm prompt
/// wider than a 30-column one. Clamping here is what keeps the centring
/// subtraction from underflowing, so a too-small terminal gets a cropped box
/// instead of a panic.
pub fn centered(area: Rect, w: u16, h: u16) -> Rect {
    let w = w.min(area.width);
    let h = h.min(area.height);
    Rect {
        x: area.x + (area.width - w) / 2,
        y: area.y + (area.height - h) / 2,
        width: w,
        height: h,
    }
}
