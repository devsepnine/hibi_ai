use std::collections::HashMap;

use crate::component::Component;

use super::{TreeNode, TreeView};

impl TreeView {
    /// Build tree from filtered component indices.
    ///
    /// Children come out in the order the scanner walked them: nothing
    /// upstream sorts components, and the caller only filters. That scan
    /// order is what the UI has always displayed — sorting here would be
    /// a visible change, not a cleanup.
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

            if !folder_map.contains_key(&folder_path) {
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
                } else if let Some(parent_idx) = parent_idx {
                    if let TreeNode::Folder { children, .. } = &mut self.nodes[parent_idx] {
                        children.push(idx);
                    }
                }
            }

            // Recurse for remaining parts
            self.insert_path(&parts[1..], comp_idx, depth + 1, folder_map, current_path);
            current_path.pop();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree::test_support::make_component;

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

    #[test]
    fn a_folder_seen_twice_is_created_once() {
        // Two components under the same folder must share one folder node,
        // and that node must hold both files. This is the `folder_map` hit
        // path -- the only branch that reuses an existing folder.
        let components = vec![
            make_component("folder/a.md"),
            make_component("folder/b.md"),
        ];

        let filtered: Vec<(usize, &Component)> = components.iter().enumerate().collect();
        let tree = TreeView::build_from_components(&components, &filtered);

        let folders: Vec<&TreeNode> = tree.nodes.iter().filter(|n| n.is_folder()).collect();
        assert_eq!(folders.len(), 1, "one folder node expected, got {}", folders.len());

        let TreeNode::Folder { children, .. } = folders[0] else {
            unreachable!("filtered to folders above");
        };
        assert_eq!(children.len(), 2, "both files must attach to the shared folder");
    }
}
