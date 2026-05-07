# iced Theming & Styling

iced는 세련된 테마 세트와 커스텀 widget이 읽을 수 있는 palette 시스템을 제공한다. 스타일링 스토리는 세 레이어이다: 한 줄짜리를 위한 **빌트인 styler**, widget별 미세 조정을 위한 **커스텀 style closure**, 전체 앱 모양을 위한 **커스텀 테마**.

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

`Theme::ALL`은 모든 빌트인이 있는 `&'static [Theme]`이다 — `pick_list`에 그대로 전달한다:

```rust
pick_list(Theme::ALL, Some(&app.theme), Message::ThemeChanged)
```

빌더로 설정:

```rust
.theme(|app: &App| app.theme.clone())
```

## Built-in Widget Stylers

각 스타일 가능한 widget은 미리 만들어진 style 함수 세트를 노출한다. 일반적인 경우의 한 줄짜리:

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

이를 먼저 사용한다. 빌트인이 맞지 않을 때만 커스텀 closure로 점프한다.

## Custom Style Closures

style closure는 `(theme, status)`를 받고 `Style`을 반환한다. 패턴은: 빌트인에서 시작해 다른 것만 override한다:

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

**Status variants**는 widget마다 다르다; `button`의 경우: `Active`, `Hovered`, `Pressed`, `Disabled`.

## The Palette

`theme.extended_palette()`는 의미적 색 쌍을 가진 `palette::Extended`를 준다. 문법:

```
palette . <role> . <strength> . <part>
        │        │             └─ color | text
        │        └─ base | strong | weak
        └─ primary | secondary | success | danger | background
```

예:

```rust
let p = theme.extended_palette();
p.background.base.color     // app background
p.background.base.text      // default text
p.primary.strong.color      // emphasized primary
p.primary.base.text         // readable text on primary.base background
p.danger.weak.color         // subtle danger tint
```

**Rule of thumb**: 배경색을 `.text` 형제와 짝짓는다. 배경으로 `p.primary.base.color` + 전경으로 `p.primary.base.text`는 항상 충분한 대비를 가진다.

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

빌트인 테마가 브랜드와 맞지 않을 때, palette를 정의하고 `Theme::custom`을 빌드한다:

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

iced는 base palette에서 `strong`/`weak` 변형을 자동으로 계산한다. 더 많은 제어가 필요하면 `Custom::with_fn`을 통해 특정 톤을 override하지만, 기본 생성이 거의 항상 괜찮다.

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

- `.default_font(...)`는 widget이 명시하지 않을 때 사용되는 font를 설정한다.
- `.font(bytes)`는 font face를 앱에 로드하므로 그 이름이 해석 가능해진다.
- text별 override: `text("Hello").font(font::Font { ... })`.

**Tip**: 코드를 보여주는 view가 있다면 monospace font도 로드한다:

```rust
text("let x = 1;").font(font::Font::MONOSPACE)
```

## Dark/Light auto-switching

`window::Event`를 통해 OS theme를 watch한다:

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

(API 이름은 0.13/0.14 사이에 진화한다 — 버전의 `window::Event` enum을 확인한다.)

## When to customize vs use built-in

- **작은 앱, 기본 모양 좋음**: `Theme::Light/Dark` + 기본 widget styler. 끝.
- **브랜드 일관성**: 커스텀 `Palette`. widget styler는 빌트인으로 두라; palette가 흐른다.
- **특정 widget 미세 조정**: 그 widget에만 커스텀 style closure. theme를 건드리지 마라.
- **픽셀 퍼펙트 디자인**: 커스텀 palette + widget당 커스텀 closure. 공유 style fn이 있는 `style` 모듈에 투자.

## Style Module Organization

몇 화면 이상 앱의 경우, style을 `view`에서 분리한다:

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

이는 `view`를 narrative ("save라고 하는 button")로 유지하고 style 변경을 한 파일에 국한시킨다.
