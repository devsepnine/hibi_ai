use super::Language;
use iced::widget::{Column, Space, column, container, progress_bar, row, scrollable, text};
use iced::{Element, Fill};

use super::{Message, actions::Action, dialogs::neutral, fonts, styles};
use crate::app::{App, View};

pub(super) struct LogScroll {
    pub following: bool,
}

impl Default for LogScroll {
    fn default() -> Self {
        Self { following: true }
    }
}

impl LogScroll {
    pub fn update(
        &mut self,
        viewport: scrollable::Viewport,
    ) -> Option<impl iced::advanced::widget::Operation<Message> + use<>> {
        let offset = viewport.absolute_offset();
        let reversed = viewport.absolute_offset_reversed();
        let distance_to_bottom = if self.following { offset.y } else { reversed.y };
        let following = distance_to_bottom <= 1.0;
        if self.following == following {
            return None;
        }
        self.following = following;
        Some(iced::advanced::widget::operation::scrollable::scroll_to(
            "operation-log".into(),
            iced::widget::operation::AbsoluteOffset {
                x: None,
                y: Some(if following { 0.0 } else { reversed.y }),
            },
        ))
    }
}

pub(super) fn diff(lang: Language, app: &App) -> Element<'_, Message> {
    let lines = app
        .diff_content
        .as_deref()
        .unwrap_or(lang.text("No differences available."))
        .lines()
        .fold(Column::new().spacing(3), |lines, line| {
            let style = if line.starts_with('+') {
                text::success
            } else if line.starts_with('-') {
                text::danger
            } else {
                text::default
            };
            lines.push(text(line).font(fonts::monospace()).size(13).style(style))
        });
    column![
        row![
            text(lang.text("Compare configuration")).size(24),
            Space::new().width(Fill),
            neutral(lang.text("Back to components"), Action::Back)
        ]
        .spacing(12)
        .align_y(iced::Center),
        text(lang.text("Source content compared with the installed version."))
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
        .padding(16)
        .style(styles::surface)
        .height(Fill),
    ]
    .spacing(16)
    .height(Fill)
    .into()
}

