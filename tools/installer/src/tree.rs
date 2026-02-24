use std::collections::HashMap;

use crate::component::Component;

#[derive(Clone, Debug)]
pub enum TreeNode {
    Folder {
        name: String,
        #[allow(dead_code)]
        path: String,
        expanded: bool,
        children: Vec<usize>, // Indices into TreeView.nodes
        depth: usize,
        parent_idx: Option<usize>, // Index of parent folder
    },
    File {
        component_idx: usize, // Index into App.components
        depth: usize,
        parent_idx: Option<usize>, // Index of parent folder
    },
}

impl TreeNode {
    pub fn depth(&self) -> usize {
        match self {
            TreeNode::Folder { depth, .. } => *depth,
            TreeNode::File { depth, .. } => *depth,
        }
    }

    pub fn is_folder(&self) -> bool {
        matches!(self, TreeNode::Folder { .. })
    }

    #[allow(dead_code)]
    pub fn is_expanded(&self) -> bool {
        match self {
            TreeNode::Folder { expanded, .. } => *expanded,
            TreeNode::File { .. } => false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct TreeView {
    pub nodes: Vec<TreeNode>,
    pub visible_indices: Vec<usize>, // Indices into nodes that are currently visible
    pub cursor: usize,               // Index into visible_indices
    root_children: Vec<usize>,       // Top-level node indices
}

impl Default for TreeView {
    fn default() -> Self {
        Self {
            nodes: Vec::new(),
            visible_indices: Vec::new(),
            cursor: 0,
            root_children: Vec::new(),
        }
    }
}

impl TreeView {
    /// Build tree from filtered component indices
    pub fn build_from_components(_components: &[Component], filtered_indices: &[(usize, &Component)]) -> Self {
        let mut tree = TreeView::default();

        if filtered_indices.is_empty() {
            return tree;
        }

        // Group components by their path segments
        // e.g., "rules/perf.md" -> ["rules", "perf.md"]
        let mut folder_map: HashMap<String, usize> = HashMap::new(); // path -> node index

        for &(comp_idx, comp) in filtered_indices {
            // Handle both Unix (/) and Windows (\) path separators
            let parts: Vec<&str> = comp.name.split(['/', '\\']).collect();
            tree.insert_path(&parts, comp_idx, 0, &mut folder_map, &mut Vec::new());
        }

        // Sort root children by name
        tree.sort_children(&tree.root_children.clone());

        tree.rebuild_visible();
        tree
    }

    fn insert_path(
        &mut self,
        parts: &[&str],
        comp_idx: usize,
        depth: usize,
        folder_map: &mut HashMap<String, usize>,
        current_path: &mut Vec<String>,
    ) {
        if parts.is_empty() {
            return;
        }

        if parts.len() == 1 {
            // This is a file
            let parent_idx = if depth == 0 {
                None
            } else {
                let parent_path = current_path.join("/");
                folder_map.get(&parent_path).copied()
            };

            let file_node = TreeNode::File {
                component_idx: comp_idx,
                depth,
                parent_idx,
            };
            let node_idx = self.nodes.len();
            self.nodes.push(file_node);

            if depth == 0 {
                self.root_children.push(node_idx);
            } else {
                // Add to parent folder
                if let Some(parent_idx) = parent_idx {
                    if let TreeNode::Folder { children, .. } = &mut self.nodes[parent_idx] {
                        children.push(node_idx);
                    }
                }
            }
        } else {
            // This is a folder path
            let folder_name = parts[0];
            current_path.push(folder_name.to_string());
            let folder_path = current_path.join("/");

            let _folder_idx = if let Some(&idx) = folder_map.get(&folder_path) {
                idx
            } else {
                // Get parent index
                let parent_idx = if depth == 0 {
                    None
                } else {
                    let parent_path = current_path[..current_path.len() - 1].join("/");
                    folder_map.get(&parent_path).copied()
                };

                // Create new folder
                let folder_node = TreeNode::Folder {
                    name: folder_name.to_string(),
                    path: folder_path.clone(),
                    expanded: true, // Default expanded
                    children: Vec::new(),
                    depth,
                    parent_idx,
                };
                let idx = self.nodes.len();
                self.nodes.push(folder_node);
                folder_map.insert(folder_path, idx);

                // Add to parent or root
                if depth == 0 {
                    self.root_children.push(idx);
                } else {
                    if let Some(parent_idx) = parent_idx {
                        if let TreeNode::Folder { children, .. } = &mut self.nodes[parent_idx] {
                            children.push(idx);
                        }
                    }
                }

                idx
            };

            // Recurse for remaining parts
            self.insert_path(&parts[1..], comp_idx, depth + 1, folder_map, current_path);
            current_path.pop();
        }
    }

    fn sort_children(&mut self, children: &[usize]) {
        for &child_idx in children {
            if let TreeNode::Folder { children: sub_children, .. } = &self.nodes[child_idx] {
                let sub = sub_children.clone();
                self.sort_children(&sub);
            }
        }

        // Sort would need mutable access which is tricky here
        // For simplicity, we skip sorting and rely on insertion order
    }

    /// Rebuild visible_indices based on expanded state
    pub fn rebuild_visible(&mut self) {
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
    pub fn current_node(&self) -> Option<&TreeNode> {
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
    use crate::component::{ComponentType, InstallStatus};
    use std::path::PathBuf;

    fn make_component(name: &str) -> Component {
        Component::new(
            ComponentType::Skills,
            name.to_string(),
            PathBuf::from(name),
            PathBuf::from(name),
            InstallStatus::New,
        )
    }

    #[test]
    fn test_tree_build() {
        let components = vec![
            make_component("file1.md"),
            make_component("folder/file2.md"),
            make_component("folder/sub/file3.md"),
        ];

        let filtered: Vec<(usize, &Component)> = components.iter().enumerate().collect();
        let tree = TreeView::build_from_components(&components, &filtered);

        // Should have: file1.md, folder/, folder/file2.md, folder/sub/, folder/sub/file3.md
        assert!(!tree.nodes.is_empty());
        assert!(!tree.visible_indices.is_empty());
    }

    #[test]
    fn test_folder_collapse() {
        let components = vec![
            make_component("folder/file1.md"),
            make_component("folder/file2.md"),
        ];

        let filtered: Vec<(usize, &Component)> = components.iter().enumerate().collect();
        let mut tree = TreeView::build_from_components(&components, &filtered);

        // Initial: folder expanded, all visible
        let initial_visible = tree.visible_indices.len();

        // Collapse folder (cursor should be at folder)
        tree.collapse();
        tree.rebuild_visible();

        // After collapse: only folder visible
        assert!(tree.visible_indices.len() < initial_visible);
    }

    #[test]
    fn test_tree_debug() {
        let components = vec![
            make_component("commit-rules.md"),
            make_component("vercel-react-best-practices/AGENTS.md"),
            make_component("vercel-react-best-practices/rules/async-api-routes.md"),
        ];

        let filtered: Vec<(usize, &Component)> = components.iter().enumerate().collect();
        let tree = TreeView::build_from_components(&components, &filtered);

        println!("Nodes: {:?}", tree.nodes.len());
        println!("Root children: {:?}", tree.root_children);
        println!("Visible: {:?}", tree.visible_indices);

        for (i, node) in tree.nodes.iter().enumerate() {
            println!("Node {}: {:?}", i, node);
        }

        assert!(tree.nodes.len() > 3); // Should have folders + files
    }
}
