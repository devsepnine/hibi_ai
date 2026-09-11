use crate::component::Component;

use super::{TreeNode, TreeView};

impl TreeView {
    /// Get all component indices under a folder (recursive)
    pub fn get_folder_component_indices(&self, folder_idx: usize) -> Vec<usize> {
        let mut indices = Vec::new();
        self.collect_component_indices(folder_idx, &mut indices);
        indices
    }

    fn collect_component_indices(&self, node_idx: usize, indices: &mut Vec<usize>) {
        match &self.nodes[node_idx] {
            TreeNode::File { component_idx, .. } => {
                indices.push(*component_idx);
            }
            TreeNode::Folder { children, .. } => {
                for &child_idx in children {
                    self.collect_component_indices(child_idx, indices);
                }
            }
        }
    }

    /// Check if all components under a folder are selected
    pub fn is_folder_all_selected(&self, folder_idx: usize, components: &[Component]) -> bool {
        let indices = self.get_folder_component_indices(folder_idx);
        if indices.is_empty() {
            return false;
        }
        indices.iter().all(|&idx| components.get(idx).map(|c| c.selected).unwrap_or(false))
    }

    /// Check if any component under a folder is selected
    pub fn is_folder_any_selected(&self, folder_idx: usize, components: &[Component]) -> bool {
        let indices = self.get_folder_component_indices(folder_idx);
        indices.iter().any(|&idx| components.get(idx).map(|c| c.selected).unwrap_or(false))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree::test_support::make_component;

    /// A folder tree over `folder/sub/deep.md` plus two shallower files, with
    /// every component deselected so each test opts rows back in explicitly.
    fn nested_tree() -> (Vec<Component>, TreeView, usize) {
        let mut components = vec![
            make_component("folder/a.md"),
            make_component("folder/sub/deep.md"),
        ];
        for c in &mut components {
            c.selected = false;
        }

        let filtered: Vec<(usize, &Component)> = components.iter().enumerate().collect();
        let tree = TreeView::build_from_components(&components, &filtered);
        let folder_idx = tree
            .nodes
            .iter()
            .position(|n| n.is_folder() && n.depth() == 0)
            .expect("root folder");

        (components, tree, folder_idx)
    }

    #[test]
    fn folder_indices_reach_through_nested_folders() {
        let (_components, tree, folder_idx) = nested_tree();

        let mut indices = tree.get_folder_component_indices(folder_idx);
        indices.sort_unstable();

        assert_eq!(indices, vec![0, 1], "both the direct and the nested file must be collected");
    }

    #[test]
    fn all_selected_requires_every_descendant() {
        let (mut components, tree, folder_idx) = nested_tree();

        components[0].selected = true;
        assert!(tree.is_folder_any_selected(folder_idx, &components));
        assert!(
            !tree.is_folder_all_selected(folder_idx, &components),
            "the nested file is still unselected"
        );

        components[1].selected = true;
        assert!(tree.is_folder_all_selected(folder_idx, &components));
    }

    #[test]
    fn an_empty_folder_counts_as_neither() {
        // `all` over an empty set is vacuously true, which would render an
        // empty folder as fully selected; `is_folder_all_selected` guards
        // against that, and this pins the guard.
        let mut tree = TreeView::default();
        tree.nodes.push(TreeNode::Folder {
            name: "empty".to_string(),
            path: "empty".to_string(),
            expanded: true,
            children: Vec::new(),
            depth: 0,
            parent_idx: None,
        });

        assert!(!tree.is_folder_all_selected(0, &[]));
        assert!(!tree.is_folder_any_selected(0, &[]));
    }
}
