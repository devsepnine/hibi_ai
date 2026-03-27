use std::path::PathBuf;

use anyhow::Result;

use super::{App, View};
use crate::source::{SourceEntry, SourceKind};
use crate::source::config;

impl App {
    pub(super) fn source_input_submit(&mut self) -> Result<()> {
        match self.current_view {
            View::SourceAddUrl => self.submit_url(),
            View::SourceAddBranch => self.submit_branch(),
            View::SourceAddPath => self.submit_path(),
            View::SourceAddRoot => self.submit_root(),
            _ => Ok(()),
        }
    }

    fn submit_url(&mut self) -> Result<()> {
        let url = self.source_input_buffer.trim().to_string();
        if let Err(e) = config::validate_git_url(&url) {
            self.source_input_error = Some(e.to_string());
            return Ok(());
        }
        self.source_input_error = None;
        self.source_pending_url = url;

        self.source_input_buffer = self.editing_field(|e| match e {
            SourceEntry::Git { branch, .. } => branch.clone(),
            _ => None,
        }).unwrap_or_default();
        self.current_view = View::SourceAddBranch;
        Ok(())
    }

    fn submit_branch(&mut self) -> Result<()> {
        let branch_str = self.source_input_buffer.trim().to_string();
        let branch = if branch_str.is_empty() {
            None
        } else {
            if let Err(e) = config::validate_branch(&branch_str) {
                self.source_input_error = Some(e.to_string());
                return Ok(());
            }
            Some(branch_str)
        };

        self.source_pending_branch = branch;
        self.advance_to_root_step(|e| match e {
            SourceEntry::Git { root, .. } => root.clone(),
            _ => None,
        })
    }

    fn submit_path(&mut self) -> Result<()> {
        let path_str = self.source_input_buffer.trim().to_string();
        let path = PathBuf::from(&path_str);
        if let Err(e) = config::validate_local_path(&path) {
            self.source_input_error = Some(e.to_string());
            return Ok(());
        }

        self.source_pending_url = path_str;
        self.advance_to_root_step(|e| match e {
            SourceEntry::Local { root, .. } => root.clone(),
            _ => None,
        })
    }

    fn submit_root(&mut self) -> Result<()> {
        let root_str = self.source_input_buffer.trim().to_string();
        if root_str.contains("..") {
            self.source_input_error = Some("Path traversal (..) not allowed in root".to_string());
            return Ok(());
        }
        self.source_pending_root = if root_str.is_empty() { None } else { Some(root_str) };
        self.source_input_buffer.clear();
        self.source_input_error = None;
        self.current_view = View::SourceAddMapTo;
        Ok(())
    }

    pub fn finish_with_map_to(&mut self, map_to: Option<&str>) -> Result<()> {
        let map_to = map_to.map(|s| s.to_string());
        let root = self.source_pending_root.take();

        let entry = match self.source_add_kind {
            Some(SourceKind::Git) => SourceEntry::Git {
                url: std::mem::take(&mut self.source_pending_url),
                branch: self.source_pending_branch.take(),
                root,
                map_to,
            },
            Some(SourceKind::Local) => SourceEntry::Local {
                path: PathBuf::from(std::mem::take(&mut self.source_pending_url)),
                root,
                map_to,
            },
            _ => {
                self.source_cancel();
                return Ok(());
            }
        };

        self.save_entry(entry)?;
        Ok(())
    }

    pub(super) fn save_entry(&mut self, entry: SourceEntry) -> Result<()> {
        if let Some(idx) = self.source_edit_index {
            self.source_entries[idx] = entry;
        } else {
            self.source_entries.push(entry);
        }

        config::save_config(&self.source_entries, self.source_auto_update)?;
        self.source_edit_index = None;
        self.source_start_resolve();
        Ok(())
    }

    /// Extract a field from the entry being edited (returns None for new entries).
    fn editing_field<T: Clone>(&self, f: impl Fn(&SourceEntry) -> Option<T>) -> Option<T> {
        self.source_edit_index
            .and_then(|idx| self.source_entries.get(idx))
            .and_then(|e| f(e))
    }

    /// Advance to the Root input step, pre-filling from existing entry if editing.
    fn advance_to_root_step(&mut self, extract_root: impl Fn(&SourceEntry) -> Option<String>) -> Result<()> {
        self.source_input_error = None;
        self.source_input_buffer = self.source_edit_index
            .and_then(|idx| self.source_entries.get(idx))
            .and_then(|e| extract_root(e))
            .unwrap_or_default();
        self.current_view = View::SourceAddRoot;
        Ok(())
    }
}
