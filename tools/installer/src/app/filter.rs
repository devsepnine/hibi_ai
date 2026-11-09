use super::{App, Tab};
use crate::component::{Component, InstallStatus};
use crate::tree::TreeView;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ComponentFilter {
    #[default]
    All,
    Modified,
    External,
    New,
}

impl ComponentFilter {
    pub fn display(self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Modified => "Modified",
            Self::External => "External",
            Self::New => "New",
        }
    }

    pub fn matches(self, status: &InstallStatus) -> bool {
        match self {
            Self::All => true,
            Self::Modified => *status == InstallStatus::Modified,
            Self::External => *status == InstallStatus::External,
            Self::New => *status == InstallStatus::New,
        }
    }

    fn next(self) -> Self {
        match self {
            Self::All => Self::Modified,
            Self::Modified => Self::External,
            Self::External => Self::New,
            Self::New => Self::All,
        }
    }
}

impl App {
    pub fn cycle_component_filter(&mut self) {
        if self.tab.to_component_type().is_none() {
            return;
        }
        self.component_filter = self.component_filter.next();
        self.list_index = 0;
        self.rebuild_tree_views();
    }

    pub fn current_components(&self) -> Vec<(usize, &Component)> {
        self.components_for_tab(self.tab)
    }

    fn components_for_tab(&self, tab: Tab) -> Vec<(usize, &Component)> {
        let Some(component_type) = tab.to_component_type() else {
            return Vec::new();
        };
        self.components
            .iter()
            .enumerate()
            .filter(|(_, c)| {
                c.component_type == component_type && self.component_filter.matches(&c.status)
            })
            .collect()
    }

    pub(super) fn rebuild_tree_views(&mut self) {
        self.tree_views = Tab::all()
            .iter()
            .filter(|tab| tab.to_component_type().is_some())
            .map(|&tab| {
                let components = self.components_for_tab(tab);
                (
                    tab,
                    TreeView::build_from_components(&self.components, &components),
                )
            })
            .collect();
    }
}
