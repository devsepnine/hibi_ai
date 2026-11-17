use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use super::App;
use crate::tree::TreeNode;

pub(super) struct FolderState {
    selected: bool,
}

impl App {
    pub(super) fn folder_states(&self) -> HashMap<PathBuf, FolderState> {
        let mut states = HashMap::new();
        let Some(root) = self.folder_path("") else {
            return states;
        };
        let component_type = self.tab.to_component_type();
        for component in self
            .components
            .iter()
            .filter(|c| Some(&c.component_type) == component_type.as_ref())
        {
            let path = self.component_path(component);
            for folder in path
                .ancestors()
                .skip(1)
                .take_while(|folder| *folder != root && folder.starts_with(&root))
            {
                let state = states
                    .entry(folder.to_path_buf())
                    .or_insert(FolderState { selected: true });
                state.selected &= component.selected;
            }
        }
        states
    }

    pub fn selected_folders(&self) -> Vec<String> {
        let Some(tree) = self.get_tree_view() else {
            return Vec::new();
        };
        let states = self.folder_states();
        let mut covered = HashSet::new();
        let mut folders = Vec::new();
        for node in &tree.nodes {
            let TreeNode::Folder { path, .. } = node else {
                continue;
            };
            let Some(folder) = self.folder_path(path) else {
                continue;
            };
            if folder.ancestors().any(|parent| covered.contains(parent)) {
                continue;
            }
            if states.get(&folder).is_some_and(|state| state.selected) {
                covered.insert(folder);
                folders.push(path.clone());
            }
        }
        folders
    }

    pub fn hide_selected_folders(&mut self) {
        if self.get_tree_view().is_none() {
            return;
        }
        let folders = self.selected_folders();
        if folders.is_empty() {
            self.status_message = Some("No fully checked folders to hide".to_string());
            return;
        }
        self.request_folder_hide(folders);
    }

    #[cfg(feature = "gui")]
    pub fn unhide_selected_folders(&mut self) {
        let folders = self.selected_folders();
        if folders.is_empty() {
            return;
        }
        let paths: Vec<_> = folders
            .iter()
            .filter_map(|path| self.folder_path(path))
            .collect();
        if let Some(parent) = paths
            .iter()
            .flat_map(|path| path.ancestors().skip(1))
            .find(|parent| self.hidden_folders.contains(*parent))
        {
            self.status_message = Some(format!(
                "Hidden by {}. Select that parent folder to unhide it.",
                parent.display()
            ));
            return;
        }
        let mut updated = self.hidden_folders.clone();
        updated.retain(|folder| !paths.iter().any(|path| folder.starts_with(path)));
        if updated != self.hidden_folders && !self.save_folder_visibility(updated) {
            return;
        }
        self.rebuild_tree_views();
        self.status_message = Some(format!(
            "{} selected folder(s) are visible. Existing selections are unchanged.",
            folders.len()
        ));
    }
}
