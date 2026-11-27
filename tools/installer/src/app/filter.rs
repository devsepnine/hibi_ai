use super::{App, HiddenVisibility, Tab};
use crate::component::{Component, InstallStatus};
use crate::tree::TreeView;
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum ComponentFilter {
    #[default]
    Default,
    Modified,
    External,
    New,
    Installed,
}

impl ComponentFilter {
    pub fn display(self) -> &'static str {
        match self {
            Self::Default => "Default",
            Self::Installed => "Installed",
            Self::Modified => "Modified",
            Self::External => "External",
            Self::New => "New",
        }
    }

    pub fn matches(self, status: &InstallStatus) -> bool {
        match self {
            Self::Default => *status != InstallStatus::External,
            Self::Installed => matches!(status, InstallStatus::Unchanged | InstallStatus::Managed),
            Self::Modified => *status == InstallStatus::Modified,
            Self::External => *status == InstallStatus::External,
            Self::New => *status == InstallStatus::New,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FilterChoice {
    Status(ComponentFilter),
    Visibility(HiddenVisibility),
}

pub const STATUS_CHOICES: [ComponentFilter; 5] = [
    ComponentFilter::Default,
    ComponentFilter::Modified,
    ComponentFilter::External,
    ComponentFilter::New,
    ComponentFilter::Installed,
];
pub const VISIBILITY_CHOICES: [HiddenVisibility; 3] = [
    HiddenVisibility::Exclude,
    HiddenVisibility::Include,
    HiddenVisibility::Only,
];

impl FilterChoice {
    pub const FIRST: Self = Self::Status(ComponentFilter::Default);

    pub fn choices() -> impl Iterator<Item = Self> {
        STATUS_CHOICES
            .into_iter()
            .map(Self::Status)
            .chain(VISIBILITY_CHOICES.into_iter().map(Self::Visibility))
    }

    pub fn step(self, offset: isize) -> Self {
        let count = STATUS_CHOICES.len() + VISIBILITY_CHOICES.len();
        let current = Self::choices()
            .position(|choice| choice == self)
            .unwrap_or(0);
        let next = (current as isize + offset).rem_euclid(count as isize) as usize;
        Self::choices().nth(next).unwrap_or(Self::FIRST)
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
        if filter == ComponentFilter::Default {
            self.selected.is_empty()
        } else {
            self.selected.contains(&filter)
        }
    }

    pub fn set(&mut self, filter: ComponentFilter, checked: bool) {
        if filter == ComponentFilter::Default {
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
        if self.selected.is_empty() {
            ComponentFilter::Default.matches(status)
        } else {
            self.selected.iter().any(|filter| filter.matches(status))
        }
    }

    pub fn display(&self) -> String {
        if self.selected.is_empty() {
            ComponentFilter::Default.display().into()
        } else {
            self.selected
                .iter()
                .map(|filter| filter.display())
                .collect::<Vec<_>>()
                .join(" / ")
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
            .map(|&tab| (tab, self.build_component_tree(tab)))
            .collect();
    }

    pub(super) fn rebuild_component_tree(&mut self, tab: Tab) {
        self.tree_views.insert(tab, self.build_component_tree(tab));
    }

    fn build_component_tree(&self, tab: Tab) -> TreeView {
        let components: Vec<_> = self
            .components_for_tab(tab)
            .into_iter()
            .filter(|(_, c)| self.hidden_visibility.matches(self.is_component_hidden(c)))
            .collect();
        TreeView::build_from_components(&self.components, &components)
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
            (HiddenVisibility::Only, false, true),
        ] {
            assert_eq!(mode.matches(false), visible);
            assert_eq!(mode.matches(true), hidden);
            assert_eq!(mode.shows_hidden(), hidden);
        }
    }

    #[test]
    fn default_matches_managed_sources_and_last_uncheck_restores_default() {
        let mut filters = ComponentFilters::default();
        for status in [
            InstallStatus::New,
            InstallStatus::Modified,
            InstallStatus::Blocked("invalid config".into()),
            InstallStatus::Unchanged,
            InstallStatus::Managed,
        ] {
            assert!(filters.matches(&status));
        }
        assert!(!filters.matches(&InstallStatus::External));
        filters.set(ComponentFilter::Default, false);
        assert!(filters.contains(ComponentFilter::Default));
        filters.set(ComponentFilter::Modified, true);
        filters.set(ComponentFilter::New, true);
        assert!(filters.matches(&InstallStatus::Modified));
        assert!(filters.matches(&InstallStatus::New));
        assert!(!filters.matches(&InstallStatus::Unchanged));
        filters.set(ComponentFilter::Modified, false);
        filters.set(ComponentFilter::New, false);
        assert!(filters.contains(ComponentFilter::Default));
        assert!(filters.matches(&InstallStatus::Managed));
    }

    #[test]
    fn checking_multiple_statuses_is_distinct_from_default_and_default_resets_them() {
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
        filters.set(ComponentFilter::Installed, true);
        assert!(filters.matches(&InstallStatus::Managed));
        assert!(filters.matches(&InstallStatus::Unchanged));
        filters.set(ComponentFilter::Default, true);
        assert!(filters.contains(ComponentFilter::Default));
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
        app.request_removal();
        assert_eq!(app.pending_removal_items().len(), 1);
        app.confirm_removal().unwrap();
        assert_eq!(app.processing_queue, [0]);
        app.close_processing();
        app.tab = Tab::McpServers;
        let before = app.component_filter.clone();
        app.set_component_filter(ComponentFilter::Default, true);
        assert_eq!(app.component_filter, before);
    }
}
