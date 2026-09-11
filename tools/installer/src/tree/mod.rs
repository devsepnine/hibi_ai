//! Collapsible folder/file tree over a flat component list.
//!
//! `TreeNode` and `TreeView` live here; the operations are split by concern
//! into sibling modules — `build` turns component paths into nodes,
//! `navigate` moves the cursor and expands/collapses, `selection` answers
//! folder-level selection questions. Nodes are held in one flat `Vec` and
//! referred to by index, so a child module reaches parents and children
//! through `usize` rather than through borrows.

mod build;
mod navigate;
mod selection;

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

/// Component fixtures shared by the sibling modules' tests.
#[cfg(test)]
mod test_support {
    use crate::component::{Component, ComponentType, InstallStatus};
    use std::path::PathBuf;

    pub(super) fn make_component(name: &str) -> Component {
        Component::new(
            ComponentType::Skills,
            name.to_string(),
            PathBuf::from(name),
            PathBuf::from(name),
            InstallStatus::New,
        )
    }
}
