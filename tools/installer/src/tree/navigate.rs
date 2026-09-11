use super::{TreeNode, TreeView};

impl TreeView {
    /// Rebuild visible_indices based on expanded state
    pub(super) fn rebuild_visible(&mut self) {
        self.visible_indices.clear();
        for &root_idx in &self.root_children.clone() {
            self.add_visible_recursive(root_idx);
        }

        // Clamp cursor
        if !self.visible_indices.is_empty() && self.cursor >= self.visible_indices.len() {
            self.cursor = self.visible_indices.len() - 1;
        }
    }

    fn add_visible_recursive(&mut self, node_idx: usize) {
        self.visible_indices.push(node_idx);

        if let TreeNode::Folder { expanded, children, .. } = &self.nodes[node_idx] {
            if *expanded {
                let child_indices = children.clone();
                for child_idx in child_indices {
                    self.add_visible_recursive(child_idx);
                }
            }
        }
    }

    /// Get current node at cursor
    fn current_node(&self) -> Option<&TreeNode> {
        self.visible_indices
            .get(self.cursor)
            .and_then(|&idx| self.nodes.get(idx))
    }

    /// Get current node index
    pub fn current_node_idx(&self) -> Option<usize> {
        self.visible_indices.get(self.cursor).copied()
    }

    /// Check if cursor is on a folder
    pub fn is_on_folder(&self) -> bool {
        self.current_node().map(|n| n.is_folder()).unwrap_or(false)
    }

    /// Check if current folder is expanded
    pub fn is_current_folder_expanded(&self) -> bool {
        self.current_node().map(|n| n.is_expanded()).unwrap_or(false)
    }

    /// Get component index if cursor is on a file
    pub fn current_component_idx(&self) -> Option<usize> {
        match self.current_node() {
            Some(TreeNode::File { component_idx, .. }) => Some(*component_idx),
            _ => None,
        }
    }

    /// Toggle expand/collapse for current folder
    pub fn toggle_expand(&mut self) {
        if let Some(node_idx) = self.current_node_idx() {
            if let TreeNode::Folder { expanded, .. } = &mut self.nodes[node_idx] {
                *expanded = !*expanded;
                self.rebuild_visible();
            }
        }
    }

    /// Expand current folder (if it's a folder)
    pub fn expand(&mut self) {
        if let Some(node_idx) = self.current_node_idx() {
            if let TreeNode::Folder { expanded, .. } = &mut self.nodes[node_idx] {
                if !*expanded {
                    *expanded = true;
                    self.rebuild_visible();
                }
            }
        }
    }

    /// Collapse current folder (if it's a folder)
    pub fn collapse(&mut self) {
        if let Some(node_idx) = self.current_node_idx() {
            if let TreeNode::Folder { expanded, .. } = &mut self.nodes[node_idx] {
                if *expanded {
                    *expanded = false;
                    self.rebuild_visible();
                }
            }
        }
    }

    /// Collapse parent folder (when cursor is on a file or folder)
    pub fn collapse_parent(&mut self) {
        if let Some(current_idx) = self.current_node_idx() {
            // Get parent index directly from the node
            let parent_idx = match &self.nodes[current_idx] {
                TreeNode::Folder { parent_idx, .. } => *parent_idx,
                TreeNode::File { parent_idx, .. } => *parent_idx,
            };

            // If parent exists, collapse it
            if let Some(parent_idx) = parent_idx {
                if let TreeNode::Folder { expanded, .. } = &mut self.nodes[parent_idx] {
                    if *expanded {
                        *expanded = false;
                        self.rebuild_visible();
                        // Move cursor to the collapsed parent folder
                        if let Some(new_pos) = self.visible_indices.iter().position(|&idx| idx == parent_idx) {
                            self.cursor = new_pos;
                        }
                    }
                }
            }
        }
    }

    /// Move cursor down
    pub fn next(&mut self) {
        if !self.visible_indices.is_empty() {
            self.cursor = (self.cursor + 1) % self.visible_indices.len();
        }
    }

    /// Move cursor up
    pub fn prev(&mut self) {
        if !self.visible_indices.is_empty() {
            self.cursor = if self.cursor == 0 {
                self.visible_indices.len() - 1
            } else {
                self.cursor - 1
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::Component;
    use crate::tree::test_support::make_component;

    fn tree_of(names: &[&str]) -> (Vec<Component>, TreeView) {
        let components: Vec<Component> = names.iter().map(|n| make_component(n)).collect();
        let filtered: Vec<(usize, &Component)> = components.iter().enumerate().collect();
        let tree = TreeView::build_from_components(&components, &filtered);
        (components, tree)
    }

    #[test]
    fn test_folder_collapse() {
        let (_components, mut tree) = tree_of(&["folder/file1.md", "folder/file2.md"]);

        // Initial: folder expanded, all visible
        let initial_visible = tree.visible_indices.len();

        // Collapse folder (cursor should be at folder)
        tree.collapse();
        tree.rebuild_visible();

        // After collapse: only folder visible
        assert!(tree.visible_indices.len() < initial_visible);
    }

    #[test]
    fn collapse_parent_moves_the_cursor_onto_the_folder() {
        // Cursor starts on the folder, so step onto the child file first --
        // collapsing from there must both hide the file and land the cursor
        // on the now-collapsed parent, or the next keypress acts on a
        // different row than the user sees highlighted.
        let (_components, mut tree) = tree_of(&["folder/file1.md", "folder/file2.md"]);
        tree.next();
        assert!(tree.current_component_idx().is_some(), "expected to be on a file");

        tree.collapse_parent();

        assert_eq!(tree.visible_indices.len(), 1, "only the folder should remain visible");
        assert!(tree.is_on_folder(), "cursor must follow the collapsed parent");
        assert!(!tree.is_current_folder_expanded());
    }

    #[test]
    fn cursor_wraps_at_both_ends() {
        let (_components, mut tree) = tree_of(&["a.md", "b.md"]);
        assert_eq!(tree.cursor, 0);

        tree.prev();
        assert_eq!(tree.cursor, 1, "prev at the top wraps to the last row");

        tree.next();
        assert_eq!(tree.cursor, 0, "next at the bottom wraps to the first row");
    }

    #[test]
    fn navigation_on_an_empty_tree_is_inert() {
        let mut tree = TreeView::default();
        tree.next();
        tree.prev();
        tree.toggle_expand();
        tree.collapse_parent();
        assert_eq!(tree.cursor, 0);
        assert!(tree.current_node().is_none());
    }
}
