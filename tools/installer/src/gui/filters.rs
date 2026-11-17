use iced::widget::{
    Column, Space, button, checkbox, column, container, radio, row, rule, scrollable, svg, text,
};
use iced::{Element, Fill};

use super::{Message, actions::Action, styles};
use crate::app::{App, ComponentFilter, HiddenVisibility};
mod dropdown;

pub(super) fn view(app: &App, open: bool) -> Element<'_, Message> {
    let active = [
        ComponentFilter::Modified,
        ComponentFilter::External,
        ComponentFilter::New,
    ]
    .into_iter()
    .filter(|filter| app.component_filter.contains(*filter))
    .count()
        + usize::from(app.hidden_visibility.shows_hidden());
    let label = if active == 0 {
        "Filters · Default".into()
    } else {
        format!("Filters · {active}")
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
            text("Status").size(11).style(styles::muted),
            status_choices(app),
            rule::horizontal(1).style(styles::divider),
            visibility_choices(app),
        ]
        .spacing(14),
    ))
    .padding(16)
    .width(260)
    .style(styles::surface)
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

fn status_choices(app: &App) -> Column<'_, Message> {
    [
        ComponentFilter::All,
        ComponentFilter::Modified,
        ComponentFilter::External,
        ComponentFilter::New,
    ]
    .into_iter()
    .fold(column![].spacing(12), |choices, filter| {
        choices.push(
            checkbox(app.component_filter.contains(filter))
                .style(styles::check)
                .label(if filter == ComponentFilter::All {
                    "Default"
                } else {
                    filter.display()
                })
                .size(14)
                .text_size(13)
                .on_toggle(move |checked| Message::Action(Action::SetFilter(filter, checked))),
        )
    })
}

fn visibility_choices(app: &App) -> Column<'_, Message> {
    [
        (
            HiddenVisibility::Exclude,
            "Visible only",
            "visibility-visible",
        ),
        (
            HiddenVisibility::Include,
            "Include hidden",
            "visibility-include",
        ),
        (HiddenVisibility::Only, "Hidden only", "visibility-only"),
    ]
    .into_iter()
    .fold(
        column![
            text(format!("Visibility · {} hidden", app.hidden_folder_count()))
                .size(10)
                .style(styles::muted)
        ]
        .spacing(6)
        .width(Fill),
        |choices, (mode, label, id)| {
            choices.push(
                container(
                    radio(label, mode, Some(app.hidden_visibility), |mode| {
                        Message::Action(Action::SetHiddenVisibility(mode))
                    })
                    .size(14)
                    .text_size(13)
                    .spacing(8),
                )
                .id(id),
            )
        },
    )
}
