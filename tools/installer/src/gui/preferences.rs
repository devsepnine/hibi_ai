use std::sync::mpsc::{self, Receiver, TryRecvError};

use anyhow::Result;

use super::runtime::Runtime;
use crate::app::App;
use crate::fs::installer::preferences::{self, Accent, Appearance, Language, Preferences};

impl Runtime {
    pub(super) fn restore_preferences(&mut self, saved: Option<&Preferences>) {
        if self.appearance.busy() {
            return;
        }
        self.accent = saved.and_then(|p| p.accent).unwrap_or_default();
        self.theme_choice = saved.and_then(|p| p.appearance.clone()).unwrap_or_default();
        self.language = saved
            .and_then(|p| p.language)
            .unwrap_or_else(|| Language::preferred(sys_locale::get_locales()));
    }
}

#[derive(Default)]
pub(super) struct Writer {
    pending: Option<Preferences>,
    running: Option<Receiver<Result<()>>>,
}

impl Writer {
    pub fn request_theme(&mut self, choice: Appearance) {
        self.pending
            .get_or_insert_with(Preferences::default)
            .appearance = Some(choice);
    }

    pub fn request_accent(&mut self, accent: Accent) {
        self.pending.get_or_insert_with(Preferences::default).accent = Some(accent);
    }

    pub fn request_language(&mut self, language: Language) {
        self.pending
            .get_or_insert_with(Preferences::default)
            .language = Some(language);
    }

    pub fn busy(&self) -> bool {
        self.pending.is_some() || self.running.is_some()
    }

    pub fn tick(&mut self) -> Option<Result<()>> {
        let result = self.poll();
        if self.running.is_none()
            && let Some(settings) = self.pending.take()
        {
            let (tx, rx) = mpsc::channel();
            self.running = Some(rx);
            std::thread::spawn(move || {
                let _ = tx.send(preferences::save(settings));
            });
        }
        result
    }

    fn poll(&mut self) -> Option<Result<()>> {
        let rx = self.running.as_ref()?;
        let result = match rx.try_recv() {
            Ok(result) => result,
            Err(TryRecvError::Empty) => return None,
            Err(TryRecvError::Disconnected) => Err(anyhow::anyhow!(
                "Display settings worker stopped unexpectedly"
            )),
        };
        self.running = None;
        Some(result)
    }
}

pub(super) fn report_load_error(app: &mut App, error: anyhow::Error) {
    let warning = format!("Could not load display settings: {error:#}");
    let message = app.status_message.get_or_insert_with(String::new);
    if !message.is_empty() {
        message.push_str("; ");
    }
    message.push_str(&warning);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gui::actions::Action;

    #[test]
    fn retry_keeps_choices_that_are_still_being_saved() {
        let mut runtime = Runtime::empty();
        let nord = Appearance::Palette("Nord".into());
        runtime.dispatch(Action::SetAppearance(nord.clone()));
        runtime.dispatch(Action::SetLanguage(Language::Korean));
        runtime.dispatch(Action::SetAccent(Accent::Mint));
        runtime.restore_preferences(Some(&Preferences {
            appearance: Some(Appearance::Dark),
            language: Some(Language::English),
            ..Preferences::default()
        }));
        assert_eq!(runtime.theme_choice, nord);
        assert_eq!(runtime.language, Language::Korean);
        assert_eq!(runtime.accent, Accent::Mint);
    }

    #[test]
    fn rapid_theme_changes_coalesce_while_an_earlier_write_is_running() {
        let (tx, rx) = mpsc::channel();
        let mut writer = Writer {
            running: Some(rx),
            pending: None,
        };
        writer.request_language(Language::Japanese);
        writer.request_accent(Accent::Lavender);
        writer.request_theme(Appearance::Dark);
        writer.request_accent(Accent::Rose);
        writer.request_language(Language::Korean);
        writer.request_theme(Appearance::Light);
        assert!(writer.tick().is_none());
        assert_eq!(
            writer.pending,
            Some(Preferences {
                appearance: Some(Appearance::Light),
                language: Some(Language::Korean),
                accent: Some(Accent::Rose),
            })
        );
        assert!(writer.busy());
        drop(tx);
    }

    #[test]
    fn failed_or_panicked_save_settles_and_can_be_retried() {
        let (tx, rx) = mpsc::channel();
        let mut writer = Writer {
            running: Some(rx),
            pending: None,
        };
        tx.send(Err(anyhow::anyhow!("read only"))).unwrap();
        assert!(
            writer
                .poll()
                .unwrap()
                .unwrap_err()
                .to_string()
                .contains("read only")
        );
        assert!(!writer.busy());
        let (tx, rx) = mpsc::channel();
        writer.running = Some(rx);
        drop(tx);
        assert!(writer.poll().unwrap().is_err());
        assert!(!writer.busy());
    }
}
