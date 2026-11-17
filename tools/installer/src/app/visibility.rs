use super::{App, View};
use crate::component::Component;
use crate::tree::TreeNode;
use std::path::PathBuf;
mod folders;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HiddenVisibility {
    #[default]
    Exclude,
    Include,
    #[cfg(feature = "gui")]
    Only,
}

impl HiddenVisibility {
    pub fn shows_hidden(self) -> bool {
        self != Self::Exclude
    }

    pub fn matches(self, hidden: bool) -> bool {
        match self {
            Self::Exclude => !hidden,
            Self::Include => true,
            #[cfg(feature = "gui")]
            Self::Only => hidden,
        }
    }
}

pub struct HideConfirmation {
    pub folders: Vec<String>,
    pub selected_files: usize,
    pub scroll: usize,
}

impl App {
    pub(super) fn load_folder_visibility(&mut self) -> Option<String> {
        if self.visibility_path.as_os_str().is_empty() {
            return None;
        }
        match crate::fs::installer::visibility::load(&self.visibility_path) {
            Ok(folders) => {
                self.hidden_folders = folders;
                None
            }
            Err(error) => Some(format!("Failed to load folder visibility: {error:#}")),
        }
    }

    fn component_path(&self, component: &Component) -> PathBuf {
        self.dest_dir
            .join(component.component_type.display_name())
            .join(component.name.replace('\\', "/"))
    }

    pub fn is_component_hidden(&self, component: &Component) -> bool {
        if self.hidden_folders.is_empty() {
            return false;
        }
        let path = self.component_path(component);
        path.ancestors()
            .skip(1)
            .any(|folder| self.hidden_folders.contains(folder))
    }

    pub fn is_folder_hidden(&self, path: &str) -> bool {
        self.folder_path(path)
            .is_some_and(|folder| self.hidden_folders.contains(&folder))
    }

    fn folder_path(&self, path: &str) -> Option<PathBuf> {
        Some(
            self.dest_dir
                .join(self.tab.to_component_type()?.display_name())
                .join(path.replace('\\', "/")),
        )
    }

    pub fn hidden_folder_count(&self) -> usize {
        let Some(root) = self.folder_path("") else {
            return 0;
        };
        self.hidden_folders
            .iter()
            .filter(|folder| folder.starts_with(&root))
            .count()
    }

    pub fn hidden_selected_count(&self) -> usize {
        self.current_components()
            .iter()
            .filter(|(_, c)| c.selected && self.is_component_hidden(c))
            .count()
    }

    pub fn toggle_show_hidden(&mut self) {
        self.set_hidden_visibility(if self.hidden_visibility.shows_hidden() {
            HiddenVisibility::Exclude
        } else {
            HiddenVisibility::Include
        });
    }

    pub fn set_hidden_visibility(&mut self, visibility: HiddenVisibility) {
        if self.tab.to_component_type().is_none() || self.hidden_visibility == visibility {
            return;
        }
        self.hidden_visibility = visibility;
        if visibility == HiddenVisibility::Exclude {
            let hidden_selected: Vec<_> = self
                .components
                .iter()
                .enumerate()
                .filter(|(_, c)| c.selected && self.is_component_hidden(c))
                .map(|(index, _)| index)
                .collect();
            for index in hidden_selected {
                self.components[index].selected = false;
            }
        }
        self.rebuild_tree_views();
        self.status_message = Some(
            match visibility {
                HiddenVisibility::Include => {
                    "Showing hidden folders. Press x on a hidden folder to unhide it."
                }
                HiddenVisibility::Exclude => "Hidden folders concealed; hidden selections cleared.",
                #[cfg(feature = "gui")]
                HiddenVisibility::Only => {
                    "Showing only hidden folders. Unhide selected restores their visibility."
                }
            }
            .to_string(),
        );
    }

