fn main() {
    match terminal_light::luma() {
        Ok(luma) => {
            println!("✓ Terminal background detected!");
            println!("  Luma value: {:.3}", luma);
            println!("  Interpretation: {}", if luma > 0.6 { "LIGHT mode" } else { "DARK mode" });
            println!("  RGB estimation: ~{} brightness", (luma * 255.0) as u8);
        }
        Err(e) => {
            println!("✗ Failed to detect terminal background");
            println!("  Error: {:?}", e);
            println!("  Your terminal may not support OSC 11 queries");
        }
    }
}
