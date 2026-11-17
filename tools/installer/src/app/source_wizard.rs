use std::path::PathBuf;

use anyhow::Result;

use super::{App, View};
use crate::source::config;
use crate::source::{SourceEntry, SourceKind};

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

        self.source_input_buffer = self
            .editing_field(|e| match e {
                SourceEntry::Git { branch, .. } => branch.clone(),
                _ => None,
            })
            .unwrap_or_default();
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
        if self.source_add_kind == Some(SourceKind::Local) {
            let base = config::expand_tilde(&PathBuf::from(&self.source_pending_url));
            let resolved = base.join(&root_str);
            let validation = (|| -> Result<()> {
                config::validate_local_path(&resolved)?;
                if !resolved.canonicalize()?.starts_with(base.canonicalize()?) {
                    anyhow::bail!("Root must be a directory inside the local source");
                }
                Ok(())
            })();
            if let Err(error) = validation {
                self.source_input_error = Some(format!("{error:#}"));
                return Ok(());
            }
        }
        self.source_pending_root = if root_str.is_empty() {
            None
        } else {
            Some(root_str)
        };
        self.source_input_buffer.clear();
        self.source_input_error = None;
        self.current_view = View::SourceAddMapTo;
        Ok(())
    }

    pub fn finish_with_map_to(&mut self, map_to: Option<&str>) -> Result<()> {
        self.finish_with_map_to_using(map_to, config::save_config)
    }

    fn finish_with_map_to_using(
        &mut self,
        map_to: Option<&str>,
        save: impl FnOnce(&[SourceEntry], bool) -> Result<()>,
    ) -> Result<()> {
        let map_to = map_to.map(str::to_string);
        let root = self.source_pending_root.clone();

        let entry = match self.source_add_kind {
            Some(SourceKind::Git) => SourceEntry::Git {
                url: self.source_pending_url.clone(),
                branch: self.source_pending_branch.clone(),
                root,
                map_to,
            },
            Some(SourceKind::Local) => SourceEntry::Local {
                path: PathBuf::from(&self.source_pending_url),
                root,
                map_to,
            },
            _ => {
                self.source_cancel();
                return Ok(());
            }
        };

        let mut entries = self.source_entries.clone();
        if let Some(index) = self.source_edit_index {
            let Some(existing) = entries.get_mut(index) else {
                anyhow::bail!("The source being edited is no longer available");
            };
            *existing = entry;
        } else {
            entries.push(entry);
        }
        if let Err(error) = save(&entries, self.source_auto_update) {
            self.source_input_error = Some(format!("Could not save source: {error:#}"));
            return Err(error);
        }
        self.source_entries = entries;
        self.source_edit_index = None;
        self.source_pending_url.clear();
        self.source_pending_branch = None;
        self.source_pending_root = None;
        self.source_sync_status = None;
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
    fn advance_to_root_step(
        &mut self,
        extract_root: impl Fn(&SourceEntry) -> Option<String>,
    ) -> Result<()> {
        self.source_input_error = None;
        self.source_input_buffer = self
            .source_edit_index
            .and_then(|idx| self.source_entries.get(idx))
            .and_then(|e| extract_root(e))
            .unwrap_or_default();
        self.current_view = View::SourceAddRoot;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::test_support::fresh_app;

    fn missing_config_path() -> PathBuf {
        let id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir()
            .join(format!("hibi_wizard_missing_{}_{id}", std::process::id()))
            .join("sources.yaml");
        assert!(!path.parent().unwrap().exists());
        path
    }

    fn editing_app() -> App {
        let mut app = fresh_app();
        app.current_view = View::SourceAddMapTo;
        app.source_add_kind = Some(SourceKind::Git);
        app.source_pending_url = "https://example.invalid/new.git".into();
        app.source_pending_branch = Some("v2".into());
        app.source_pending_root = Some("config/설정".into());
        app
    }

    #[test]
    fn failed_add_preserves_pending_fields_and_retry_does_not_duplicate() {
        let mut app = editing_app();
        let path = missing_config_path();
        for _ in 0..2 {
            let result = app.finish_with_map_to_using(Some("skills"), |entries, _| {
                assert_eq!(entries.len(), 1);
                let SourceEntry::Git {
                    url,
                    branch,
                    root,
                    map_to,
                } = &entries[0]
                else {
                    panic!("expected Git source");
                };
                assert_eq!(url, "https://example.invalid/new.git");
                assert_eq!(branch.as_deref(), Some("v2"));
                assert_eq!(root.as_deref(), Some("config/설정"));
                assert_eq!(map_to.as_deref(), Some("skills"));
                std::fs::write(&path, "sources: []")?;
                Ok(())
            });
            assert!(result.is_err());
            assert!(app.source_entries.is_empty());
            assert_eq!(app.source_pending_url, "https://example.invalid/new.git");
            assert_eq!(app.source_pending_branch.as_deref(), Some("v2"));
            assert_eq!(app.source_pending_root.as_deref(), Some("config/설정"));
            assert_eq!(app.current_view, View::SourceAddMapTo);
            assert!(app.source_input_error.is_some());
            assert!(app.source_sync_rx.is_none());
        }
    }

    #[test]
    fn failed_edit_preserves_original_entry_and_edit_index() {
        let mut app = editing_app();
        app.source_edit_index = Some(0);
        app.source_entries.push(SourceEntry::Local {
            path: "/tmp/original".into(),
            root: None,
            map_to: None,
        });
        let before = serde_json::to_value(&app.source_entries).unwrap();
        let path = missing_config_path();
        assert!(
            app.finish_with_map_to_using(None, |_, _| {
                std::fs::write(&path, "sources: []")?;
                Ok(())
            })
            .is_err()
        );
        assert_eq!(serde_json::to_value(&app.source_entries).unwrap(), before);
        assert_eq!(app.source_edit_index, Some(0));
        assert_eq!(app.current_view, View::SourceAddMapTo);
    }

    #[test]
    fn stale_edit_index_is_an_error_without_persisting_or_panicking() {
        let mut app = editing_app();
        app.source_edit_index = Some(usize::MAX);
        assert!(
            app.finish_with_map_to_using(None, |_, _| {
                panic!("stale edit must not write configuration");
            })
            .is_err()
        );
        assert!(app.source_entries.is_empty());
        assert_eq!(app.source_pending_url, "https://example.invalid/new.git");
    }
    #[test]
    fn local_root_must_be_an_existing_subdirectory_before_mapping_step() {
        let base = missing_config_path().parent().unwrap().to_path_buf();
        std::fs::create_dir(&base).unwrap();
        std::fs::write(base.join("file-root"), b"not a directory").unwrap();
        std::fs::create_dir_all(base.join("config/설정")).unwrap();
        for root in ["missing-root", "file-root"] {
            let mut app = fresh_app();
            app.current_view = View::SourceAddRoot;
            app.source_add_kind = Some(SourceKind::Local);
            app.source_pending_url = base.to_string_lossy().into();
            app.source_input_buffer = root.into();
            app.submit_root().unwrap();
            assert_eq!(app.current_view, View::SourceAddRoot);
            assert!(app.source_input_error.is_some());
            assert_eq!(app.source_input_buffer, root);
            assert!(app.source_entries.is_empty());
        }
        let mut app = fresh_app();
        app.current_view = View::SourceAddRoot;
        app.source_add_kind = Some(SourceKind::Local);
        app.source_pending_url = base.to_string_lossy().into();
        app.source_input_buffer = "config/설정".into();
        app.submit_root().unwrap();
        assert_eq!(app.current_view, View::SourceAddMapTo);
        assert_eq!(app.source_pending_root.as_deref(), Some("config/설정"));
        assert!(app.source_input_error.is_none());
        std::fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn git_root_validation_does_not_clone_or_probe_a_local_path() {
        let mut app = fresh_app();
        app.current_view = View::SourceAddRoot;
        app.source_add_kind = Some(SourceKind::Git);
        app.source_pending_url = "https://example.invalid/repository.git".into();
        app.source_input_buffer = "config/skills".into();
        app.submit_root().unwrap();
        assert_eq!(app.current_view, View::SourceAddMapTo);
        assert!(app.source_sync_rx.is_none());
    }

    #[cfg(unix)]
    #[test]
    fn local_root_symlink_cannot_escape_selected_source() {
        let base = missing_config_path().parent().unwrap().to_path_buf();
        std::fs::create_dir(&base).unwrap();
        let source = base.join("source");
        let outside = base.join("outside");
        std::fs::create_dir(&source).unwrap();
        std::fs::create_dir(&outside).unwrap();
        std::os::unix::fs::symlink(&outside, source.join("escaped")).unwrap();
        let mut app = fresh_app();
        app.current_view = View::SourceAddRoot;
        app.source_add_kind = Some(SourceKind::Local);
        app.source_pending_url = source.to_string_lossy().into();
        app.source_input_buffer = "escaped".into();
        app.submit_root().unwrap();
        assert_eq!(app.current_view, View::SourceAddRoot);
        assert!(
            app.source_input_error
                .as_deref()
                .unwrap()
                .contains("inside the local source")
        );
        std::fs::remove_dir_all(base).unwrap();
    }
}
