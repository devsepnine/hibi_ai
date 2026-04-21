use std::sync::mpsc;
use std::thread;

use anyhow::Result;
use crossterm::event::KeyCode;

use super::{App, View, SyncStatus, build_tree_views};
use crate::component::Component;
use crate::fs::scanner;
use crate::mcp::McpServer;
use crate::plugin::Plugin;
use crate::source::{self, SourceEntry, SourceKind, ResolvedSource};
use crate::source::{config, git};

/// Post-sync component rescan output; applied back into App state so the UI
/// reflects files newly produced/removed by the sync.
pub(crate) struct RescanResult {
    pub components: Vec<Component>,
    pub mcp_servers: Vec<McpServer>,
    pub plugins: Vec<Plugin>,
}

/// Message sent from the sync/resolve background thread back to the UI.
pub(crate) struct SyncPayload {
    pub resolved: Vec<ResolvedSource>,
    pub summaries: Vec<String>,
    pub had_error: bool,
    pub rescan: Option<RescanResult>,
}

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
                    if let SourceEntry::Git { url, .. } = &self.source_entries[entry_idx]
                        && let Err(e) = git::remove_cache(url)
                    {
                        self.source_sync_status = Some(SyncStatus::Error(
                            format!("Source removed, but cache cleanup failed: {}", e)
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
        let bundled_root = self.bundled_git_root.clone();
        let source_dir = self.source_dir.clone();
        let dest_dir = self.dest_dir.clone();
        let target_cli = self.target_cli;
        let (result_tx, result_rx) = mpsc::channel::<SyncPayload>();
        let (cancel_tx, cancel_rx) = mpsc::channel::<()>();
        self.source_sync_rx = Some(result_rx);
        self.source_sync_cancel_tx = Some(cancel_tx);
        self.current_view = View::SourceSyncing;

        thread::spawn(move || {
            let report = source::sync_all_sources(bundled_root.as_deref(), &source_dir, &cancel_rx);

            // Rescan component inventory so freshly pulled files appear in the
            // UI. Skip when no CLI target is selected yet (nothing to scan for).
            let rescan = target_cli.and_then(|cli| {
                let components = scanner::scan_all_sources(&report.resolved, &dest_dir, cli).ok()?;
                let mcp_servers = scanner::scan_all_mcp_sources(&report.resolved, cli)
                    .map(|(servers, _)| servers)
                    .unwrap_or_default();
                let plugins = scanner::scan_all_plugin_sources(&report.resolved).unwrap_or_default();
                Some(RescanResult { components, mcp_servers, plugins })
            });

            let _ = result_tx.send(SyncPayload {
                resolved: report.resolved,
                summaries: report.summaries,
                had_error: report.had_error,
                rescan,
            });
        });
    }

    pub(crate) fn source_start_resolve(&mut self) {
        let source_dir = self.source_dir.clone();
        let (tx, rx) = mpsc::channel::<SyncPayload>();
        self.source_sync_rx = Some(rx);
        self.current_view = View::SourceSyncing;

        thread::spawn(move || {
            let result = source::resolve_all_sources(&source_dir);
            let (resolved, warnings) = match result {
                Ok(r) => (r.sources, r.warnings),
                Err(_) => (vec![source::ResolvedSource::bundled(&source_dir)], Vec::new()),
            };
            let _ = tx.send(SyncPayload {
                resolved,
                summaries: warnings,
                had_error: false,
                rescan: None,
            });
        });
    }

    fn source_reload(&mut self) -> Result<()> {
        let result = source::resolve_all_sources(&self.source_dir)?;
        self.sources = result.sources;
        if !result.warnings.is_empty() {
            self.source_sync_status = Some(SyncStatus::Error(result.warnings.join("; ")));
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
            Ok(payload) => {
                self.sources = payload.resolved;
                self.source_sync_cancel_tx = None;
                self.source_sync_status = Some(if payload.summaries.is_empty() {
                    SyncStatus::Success("Resolved sources".to_string())
                } else if payload.had_error {
                    SyncStatus::Error(payload.summaries.join("; "))
                } else {
                    SyncStatus::Success(payload.summaries.join("; "))
                });

                // Apply rescan so the component list and tree views reflect
                // whatever the sync pulled (new files, deletions, edits).
                if let Some(rescan) = payload.rescan {
                    self.components = rescan.components;
                    self.mcp_servers = rescan.mcp_servers;
                    self.plugins = rescan.plugins;
                    self.tree_views = build_tree_views(&self.components);
                }

                self.current_view = View::Sources;
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                self.source_sync_rx = Some(rx);
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                self.source_sync_cancel_tx = None;
                self.source_sync_status = Some(SyncStatus::Error("Sync thread crashed".to_string()));
                self.current_view = View::Sources;
            }
        }
    }
}
