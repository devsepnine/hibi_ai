# Ratatui Widgets (0.30 API)

빌트인 위젯과 그 관용적 0.30 사용법. 모든 예제는 `Block::bordered()`, `Layout::*::areas()` 디스트럭처링, `f.area()`를 사용한다.

## Paragraph

```rust
use ratatui::widgets::{Block, Paragraph, Wrap};

let p = Paragraph::new("Hello, world!")
    .block(Block::bordered().title("Title"))
    .wrap(Wrap { trim: true });

f.render_widget(p, area);
```

리치 텍스트 (span당 mixed 스타일)의 경우, `Text::from(vec![Line::from(vec![Span::raw, Span::styled])])`를 사용한다.

## List (Stateful — supports selection)

```rust
use ratatui::widgets::{Block, List, ListItem, ListState};
use ratatui::style::{Color, Style, Modifier};

let items: Vec<ListItem> = app.items
    .iter()
    .map(|i| ListItem::new(i.as_str()))
    .collect();

let list = List::new(items)
    .block(Block::bordered().title("Items"))
    .highlight_style(
        Style::new()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD)
    )
    .highlight_symbol("> ");

let mut state = ListState::default();
state.select(Some(app.selected));
f.render_stateful_widget(list, area, &mut state);
```

**Key point**: `ListState`는 위젯 외부에 살아 selection이 redraw 간에 지속된다. `App`에 저장한다.

## Table

```rust
use ratatui::widgets::{Block, Table, Row};
use ratatui::layout::Constraint;
use ratatui::style::{Color, Style};

let header = Row::new(vec!["Name", "Age", "City"])
    .style(Style::new().fg(Color::Yellow));

let rows = vec![
    Row::new(vec!["Alice", "30", "NYC"]),
    Row::new(vec!["Bob",   "25", "SF"]),
];

let table = Table::new(rows, [
    Constraint::Percentage(40),
    Constraint::Percentage(20),
    Constraint::Percentage(40),
])
.header(header)
.block(Block::bordered().title("Users"))
.row_highlight_style(Style::new().bg(Color::DarkGray));

f.render_widget(table, area);
```

선택 가능한 Table의 경우, `TableState`와 `render_stateful_widget`을 사용한다 (`List`와 평행).

## Gauge

```rust
use ratatui::widgets::{Block, Gauge};
use ratatui::style::{Color, Style};

let g = Gauge::default()
    .block(Block::bordered().title("Progress"))
    .gauge_style(Style::new().fg(Color::Green))
    .percent(app.progress); // 0..=100

f.render_widget(g, area);
```

게이지 내부 라벨: `.label(format!("{}/{}", done, total))`.

## Tabs

```rust
use ratatui::widgets::{Block, Tabs};
use ratatui::style::{Color, Modifier, Style};

let tabs = Tabs::new(vec!["Logs", "Config", "Help"])
    .block(Block::bordered().title("Mode"))
    .select(app.tab_index)
    .style(Style::new().fg(Color::White))
    .highlight_style(
        Style::new()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
    )
    .divider(" | ");

f.render_widget(tabs, area);
```

## Block (containers, titles, borders)

```rust
use ratatui::widgets::{Block, Borders, BorderType, Padding};
use ratatui::layout::Alignment;
use ratatui::style::{Color, Style};
use ratatui::text::Line;

let block = Block::bordered()
    .title("Center Title")
    .title_alignment(Alignment::Center)
    .title_bottom(Line::from(" footer ").style(Style::new().fg(Color::Gray)))
    .border_type(BorderType::Rounded)
    .border_style(Style::new().fg(Color::Cyan))
    .padding(Padding::horizontal(2));

// Get inner area for nested content (excludes borders + padding):
let inner = block.inner(area);
f.render_widget(block, area);
f.render_widget(content, inner);
```

`Block::bordered()`는 `Block::default().borders(Borders::ALL)`의 0.30 단축형이다.

## Scrollbar

```rust
use ratatui::widgets::{Scrollbar, ScrollbarOrientation, ScrollbarState};

let mut scroll_state = ScrollbarState::new(total_items).position(current_position);
let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);

f.render_stateful_widget(scrollbar, area, &mut scroll_state);
```

bar가 보이는 것과 일치하도록 `Paragraph`의 `.scroll((y, 0))`과 짝지운다.

## Sparkline

```rust
use ratatui::widgets::{Block, Sparkline};
use ratatui::style::{Color, Style};

let sparkline = Sparkline::default()
    .block(Block::bordered().title("Throughput"))
    .data(&app.data_points)
    .style(Style::new().fg(Color::Green));

f.render_widget(sparkline, area);
```

## BarChart

```rust
use ratatui::widgets::{Block, BarChart};
use ratatui::style::{Color, Style};

let chart = BarChart::default()
    .block(Block::bordered().title("Counts"))
    .bar_width(3)
    .bar_gap(1)
    .bar_style(Style::new().fg(Color::Yellow))
    .data(&[("a", 10), ("b", 25), ("c", 7)]);

f.render_widget(chart, area);
```

## Chart (line / scatter)

```rust
use ratatui::widgets::{Axis, Block, Chart, Dataset, GraphType};
use ratatui::style::{Color, Style};
use ratatui::symbols;

let dataset = Dataset::default()
    .name("series-1")
    .marker(symbols::Marker::Braille)
    .style(Style::new().fg(Color::Cyan))
    .graph_type(GraphType::Line)
    .data(&[(0.0, 1.0), (1.0, 2.0), (2.0, 1.5)]);

let chart = Chart::new(vec![dataset])
    .block(Block::bordered().title("Series"))
    .x_axis(Axis::default().title("X").bounds([0.0, 5.0]))
    .y_axis(Axis::default().title("Y").bounds([0.0, 3.0]));

f.render_widget(chart, area);
```

## Custom Widget (impl Widget)

빌트인이 맞지 않을 때, struct에 `Widget`을 구현한다. render fn은 buffer에 직접 쓴다:

```rust
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Widget;
use ratatui::text::Line;

struct StatusLine { msg: String }

impl Widget for StatusLine {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Line::raw(self.msg).render(area, buf);
    }
}

// Usage
f.render_widget(StatusLine { msg: "Ready".into() }, area);
```

draw 간에 변경 가능한 state가 필요한 위젯의 경우, 대신 `StatefulWidget`을 구현한다.

## Widget Selection Cheatsheet

| Need | Widget |
|---|---|
| 정적 텍스트 / 다중 라인 | Paragraph |
| 선택 가능한 리스트 | List + ListState |
| 표 형태 데이터 | Table (+선택 가능하면 TableState) |
| 진행 0–100% | Gauge |
| 최상위 모드 탭 | Tabs |
| 제목 + 테두리가 있는 컨테이너 | Block |
| 긴 콘텐츠의 위치 표시 | Scrollbar |
| 시계열 미니 그래프 | Sparkline / Chart |
| 카테고리별 카운트 | BarChart |
| 어떤 맞춤형이든 | impl Widget |
