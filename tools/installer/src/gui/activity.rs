use iced::widget::{Column, Space, column, container, progress_bar, row, scrollable, text};
use iced::{Element, Fill};

use super::{Message, actions::Action, dialogs::neutral, styles};
use crate::app::{App, View};

pub(super) fn diff(app: &App) -> Element<'_, Message> {
    let lines = app
        .diff_content
        .as_deref()
        .unwrap_or("No differences available.")
        .lines()
        .fold(Column::new().spacing(3), |lines, line| {
            let style = if line.starts_with('+') {
                text::success
            } else if line.starts_with('-') {
                text::danger
            } else {
                text::default
            };
            lines.push(text(line).font(iced::Font::MONOSPACE).size(13).style(style))
        });
    column![
        text("REVIEW CHANGES").size(11).style(styles::muted),
        row![
            text("Compare configuration").size(26),
            Space::new().width(Fill),
            neutral("Back to components", Action::Back)
        ]
        .spacing(12)
        .align_y(iced::Center),
        text("Source content compared with the installed version.")
            .size(14)
            .style(styles::muted),
        container(
            scrollable(lines)
                .id("diff")
                .direction(scrollable::Direction::Both {
                    vertical: Default::default(),
                    horizontal: Default::default()
                })
                .height(Fill)
        )
        .padding(22)
        .style(styles::surface)
        .height(Fill),
    ]
    .spacing(16)
    .height(Fill)
    .into()
}

pub(super) fn progress(app: &App) -> Element<'_, Message> {
    let total = app.processing_total.unwrap_or(app.processing_queue.len());
    let attempted = app.processing_progress.unwrap_or(0).min(total);
    let failed = app
        .processing_log
        .iter()
        .filter(|line| line.starts_with("[ERR]") || line.starts_with("[ERROR]"))
        .count();
    let title = if app.current_view == View::Preflighting {
        "Checking requirements"
    } else if app.cancelling {
        "Cancelling and cleaning up"
    } else if app.refreshing {
        "Refreshing installed state"
    } else if app.processing_complete
        && app
            .processing_log
            .iter()
            .any(|line| line.contains("Cancelled by user"))
    {
        "Operation cancelled"
    } else if app.processing_complete && failed > 0 {
        "Completed with errors"
    } else if app.processing_complete {
        "Operation complete"
    } else if app.is_removing {
        "Removing components"
    } else {
        "Installing components"
    };
    let subtitle = if app.current_view == View::Preflighting {
        "Checking that your environment is ready. No components have been changed yet.".to_string()
    } else if app.processing_complete {
        app.status_message
            .clone()
            .unwrap_or_else(|| format!("{attempted} of {total} items attempted · {failed} errors"))
    } else {
        format!("{attempted} of {total} items attempted · {failed} errors")
    };
    let logs = app
        .processing_log
        .iter()
        .fold(Column::new().spacing(7), |column, line| {
            let style = if line.starts_with("[ERR") {
                text::danger
            } else if line.starts_with("[WARN]") {
                text::warning
            } else {
                text::default
            };
            column.push(text(line).font(iced::Font::MONOSPACE).size(12).style(style))
        });
    let mut body = column![
        text("ACTIVITY").size(11).style(styles::muted),
        text(title).size(28),
        text(subtitle).size(14).style(styles::muted),
    ]
    .spacing(14);
    if app.current_view != View::Preflighting {
        body = body.push(progress_bar(0.0..=total.max(1) as f32, attempted as f32).girth(5));
    }
    body.push(
        container(
            column![
                row![
                    text("Operation log").size(16),
                    Space::new().width(Fill),
                    text(if app.processing_complete {
                        "Finished"
                    } else {
                        "Live"
                    })
                    .size(11)
                    .style(styles::muted)
                ]
                .align_y(iced::Center),
                scrollable(logs)
                    .direction(scrollable::Direction::Both {
                        vertical: Default::default(),
                        horizontal: Default::default()
                    })
                    .height(Fill),
            ]
            .spacing(18),
        )
        .padding(22)
        .style(styles::surface)
        .height(Fill),
    )
    .push(
        row![
            text(if app.processing_complete {
                "Review the log before leaving this screen."
            } else {
                "Cancelling waits for the current operation to stop safely."
            })
            .size(12)
            .style(styles::muted)
            .width(Fill),
            neutral(
                if app.processing_complete {
                    "Back to components"
                } else {
                    "Cancel operation"
                },
                if app.processing_complete {
                    Action::Back
                } else {
                    Action::Cancel
                }
            ),
        ]
        .spacing(14)
        .align_y(iced::Center),
    )
    .height(Fill)
    .into()
}
