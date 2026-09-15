use ratatui::layout::Rect;

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
