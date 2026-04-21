# Ratatui Widgets (0.30 API)

Built-in widgets and their idiomatic 0.30 usage. All examples use `Block::bordered()`, `Layout::*::areas()` destructuring, and `f.area()`.

## Paragraph

```rust
use ratatui::widgets::{Block, Paragraph, Wrap};

let p = Paragraph::new("Hello, world!")
    .block(Block::bordered().title("Title"))
    .wrap(Wrap { trim: true });

f.render_widget(p, area);
```

For rich text (mixed styles per span), use `Text::from(vec![Line::from(vec![Span::raw, Span::styled])])`.

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

**Key point**: `ListState` lives outside the widget so the selection persists across redraws. Store it in `App`.

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

For a selectable Table, use `TableState` and `render_stateful_widget` (parallel to `List`).

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

For a label inside the gauge: `.label(format!("{}/{}", done, total))`.

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

`Block::bordered()` is the 0.30 shorthand for `Block::default().borders(Borders::ALL)`.

## Scrollbar

```rust
use ratatui::widgets::{Scrollbar, ScrollbarOrientation, ScrollbarState};

let mut scroll_state = ScrollbarState::new(total_items).position(current_position);
let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);

f.render_stateful_widget(scrollbar, area, &mut scroll_state);
```

Pair with a `Paragraph`'s `.scroll((y, 0))` so the bar matches what's visible.

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

When built-ins don't fit, implement `Widget` for a struct. The render fn writes directly to the buffer:

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

For widgets that need mutable state across draws, implement `StatefulWidget` instead.

## Widget Selection Cheatsheet

| Need | Widget |
|---|---|
| Static text / multi-line | Paragraph |
| Selectable list | List + ListState |
| Tabular data | Table (+TableState if selectable) |
| Progress 0–100% | Gauge |
| Top-level mode tabs | Tabs |
| Container with title + border | Block |
| Show position in long content | Scrollbar |
| Time-series mini graph | Sparkline / Chart |
| Categorical counts | BarChart |
| Anything bespoke | impl Widget |
