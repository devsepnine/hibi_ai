use std::sync::mpsc;
use std::thread;

use anyhow::Result;
use crossterm::event::KeyCode;

use super::{App, View};
use crate::source::{self, SourceEntry, SourceKind};
use crate::source::{config, git};

impl App {
    /// Handle key input on the Sources list view.
    pub fn handle_sources_key(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.current_view = View::CliSelection;
                self.source_sync_status = None;
            }
            KeyCode::Up | KeyCode::Char('k') => self.source_nav_up(),
            KeyCode::Down | KeyCode::Char('j') => self.source_nav_down(),
            KeyCode::Char('a') => self.source_start_add(),
            KeyCode::Char('e') => self.source_start_edit(),
            KeyCode::Char('r') => self.source_start_remove(),
            KeyCode::Char('f') => self.source_start_sync(),
            _ => {}
        }
        Ok(())
    }

    /// Handle key input on the Add Source type selection.
    pub fn handle_source_type_key(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Char('1') => {
                self.source_add_kind = Some(SourceKind::Git);
                self.source_input_buffer.clear();
                self.source_input_error = None;
                self.current_view = View::SourceAddUrl;
            }
            KeyCode::Char('2') => {
                self.source_add_kind = Some(SourceKind::Local);
                self.source_input_buffer.clear();
                self.source_input_error = None;
                self.current_view = View::SourceAddPath;
            }
            KeyCode::Esc => self.source_cancel(),
            _ => {}
        }
        Ok(())
    }

    /// Handle key input for text input views (URL, Branch, Path, Root).
    pub fn handle_source_input_key(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Esc => self.source_cancel(),
            KeyCode::Enter => self.source_input_submit()?,
            KeyCode::Backspace => { self.source_input_buffer.pop(); }
            KeyCode::Char(c) => self.source_input_buffer.push(c),
            _ => {}
        }
        Ok(())
    }

    /// Handle key input for map_to selection view.
    pub fn handle_source_map_to_key(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Char('1') => self.finish_with_map_to(Some("agents")),
            KeyCode::Char('2') => self.finish_with_map_to(Some("commands")),
            KeyCode::Char('3') => self.finish_with_map_to(Some("contexts")),
            KeyCode::Char('4') => self.finish_with_map_to(Some("rules")),
            KeyCode::Char('5') => self.finish_with_map_to(Some("skills")),
            KeyCode::Char('6') => self.finish_with_map_to(Some("hooks")),
            KeyCode::Char('7') => self.finish_with_map_to(Some("output-styles")),
            KeyCode::Enter => self.finish_with_map_to(None),
            KeyCode::Esc => { self.source_cancel(); Ok(()) }
            _ => Ok(()),
        }
    }

    /// Handle key input on the removal confirmation dialog.
    pub fn handle_source_confirm_key(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Char('y') => {
                let entry_idx = self.source_list_index.saturating_sub(1);
                if entry_idx < self.source_entries.len() {
                    // Clean up git cache before removing the entry
                    if let SourceEntry::Git { url, .. } = &self.source_entries[entry_idx]
                        && let Err(e) = git::remove_cache(url)
                    {
                        self.source_sync_status = Some(format!(
                            "Source removed, but cache cleanup failed: {}", e
                        ));
                    }
                    self.source_entries.remove(entry_idx);
                    config::save_config(&self.source_entries, self.source_auto_update)?;
                    if self.source_list_index > self.source_entries.len() {
                        self.source_list_index = self.source_entries.len();
                    }
                    self.source_reload()?;
                }
                self.current_view = View::Sources;
            }
            KeyCode::Esc | KeyCode::Char('n') => {
                self.current_view = View::Sources;
            }
            _ => {}
        }
        Ok(())
    }

    fn source_nav_up(&mut self) {
        if self.source_list_index > 0 {
            self.source_list_index -= 1;
        }
    }

    fn source_nav_down(&mut self) {
        if self.source_list_index < self.source_entries.len() {
            self.source_list_index += 1;
        }
    }

    fn source_start_add(&mut self) {
        self.source_edit_index = None;
        self.source_input_buffer.clear();
        self.source_input_error = None;
        self.current_view = View::SourceAddType;
    }

    fn source_start_edit(&mut self) {
        if self.source_list_index == 0 {
            return;
        }
        let entry_idx = self.source_list_index - 1;
        if entry_idx >= self.source_entries.len() {
            return;
        }

        self.source_edit_index = Some(entry_idx);
        self.source_input_error = None;

        match &self.source_entries[entry_idx] {
            SourceEntry::Git { url, .. } => {
                self.source_add_kind = Some(SourceKind::Git);
                self.source_input_buffer = url.clone();
                self.current_view = View::SourceAddUrl;
            }
            SourceEntry::Local { path, .. } => {
                self.source_add_kind = Some(SourceKind::Local);
                self.source_input_buffer = path.to_string_lossy().to_string();
                self.current_view = View::SourceAddPath;
            }
        }
    }

    fn source_start_remove(&mut self) {
        if self.source_list_index == 0 || self.source_list_index > self.source_entries.len() {
            return;
        }
        self.current_view = View::SourceConfirmRemove;
    }

    fn source_start_sync(&mut self) {
        let has_git = self.sources.iter().any(|s| s.kind == SourceKind::Git);
        if !has_git {
            self.source_sync_status = Some("No git sources to sync".to_string());
            return;
        }

        let sources = self.sources.clone();
        let (tx, rx) = mpsc::channel();
        self.source_sync_rx = Some(rx);
        self.current_view = View::SourceSyncing;

        thread::spawn(move || {
            let (updated, summaries) = source::update_git_sources(&sources);
            let _ = tx.send((updated, summaries));
        });
    }

    pub(crate) fn source_start_resolve(&mut self) {
        let source_dir = self.source_dir.clone();
        let (tx, rx) = mpsc::channel();
        self.source_sync_rx = Some(rx);
        self.current_view = View::SourceSyncing;

        thread::spawn(move || {
            let result = source::resolve_all_sources(&source_dir);
            let (resolved, warnings) = match result {
                Ok(r) => (r.sources, r.warnings),
                Err(_) => (vec![source::ResolvedSource::bundled(&source_dir)], Vec::new()),
            };
            let _ = tx.send((resolved, warnings));
        });
    }

    fn source_reload(&mut self) -> Result<()> {
        let result = source::resolve_all_sources(&self.source_dir)?;
        self.sources = result.sources;
        if !result.warnings.is_empty() {
            self.source_sync_status = Some(result.warnings.join("; "));
        }
        Ok(())
    }

    pub(crate) fn source_cancel(&mut self) {
        self.source_input_buffer.clear();
        self.source_input_error = None;
        self.source_edit_index = None;
        self.source_add_kind = None;
        self.current_view = View::Sources;
    }

    /// Called from the syncing view tick to check completion.
    pub fn check_source_sync(&mut self) {
        let rx = match self.source_sync_rx.take() {
            Some(rx) => rx,
            None => return,
        };

        match rx.try_recv() {
            Ok((resolved, summaries)) => {
                self.sources = resolved;
                self.source_sync_status = if summaries.is_empty() {
                    Some("Resolved sources".to_string())
                } else {
                    Some(summaries.join("; "))
                };
                self.current_view = View::Sources;
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                self.source_sync_rx = Some(rx);
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                self.source_sync_status = Some("Sync thread crashed".to_string());
                self.current_view = View::Sources;
            }
        }
    }
}
