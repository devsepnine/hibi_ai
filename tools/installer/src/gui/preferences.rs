use std::sync::mpsc::{self, Receiver, TryRecvError};

use anyhow::Result;

use crate::app::App;
use crate::fs::installer::preferences::{self, Appearance, Language, Preferences};
use crate::theme::ThemeMode;

#[derive(Default)]
pub(super) struct Writer {
    pending: Option<Preferences>,
    running: Option<Receiver<Result<()>>>,
}

impl Writer {
    pub fn request_theme(&mut self, mode: ThemeMode) {
        self.pending
            .get_or_insert_with(Preferences::default)
            .appearance = Some(match mode {
            ThemeMode::Mocha => Appearance::Dark,
            ThemeMode::Latte => Appearance::Light,
        });
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

pub(super) fn restore(app: &mut App, settings: Result<Preferences>) {
    match settings {
        Ok(settings) => {
            if let Some(appearance) = settings.appearance
                && (appearance == Appearance::Dark) != (app.theme.mode() == ThemeMode::Mocha)
            {
                app.theme.toggle();
            }
        }
        Err(error) => {
            let warning = format!("Could not load display settings: {error:#}");
            let message = app.status_message.get_or_insert_with(String::new);
            if !message.is_empty() {
                message.push_str("; ");
            }
            message.push_str(&warning);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rapid_theme_changes_coalesce_while_an_earlier_write_is_running() {
        let (tx, rx) = mpsc::channel();
        let mut writer = Writer {
            running: Some(rx),
            pending: None,
        };
        writer.request_language(Language::Japanese);
        writer.request_theme(ThemeMode::Mocha);
        writer.request_language(Language::Korean);
        writer.request_theme(ThemeMode::Latte);
        assert!(writer.tick().is_none());
        assert_eq!(
            writer.pending,
            Some(Preferences {
                appearance: Some(Appearance::Light),
                language: Some(Language::Korean)
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
