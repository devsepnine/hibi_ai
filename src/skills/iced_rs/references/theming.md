# iced Theming & Styling

iced ships a set of polished themes and a palette system that custom widgets can read. The styling story has three layers: **built-in stylers** for one-liners, **custom style closures** for per-widget tweaks, and **custom themes** for whole-app looks.

## Built-in Themes

```rust
use iced::Theme;

Theme::Light
Theme::Dark
Theme::Dracula
Theme::Nord
Theme::SolarizedLight / Theme::SolarizedDark
Theme::GruvboxLight / Theme::GruvboxDark
Theme::CatppuccinLatte / Frappe / Macchiato / Mocha
Theme::TokyoNight / TokyoNightStorm / TokyoNightLight
Theme::KanagawaWave / KanagawaDragon / KanagawaLotus
Theme::Moonfly / Nightfly / Oxocarbon / Ferra
```

`Theme::ALL` is a `&'static [Theme]` with every built-in — pass it straight to `pick_list`:

```rust
pick_list(Theme::ALL, Some(&app.theme), Message::ThemeChanged)
```

Set via the builder:

```rust
.theme(|app: &App| app.theme.clone())
```

## Built-in Widget Stylers

Each styleable widget exposes a set of ready-made style functions. One-liners for the common cases:

```rust
button("Save").style(button::primary);
button("Cancel").style(button::secondary);
button("Confirm").style(button::success);
button("Delete").style(button::danger);
button("Link").style(button::text);

text("Info").style(text::primary);
text("Warn").style(text::warning);
text("Err").style(text::danger);
text("Good").style(text::success);

container("Card").style(container::rounded_box);
container("Box").style(container::bordered_box);

text_input("...", &v).style(text_input::default);   // rarely needed — default is good
```

Use these first. Jump to a custom closure only when the built-in doesn't match.

## Custom Style Closures

A style closure receives `(theme, status)` and returns a `Style`. The pattern is: start from a built-in, override only what differs:

```rust
button("Delete")
    .on_press(Message::Delete)
    .style(|theme, status| {
        let palette = theme.extended_palette();
        match status {
            button::Status::Hovered => button::Style {
                background: Some(palette.danger.strong.color.into()),
                ..button::danger(theme, status)
            },
            _ => button::danger(theme, status),
        }
    });
```

**Status variants** differ per widget; for `button`: `Active`, `Hovered`, `Pressed`, `Disabled`.

## The Palette

`theme.extended_palette()` gives you a `palette::Extended` with semantic color pairs. The grammar:

```
palette . <role> . <strength> . <part>
        │        │             └─ color | text
        │        └─ base | strong | weak
        └─ primary | secondary | success | danger | background
```

Examples:

```rust
let p = theme.extended_palette();
p.background.base.color     // app background
p.background.base.text      // default text
p.primary.strong.color      // emphasized primary
p.primary.base.text         // readable text on primary.base background
p.danger.weak.color         // subtle danger tint
```

**Rule of thumb**: pair a background color with its `.text` sibling. `p.primary.base.color` as background + `p.primary.base.text` as foreground always has sufficient contrast.

## Container Custom Style

```rust
use iced::{Background, Border, Shadow, Color, Theme};
use iced::widget::container;

container(content)
    .padding(20)
    .style(|theme: &Theme| {
        let p = theme.extended_palette();
        container::Style {
            background: Some(Background::Color(p.background.weak.color)),
            border: Border {
                radius: 12.0.into(),
                width: 1.0,
                color: p.background.strong.color,
            },
            shadow: Shadow::default(),
            text_color: Some(p.background.weak.text),
            ..Default::default()
        }
    })
```

## Custom Theme (Custom Palette)

When the built-in themes don't match your brand, define a palette and build a `Theme::custom`:

```rust
use iced::theme::{Custom, Palette};
use iced::{Color, Theme};
use std::sync::Arc;

fn brand_theme() -> Theme {
    let palette = Palette {
        background: Color::from_rgb8(0x10, 0x11, 0x14),
        text:       Color::from_rgb8(0xE6, 0xE7, 0xEA),
        primary:    Color::from_rgb8(0x4C, 0x8B, 0xF5),   // your brand blue
        success:    Color::from_rgb8(0x5A, 0xC8, 0x8E),
        warning:    Color::from_rgb8(0xF1, 0xC2, 0x3C),
        danger:     Color::from_rgb8(0xE5, 0x6A, 0x6A),
    };
    Theme::custom("brand".to_string(), palette)
}

// Use in the builder
.theme(|_| brand_theme())
```

iced computes the `strong`/`weak` variations from the base palette automatically. Override specific tones via `Custom::with_fn` if you need more control, but the default generation is almost always fine.

## Fonts

```rust
use iced::font;

iced::application(App::new, App::update, App::view)
    .default_font(font::Font {
        family: font::Family::Name("Inter"),
        weight: font::Weight::Medium,
        ..Default::default()
    })
    .font(include_bytes!("../assets/Inter-Medium.ttf").as_slice())
    .run()
```

- `.default_font(...)` sets the font used when a widget doesn't specify.
- `.font(bytes)` loads a font face into the app so its name is resolvable.
- Per-text override: `text("Hello").font(font::Font { ... })`.

**Tip**: load a monospace font too if any view shows code:

```rust
text("let x = 1;").font(font::Font::MONOSPACE)
```

## Dark/Light auto-switching

Watch the OS theme via `window::Event`:

```rust
window::events().map(Message::WindowEvent)

// in update:
Message::WindowEvent(window::Event::ThemeChanged(t)) => {
    self.theme = match t {
        iced::core::theme::Base::Dark => Theme::Dark,
        iced::core::theme::Base::Light => Theme::Light,
    };
    Task::none()
}
```

(API names evolve across 0.13/0.14 — check your version's `window::Event` enum.)

## When to customize vs use built-in

- **Small app, default look good**: `Theme::Light/Dark` + default widget stylers. Done.
- **Brand consistency**: custom `Palette`. Leave widget stylers as built-ins; the palette flows through.
- **Specific widget tweak**: custom style closure on that widget only. Don't touch the theme.
- **Pixel-perfect design**: custom palette + custom closures per widget. Invest in a `style` module with shared style fns.

## Style Module Organization

For apps > a few screens, keep styles out of `view`:

```rust
// src/style.rs
use iced::widget::{button, container};

pub fn primary_button(theme: &iced::Theme, status: button::Status) -> button::Style {
    /* ... */
}

pub fn card(theme: &iced::Theme) -> container::Style {
    /* ... */
}

// src/view.rs
use crate::style;

button("Save").style(style::primary_button);
container(content).style(style::card);
```

This keeps `view` narrative ("a button that says save") and style changes local to one file.