pub(super) fn progress(lang: Language, app: &App, follow_log: bool) -> Element<'_, Message> {
    let total = app.processing_total.unwrap_or(app.processing_queue.len());
    let attempted = app.processing_progress.unwrap_or(0).min(total);
    let failed = app
        .processing_log
        .iter()
        .filter(|line| line.starts_with("[ERR]") || line.starts_with("[ERROR]"))
        .count();
    let title = if app.current_view == View::Preflighting {
        lang.text("Checking requirements")
    } else if app.cancelling {
        lang.text("Cancelling operation")
    } else if app.refreshing {
        lang.text("Refreshing installed state")
    } else if app.processing_complete
        && app
            .processing_log
            .iter()
            .any(|line| line.contains("Cancelled by user"))
    {
        lang.text("Operation cancelled")
    } else if app.processing_complete && failed > 0 {
        lang.text("Completed with errors")
    } else if app.processing_complete {
        lang.text("Operation complete")
    } else if app.is_removing {
        lang.text("Removing components")
    } else {
        lang.text("Installing components")
    };
    let subtitle = if app.current_view == View::Preflighting {
        lang.text("Checking that your environment is ready. No components have been changed yet.")
            .to_string()
    } else if let Some(message) = app
        .status_message
        .as_deref()
        .filter(|_| app.processing_complete)
    {
        lang.notice(message).into_owned()
    } else {
        localized_format!(
            lang,
            "{attempted} of {total} items attempted · {failed} errors",
            "전체 {total}개 중 {attempted}개 처리 · 오류 {failed}개",
            "全{total}件中{attempted}件を処理 · エラー{failed}件"
        )
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
            column.push(text(line).font(fonts::monospace()).size(12).style(style))
        });
    let mut body = column![
        text(title).size(24),
        text(subtitle).size(14).style(styles::muted),
    ]
    .spacing(14);
    if app.current_view != View::Preflighting {
        body = body.push(progress_bar(0.0..=total.max(1) as f32, attempted as f32).girth(5));
    }
    let action_button = if app.processing_complete {
        neutral(lang.text("Back to components"), Action::Back)
    } else {
        neutral(lang.text("Cancel operation"), Action::Cancel)
    };
    body.push(
        container(
            column![
                row![
                    text(lang.text("Operation log")).size(16),
                    Space::new().width(Fill),
                    text(if app.processing_complete {
                        lang.text("Finished")
                    } else {
                        lang.text("Live")
                    })
                    .size(11)
                    .style(styles::muted)
                ]
                .align_y(iced::Center),
                scrollable(logs)
                    .id("operation-log")
                    .width(Fill)
                    .direction(scrollable::Direction::Both {
                        vertical: Default::default(),
                        horizontal: Default::default()
                    })
                    .anchor_y(if follow_log {
                        scrollable::Anchor::End
                    } else {
                        scrollable::Anchor::Start
                    })
                    .on_scroll(Message::OperationLogScrolled)
                    .height(Fill),
            ]
            .spacing(18),
        )
        .padding(16)
        .style(styles::surface)
        .width(Fill)
        .height(Fill),
    )
    .push(
        row![
            text(if app.processing_complete {
                lang.text("Review the log before leaving this screen.")
            } else {
                lang.text("Cancelling waits for the current operation to stop safely.")
            })
            .size(12)
            .style(styles::muted)
            .width(Fill),
            action_button,
        ]
        .spacing(14)
        .align_y(iced::Center),
    )
    .width(Fill)
    .height(Fill)
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::advanced::widget::{Id, Operation, Tree, operation::Scrollable};
    use iced::advanced::{Layout, Shell, clipboard, layout, mouse};
    use iced::{Event, Font, Pixels, Rectangle, Renderer, Size, Vector};

    #[derive(Default)]
    struct Observe(Option<(Rectangle, Rectangle, Vector)>);

    impl Operation<Message> for Observe {
        fn traverse(&mut self, visit: &mut dyn FnMut(&mut dyn Operation<Message>)) {
            visit(self);
        }

        fn scrollable(
            &mut self,
            _: Option<&Id>,
            bounds: Rectangle,
            content: Rectangle,
            translation: Vector,
            _: &mut dyn Scrollable,
        ) {
            self.0 = Some((bounds, content, translation));
        }
    }

    struct Desktop {
        app: App,
        scroll: LogScroll,
        tree: Tree,
        renderer: Renderer,
    }

    impl Desktop {
        fn new() -> Self {
            let mut app = crate::app::test_support::fresh_app();
            app.current_view = View::Installing;
            app.processing_log = (0..80)
                .map(|index| format!("[OK] {index} {}", "long path/".repeat(30)))
                .collect();
            let tree = Tree::new(progress(Language::English, &app, true).as_widget());
            Self {
                app,
                scroll: LogScroll::default(),
                tree,
                renderer: Renderer::new(Font::default(), Pixels(16.0)),
            }
        }

        fn operate(&mut self, operation: &mut dyn Operation<Message>) {
            let mut element = progress(Language::English, &self.app, self.scroll.following);
            self.tree.diff(element.as_widget());
            let node = element.as_widget_mut().layout(
                &mut self.tree,
                &self.renderer,
                &layout::Limits::new(Size::ZERO, Size::new(720.0, 640.0)),
            );
            element.as_widget_mut().operate(
                &mut self.tree,
                Layout::new(&node),
                &self.renderer,
                &mut iced::advanced::widget::operation::black_box(operation),
            );
        }

        fn observe(&mut self) -> (Rectangle, Rectangle, Vector) {
            let mut operation = Observe::default();
            self.operate(&mut operation);
            operation.0.unwrap()
        }

        fn wheel(&mut self, x: f32, y: f32) {
            let (bounds, _, _) = self.observe();
            let mut element = progress(Language::English, &self.app, self.scroll.following);
            let node = element.as_widget_mut().layout(
                &mut self.tree,
                &self.renderer,
                &layout::Limits::new(Size::ZERO, Size::new(720.0, 640.0)),
            );
            let mut messages = Vec::new();
            element.as_widget_mut().update(
                &mut self.tree,
                &Event::Mouse(mouse::Event::WheelScrolled {
                    delta: mouse::ScrollDelta::Pixels { x, y },
                }),
                Layout::new(&node),
                mouse::Cursor::Available(bounds.center()),
                &self.renderer,
                &mut clipboard::Null,
                &mut Shell::new(&mut messages),
                &Rectangle::with_size(Size::new(720.0, 640.0)),
            );
            drop(element);
            assert!(!messages.is_empty());
            for message in messages {
                if let Message::OperationLogScrolled(viewport) = message
                    && let Some(mut operation) = self.scroll.update(viewport)
                {
                    self.operate(&mut operation);
                }
            }
        }
    }

    #[test]
    fn scrolling_up_preserves_the_reading_position_until_returning_to_bottom() {
        let mut desktop = Desktop::new();
        let (bounds, content, bottom) = desktop.observe();
        assert!((bottom.y - (content.height - bounds.height)).abs() < 1.0);
        desktop.wheel(0.0, 120.0);
        assert!(!desktop.scroll.following);
        let (_, _, paused) = desktop.observe();
        assert!((paused.y - (bottom.y - 120.0)).abs() < 1.0);

        desktop
            .app
            .processing_log
            .extend((80..120).map(|index| format!("[OK] {index}")));
        let (_, _, appended) = desktop.observe();
        assert_eq!(appended.y, paused.y);
        desktop.app.processing_complete = true;
        assert_eq!(desktop.observe().2.y, paused.y);

        desktop.wheel(0.0, -100_000.0);
        assert!(desktop.scroll.following);
        desktop.app.processing_log.push("[OK] Finished".into());
        let (bounds, content, resumed) = desktop.observe();
        assert!((resumed.y - (content.height - bounds.height)).abs() < 1.0);
    }

    #[test]
    fn horizontal_scrolling_is_preserved_when_following_is_paused_and_resumed() {
        let mut desktop = Desktop::new();
        desktop.wheel(-100.0, 0.0);
        assert!(desktop.scroll.following);
        let horizontal = desktop.observe().2.x;
        assert_eq!(horizontal, 100.0);
        desktop.wheel(0.0, 120.0);
        assert_eq!(desktop.observe().2.x, horizontal);
        desktop.wheel(0.0, -100_000.0);
        assert_eq!(desktop.observe().2.x, horizontal);
    }

    #[test]
    fn leaving_the_operation_resets_following_for_the_next_operation() {
        let mut runtime = super::super::runtime::Runtime::empty();
        let mut app = crate::app::test_support::fresh_app();
        app.current_view = View::Installing;
        app.processing_complete = true;
        runtime.app = Some(app);
        runtime.operation_log.following = false;
        let _ = super::super::update(&mut runtime, Message::Action(Action::Back));
        assert_eq!(runtime.app.unwrap().current_view, View::List);
        assert!(runtime.operation_log.following);
    }
}
