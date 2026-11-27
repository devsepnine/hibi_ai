use super::Language;
use iced::widget::{
    Column, Space, button, checkbox, column, container, radio, row, rule, scrollable, svg, text,
};
use iced::{Element, Fill};

use super::{Message, actions::Action, styles};
use crate::app::{App, ComponentFilter, HiddenVisibility};
mod dropdown;

pub(super) use crate::app::FilterChoice as Choice;
use crate::app::{STATUS_CHOICES, VISIBILITY_CHOICES};

impl Choice {
    pub fn action(self, app: &App) -> Action {
        match self {
            Self::Status(filter) => Action::SetFilter(
                filter,
                filter == ComponentFilter::Default || !app.component_filter.contains(filter),
            ),
            Self::Visibility(mode) => Action::SetHiddenVisibility(mode),
        }
    }
}

pub(super) fn view(lang: Language, app: &App, cursor: Option<Choice>) -> Element<'_, Message> {
    let open = cursor.is_some();
    let active = STATUS_CHOICES
        .into_iter()
        .filter(|filter| {
            *filter != ComponentFilter::Default && app.component_filter.contains(*filter)
        })
        .count()
        + usize::from(app.hidden_visibility.shows_hidden());
    let label = if active == 0 {
        lang.text("Filters · Default").into()
    } else {
        localized_format!(
            lang,
            "Filters · {active}",
            "필터 · {active}",
            "フィルター · {active}"
        )
    };
    let anchor = button(
        row![text(label).size(12), Space::new().width(Fill), caret(open)]
            .spacing(8)
            .align_y(iced::Center),
    )
    .padding([9, 12])
    .width(180)
    .height(36)
    .style(if active == 0 {
        styles::secondary
    } else {
        styles::navigation
    })
    .on_press(Message::Action(Action::ToggleFilterMenu));
    let menu = container(scrollable(
        column![
            text(lang.text("Status")).size(11).style(styles::muted),
            status_choices(lang, app, cursor),
            rule::horizontal(1).style(styles::divider),
            visibility_choices(lang, app, cursor),
        ]
        .spacing(14),
    ))
    .padding(16)
    .width(260)
    .style(styles::popup)
    .id("filters-menu");
    dropdown::view(anchor.into(), menu.into(), open)
}

fn caret(open: bool) -> iced::widget::Svg<'static> {
    let data: &[u8] = if open {
        br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16"><path d="M4 10L8 6L12 10" fill="none" stroke="black" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/></svg>"#
    } else {
        br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16"><path d="M4 6L8 10L12 6" fill="none" stroke="black" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/></svg>"#
    };
    svg(svg::Handle::from_memory(data))
        .width(12)
        .height(12)
        .style(|theme: &iced::Theme, _| svg::Style {
            color: Some(theme.palette().text),
        })
}

fn status_choices(lang: Language, app: &App, cursor: Option<Choice>) -> Column<'_, Message> {
    STATUS_CHOICES
        .into_iter()
        .fold(column![].spacing(4), |choices, filter| {
            choices.push(choice_row(
                checkbox(app.component_filter.contains(filter))
                    .style(styles::check)
                    .label(lang.text(filter.display()))
                    .size(14)
                    .width(Fill)
                    .text_size(13)
                    .on_toggle(move |checked| Message::Action(Action::SetFilter(filter, checked))),
                cursor == Some(Choice::Status(filter)),
            ))
        })
}

fn visibility_choices(lang: Language, app: &App, cursor: Option<Choice>) -> Column<'_, Message> {
    VISIBILITY_CHOICES.into_iter().fold(
        column![
            text(localized_format!(
                lang,
                "Visibility · {} hidden",
                "표시 범위 · 숨긴 폴더 {}개",
                "表示範囲 · 非表示 {} 件",
                app.hidden_folder_count()
            ))
            .size(10)
            .style(styles::muted)
        ]
        .spacing(6)
        .width(Fill),
        |choices, mode| {
            let (label, id) = match mode {
                HiddenVisibility::Exclude => ("Visible only", "visibility-visible"),
                HiddenVisibility::Include => ("Include hidden", "visibility-include"),
                HiddenVisibility::Only => ("Hidden only", "visibility-only"),
            };
            choices.push(
                choice_row(
                    radio(
                        lang.text(label),
                        mode,
                        Some(app.hidden_visibility),
                        |mode| Message::Action(Action::SetHiddenVisibility(mode)),
                    )
                    .size(14)
                    .width(Fill)
                    .text_size(13)
                    .spacing(8),
                    cursor == Some(Choice::Visibility(mode)),
                )
                .id(id),
            )
        },
    )
}

fn choice_row<'a>(
    control: impl Into<Element<'a, Message>>,
    focused: bool,
) -> iced::widget::Container<'a, Message> {
    container(control)
        .width(Fill)
        .padding(4)
        .style(move |theme| {
            if focused {
                styles::focus_ring(theme)
            } else {
                Default::default()
            }
        })
}