    pub fn toggle_folder_visibility(&mut self) {
        let Some(tree) = self.get_tree_view() else {
            return;
        };
        let path = match tree
            .current_node_idx()
            .and_then(|index| tree.nodes.get(index))
        {
            Some(TreeNode::Folder { path, .. }) => path.clone(),
            _ => {
                self.status_message = Some("Select a folder to hide or unhide".to_string());
                return;
            }
        };
        let Some(folder) = self.folder_path(&path) else {
            return;
        };
        if let Some(parent) = folder
            .ancestors()
            .skip(1)
            .find(|parent| self.hidden_folders.contains(*parent))
        {
            self.status_message = Some(format!(
                "Hidden by {}. Unhide that parent folder first.",
                parent.display()
            ));
            return;
        }
        if self.hidden_folders.contains(&folder) {
            let mut updated = self.hidden_folders.clone();
            updated.remove(&folder);
            if self.save_folder_visibility(updated) {
                self.rebuild_tree_views();
                self.status_message = Some(format!(
                    "Unhidden {path}. Existing selections are unchanged."
                ));
            }
            return;
        }
        self.request_folder_hide(vec![path]);
    }

    fn request_folder_hide(&mut self, folders: Vec<String>) {
        let selected_files = self
            .folder_component_indices(&folders)
            .iter()
            .filter(|&&index| self.components[index].selected)
            .count();
        self.hide_confirmation = Some(HideConfirmation {
            folders,
            selected_files,
            scroll: 0,
        });
        self.current_view = View::ConfirmHide;
    }

    pub fn cancel_folder_hide(&mut self) {
        self.hide_confirmation = None;
        self.current_view = View::List;
    }

    pub fn confirm_folder_hide(&mut self) {
        let Some(pending) = self.hide_confirmation.take() else {
            return;
        };
        self.current_view = View::List;
        let mut updated = self.hidden_folders.clone();
        let states = self.folder_states();
        for path in &pending.folders {
            let Some(folder) = self.folder_path(path) else {
                return;
            };
            if !states.contains_key(&folder) {
                self.status_message =
                    Some("Folder contents changed. Review the folders before hiding.".to_string());
                return;
            }
            updated.insert(folder);
        }
        let indices = self.folder_component_indices(&pending.folders);
        if !self.save_folder_visibility(updated) {
            return;
        }
        for index in indices {
            self.components[index].selected = false;
        }
        self.rebuild_tree_views();
        self.status_message = Some(format!(
            "Hidden {} folder(s); selections cleared. H shows hidden folders.",
            pending.folders.len()
        ));
    }

    pub fn scroll_folder_hide_down(&mut self, max: usize) {
        let Some(pending) = &mut self.hide_confirmation else {
            return;
        };
        pending.scroll = pending.scroll.saturating_add(1).min(max);
    }

    pub fn scroll_folder_hide_up(&mut self, max: usize) {
        if let Some(pending) = &mut self.hide_confirmation {
            pending.scroll = pending.scroll.min(max).saturating_sub(1);
        }
    }

    fn save_folder_visibility(&mut self, updated: std::collections::BTreeSet<PathBuf>) -> bool {
        match crate::fs::installer::visibility::save(
            &self.visibility_path,
            &self.hidden_folders,
            &updated,
        ) {
            Ok(()) => {
                self.hidden_folders = updated;
                true
            }
            Err(error) => {
                self.status_message = Some(format!("Failed to save folder visibility: {error:#}"));
                false
            }
        }
    }

    fn folder_component_indices(&self, folders: &[String]) -> Vec<usize> {
        let paths: std::collections::HashSet<_> = folders
            .iter()
            .filter_map(|path| self.folder_path(path))
            .collect();
        self.components
            .iter()
            .enumerate()
            .filter(|(_, c)| {
                let path = self.component_path(c);
                path.ancestors()
                    .skip(1)
                    .any(|folder| paths.contains(folder))
            })
            .map(|(index, _)| index)
            .collect()
    }
}
