use super::{App, Tab};
use crate::component::{Component, InstallStatus};
use crate::tree::TreeView;
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
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

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ComponentFilters {
    selected: BTreeSet<ComponentFilter>,
}

impl From<ComponentFilter> for ComponentFilters {
    fn from(filter: ComponentFilter) -> Self {
        let mut filters = Self::default();
        filters.set(filter, true);
        filters
    }
}

impl ComponentFilters {
    pub fn contains(&self, filter: ComponentFilter) -> bool {
        if filter == ComponentFilter::All {
            self.selected.is_empty()
        } else {
            self.selected.contains(&filter)
        }
    }

    pub fn set(&mut self, filter: ComponentFilter, checked: bool) {
        if filter == ComponentFilter::All {
            if checked {
                self.selected.clear();
            }
        } else if checked {
            self.selected.insert(filter);
        } else {
            self.selected.remove(&filter);
        }
    }

    pub fn matches(&self, status: &InstallStatus) -> bool {
        self.selected.is_empty() || self.selected.iter().any(|filter| filter.matches(status))
    }

    pub fn display(&self) -> String {
        if self.selected.is_empty() {
            "All".into()
        } else {
            self.selected
                .iter()
                .map(|filter| filter.display())
                .collect::<Vec<_>>()
                .join(" / ")
        }
    }

    fn next(&self) -> Self {
        match self.selected.first() {
            None => ComponentFilter::Modified.into(),
            Some(filter) if self.selected.len() == 1 => filter.next().into(),
            _ => Self::default(),
        }
    }
}

impl App {
    pub fn set_component_filter(&mut self, filter: ComponentFilter, checked: bool) {
        if self.tab.to_component_type().is_none() {
            return;
        }
        self.component_filter.set(filter, checked);
        self.list_index = 0;
        self.rebuild_tree_views();
    }

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

    pub fn visible_components(&self) -> Vec<(usize, &Component)> {
        self.current_components()
            .into_iter()
            .filter(|(_, c)| self.hidden_visibility.matches(self.is_component_hidden(c)))
            .collect()
    }

    fn components_for_tab(&self, tab: Tab) -> Vec<(usize, &Component)> {
        let Some(component_type) = tab.to_component_type() else {
            return Vec::new();
        };
        let query = super::search::SearchTerms::new(&self.search_query);
        self.components
            .iter()
            .enumerate()
            .filter(|(_, c)| {
                c.component_type == component_type
                    && self.component_filter.matches(&c.status)
                    && query.matches_component(c)
            })
            .collect()
    }

    pub(super) fn rebuild_tree_views(&mut self) {
        self.tree_views = Tab::all()
            .iter()
            .filter(|tab| tab.to_component_type().is_some())
            .map(|&tab| {
                let components: Vec<_> = self
                    .components_for_tab(tab)
                    .into_iter()
                    .filter(|(_, c)| self.hidden_visibility.matches(self.is_component_hidden(c)))
                    .collect();
                (
                    tab,
                    TreeView::build_from_components(&self.components, &components),
                )
            })
            .collect();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::test_support::fresh_app;
    use crate::tree::test_support::make_component;

    #[test]
    fn visibility_modes_match_visible_and_hidden_items_explicitly() {
        use crate::app::HiddenVisibility;
        assert_eq!(HiddenVisibility::default(), HiddenVisibility::Exclude);
        for (mode, visible, hidden) in [
            (HiddenVisibility::Exclude, true, false),
            (HiddenVisibility::Include, true, true),
            #[cfg(feature = "gui")]
            (HiddenVisibility::Only, false, true),
        ] {
            assert_eq!(mode.matches(false), visible);
            assert_eq!(mode.matches(true), hidden);
            assert_eq!(mode.shows_hidden(), hidden);
        }
    }

    #[test]
    fn default_matches_every_status_and_last_uncheck_restores_default() {
        let mut filters = ComponentFilters::default();
        for status in [
            InstallStatus::New,
            InstallStatus::Modified,
            InstallStatus::External,
            InstallStatus::Unchanged,
            InstallStatus::Managed,
        ] {
            assert!(filters.matches(&status));
        }
        filters.set(ComponentFilter::All, false);
        assert!(filters.contains(ComponentFilter::All));
        filters.set(ComponentFilter::Modified, true);
        filters.set(ComponentFilter::New, true);
        assert!(filters.matches(&InstallStatus::Modified));
        assert!(filters.matches(&InstallStatus::New));
        assert!(!filters.matches(&InstallStatus::Unchanged));
        filters.set(ComponentFilter::Modified, false);
        filters.set(ComponentFilter::New, false);
        assert!(filters.contains(ComponentFilter::All));
        assert!(filters.matches(&InstallStatus::Managed));
    }

    #[test]
    fn checking_all_three_statuses_is_distinct_from_default_and_default_resets_them() {
        let mut filters = ComponentFilters::default();
        for filter in [
            ComponentFilter::Modified,
            ComponentFilter::External,
            ComponentFilter::New,
        ] {
            filters.set(filter, true);
            filters.set(filter, true);
        }
        assert!(!filters.matches(&InstallStatus::Unchanged));
        assert!(!filters.matches(&InstallStatus::Managed));
        assert_eq!(filters.display(), "Modified / External / New");
        assert_eq!(filters.next(), ComponentFilters::default());
        filters.set(ComponentFilter::All, true);
        assert!(filters.contains(ComponentFilter::All));
        assert!(!filters.contains(ComponentFilter::Modified));
        assert!(filters.matches(&InstallStatus::Unchanged));
    }

    #[test]
    fn multiple_statuses_bound_install_and_remove_queues_and_do_not_apply_to_mcp() {
        let mut app = fresh_app();
        app.components = ["modified/a.md", "external/a.md", "new/a.md"]
            .map(make_component)
            .into();
        app.components[0].status = InstallStatus::Modified;
        app.components[1].status = InstallStatus::External;
        app.set_component_filter(ComponentFilter::Modified, true);
        app.set_component_filter(ComponentFilter::New, true);
        app.install_selected().unwrap();
        assert_eq!(app.processing_queue, [0, 2]);
        app.close_processing();
        app.remove_selected().unwrap();
        assert_eq!(app.processing_queue, [0, 2]);
        app.close_processing();
        app.tab = Tab::McpServers;
        let before = app.component_filter.clone();
        app.set_component_filter(ComponentFilter::All, true);
        assert_eq!(app.component_filter, before);
    }
}
