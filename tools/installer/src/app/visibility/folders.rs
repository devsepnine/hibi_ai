use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use super::App;
use crate::component::InstallStatus;
use crate::tree::TreeNode;

pub(super) struct FolderState {
    pub(super) external: bool,
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
                let state = states.entry(folder.to_path_buf()).or_insert(FolderState {
                    external: true,
                    selected: true,
                });
                state.external &= component.status == InstallStatus::External;
                state.selected &= component.selected;
            }
        }
        states
    }

    pub fn hide_selected_folders(&mut self) {
        let Some(tree) = self.get_tree_view() else {
            return;
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
            if self
                .hidden_folders
                .iter()
                .any(|hidden| folder.starts_with(hidden) || hidden.starts_with(&folder))
            {
                continue;
            }
            if states
                .get(&folder)
                .is_some_and(|state| state.external && state.selected)
            {
                covered.insert(folder);
                folders.push(path.clone());
            }
        }
        if folders.is_empty() {
            self.status_message = Some("No fully checked external folders to hide".to_string());
            return;
        }
        self.request_folder_hide(folders);
    }
}
