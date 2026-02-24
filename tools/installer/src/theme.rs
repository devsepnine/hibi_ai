use ratatui::style::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    Mocha,  // Catppuccin Dark
    Latte,  // Catppuccin Light
}

impl ThemeMode {
    pub fn toggle(&self) -> Self {
        match self {
            Self::Mocha => Self::Latte,
            Self::Latte => Self::Mocha,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Mocha => "Mocha",
            Self::Latte => "Latte",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Theme {
    mode: ThemeMode,
}

impl Default for Theme {
    fn default() -> Self {
        // Detect system theme preference
        let mode = Self::detect_system_theme();
        Self { mode }
    }
}

impl Theme {
    /// Detect system theme preference (light/dark)
    fn detect_system_theme() -> ThemeMode {
        match terminal_light::luma() {
            Ok(luma) if luma > 0.6 => ThemeMode::Latte,  // Light terminal
            _ => ThemeMode::Mocha,  // Dark terminal or unable to detect
        }
    }

    pub fn mode(&self) -> ThemeMode {
        self.mode
    }

    pub fn toggle(&mut self) {
        self.mode = self.mode.toggle();
    }

    // Catppuccin Mocha base colors
    const MOCHA_BASE: Color = Color::Rgb(30, 30, 46);      // #1e1e2e
    const MOCHA_MANTLE: Color = Color::Rgb(24, 24, 37);    // #181825
    const MOCHA_SURFACE0: Color = Color::Rgb(49, 50, 68);  // #313244
    const MOCHA_SURFACE1: Color = Color::Rgb(69, 71, 90);  // #45475a
    const MOCHA_OVERLAY0: Color = Color::Rgb(108, 112, 134); // #6c7086
    const MOCHA_TEXT: Color = Color::Rgb(205, 214, 244);   // #cdd6f4
    const MOCHA_SUBTEXT0: Color = Color::Rgb(166, 173, 200); // #a6adc8

    // Catppuccin Mocha accent colors
    const MOCHA_BLUE: Color = Color::Rgb(137, 180, 250);   // #89b4fa
    const MOCHA_SAPPHIRE: Color = Color::Rgb(116, 199, 236); // #74c7ec
    const MOCHA_GREEN: Color = Color::Rgb(166, 227, 161);  // #a6e3a1
    const MOCHA_YELLOW: Color = Color::Rgb(249, 226, 175); // #f9e2af
    const MOCHA_RED: Color = Color::Rgb(243, 139, 168);    // #f38ba8
    const MOCHA_MAUVE: Color = Color::Rgb(203, 166, 247);  // #cba6f7
    const MOCHA_PEACH: Color = Color::Rgb(250, 179, 135);  // #fab387

    // Catppuccin Latte base colors
    const LATTE_BASE: Color = Color::Rgb(239, 241, 245);   // #eff1f5
    const LATTE_MANTLE: Color = Color::Rgb(230, 233, 239); // #e6e9ef
    const LATTE_SURFACE0: Color = Color::Rgb(204, 208, 218); // #ccd0da
    const LATTE_SURFACE1: Color = Color::Rgb(188, 192, 204); // #bcc0cc
    const LATTE_OVERLAY0: Color = Color::Rgb(140, 143, 161); // #8c8fa1
    const LATTE_TEXT: Color = Color::Rgb(76, 79, 105);     // #4c4f69
    const LATTE_SUBTEXT0: Color = Color::Rgb(108, 111, 133); // #6c6f85

    // Catppuccin Latte accent colors
    const LATTE_BLUE: Color = Color::Rgb(30, 102, 245);    // #1e66f5
    const LATTE_SAPPHIRE: Color = Color::Rgb(32, 159, 181); // #209fb5
    const LATTE_GREEN: Color = Color::Rgb(64, 160, 43);    // #40a02b
    const LATTE_YELLOW: Color = Color::Rgb(223, 142, 29);  // #df8e1d
    const LATTE_RED: Color = Color::Rgb(210, 15, 57);      // #d20f39
    const LATTE_MAUVE: Color = Color::Rgb(136, 57, 239);   // #8839ef
    const LATTE_PEACH: Color = Color::Rgb(254, 100, 11);   // #fe640b

    // Background colors
    pub fn bg_primary(&self) -> Color {
        match self.mode {
            ThemeMode::Mocha => Self::MOCHA_BASE,
            ThemeMode::Latte => Self::LATTE_BASE,
        }
    }

    pub fn bg_secondary(&self) -> Color {
        match self.mode {
            ThemeMode::Mocha => Self::MOCHA_MANTLE,
            ThemeMode::Latte => Self::LATTE_MANTLE,
        }
    }

    // Text colors
    pub fn text_primary(&self) -> Color {
        match self.mode {
            ThemeMode::Mocha => Self::MOCHA_TEXT,
            ThemeMode::Latte => Self::LATTE_TEXT,
        }
    }

    pub fn text_secondary(&self) -> Color {
        match self.mode {
            ThemeMode::Mocha => Self::MOCHA_SUBTEXT0,
            ThemeMode::Latte => Self::LATTE_SUBTEXT0,
        }
    }

    pub fn text_muted(&self) -> Color {
        match self.mode {
            ThemeMode::Mocha => Self::MOCHA_OVERLAY0,
            ThemeMode::Latte => Self::LATTE_OVERLAY0,
        }
    }

    // Accent colors
    pub fn accent_primary(&self) -> Color {
        match self.mode {
            ThemeMode::Mocha => Self::MOCHA_BLUE,
            ThemeMode::Latte => Self::LATTE_BLUE,
        }
    }

    pub fn accent_secondary(&self) -> Color {
        match self.mode {
            ThemeMode::Mocha => Self::MOCHA_SAPPHIRE,
            ThemeMode::Latte => Self::LATTE_SAPPHIRE,
        }
    }

    // Status colors
    pub fn success(&self) -> Color {
        match self.mode {
            ThemeMode::Mocha => Self::MOCHA_GREEN,
            ThemeMode::Latte => Self::LATTE_GREEN,
        }
    }

    pub fn warning(&self) -> Color {
        match self.mode {
            ThemeMode::Mocha => Self::MOCHA_YELLOW,
            ThemeMode::Latte => Self::LATTE_YELLOW,
        }
    }

    pub fn error(&self) -> Color {
        match self.mode {
            ThemeMode::Mocha => Self::MOCHA_RED,
            ThemeMode::Latte => Self::LATTE_RED,
        }
    }

    pub fn info(&self) -> Color {
        self.accent_secondary()
    }

    // UI element colors
    pub fn border(&self) -> Color {
        match self.mode {
            ThemeMode::Mocha => Self::MOCHA_SURFACE1,
            ThemeMode::Latte => Self::LATTE_SURFACE1,
        }
    }

    pub fn border_focused(&self) -> Color {
        self.accent_primary()
    }

    pub fn selection_bg(&self) -> Color {
        match self.mode {
            ThemeMode::Mocha => Self::MOCHA_SURFACE0,
            ThemeMode::Latte => Self::LATTE_SURFACE0,
        }
    }

    pub fn selection_fg(&self) -> Color {
        self.text_primary()
    }

    // Special colors
    pub fn highlight(&self) -> Color {
        match self.mode {
            ThemeMode::Mocha => Self::MOCHA_MAUVE,
            ThemeMode::Latte => Self::LATTE_MAUVE,
        }
    }

    pub fn diff_added(&self) -> Color {
        self.success()
    }

    pub fn diff_removed(&self) -> Color {
        self.error()
    }

    pub fn spinner(&self) -> Color {
        self.accent_primary()
    }

    pub fn peach(&self) -> Color {
        match self.mode {
            ThemeMode::Mocha => Self::MOCHA_PEACH,
            ThemeMode::Latte => Self::LATTE_PEACH,
        }
    }
}
