use super::App;
use super::types::Tab;

impl App {
    pub fn toggle_selected(&mut self) {
        if self.tab == Tab::McpServers {
            if !self
                .visible_mcp_servers()
                .iter()
                .any(|(index, _)| *index == self.mcp_index)
            {
                return;
            }
            if let Some(m) = self.mcp_servers.get_mut(self.mcp_index) {
                m.selected = !m.selected;
            }
        } else if self.tab == Tab::Plugins {
            if !self
                .visible_plugins()
                .iter()
                .any(|(index, _)| *index == self.plugin_index)
            {
                return;
            }
            if let Some(p) = self.plugins.get_mut(self.plugin_index) {
                p.selected = !p.selected;
            }
        } else if self.is_cursor_on_folder() {
            // Toggle all components under folder
            self.toggle_folder_selection();
        } else if let Some(idx) = self.selected_component_index() {
            if let Some(c) = self.components.get_mut(idx) {
                c.selected = !c.selected;
            }
        }
    }

    /// Toggle selection for all components under current folder
    fn toggle_folder_selection(&mut self) {
        if let Some(tree) = self.tree_views.get(&self.tab) {
            if let Some(node_idx) = tree.current_node_idx() {
                let indices = tree.get_folder_component_indices(node_idx);
                if indices.is_empty() {
                    return;
                }

                // Check if all are currently selected
                let all_selected = indices.iter().all(|&idx| {
                    self.components
                        .get(idx)
                        .map(|c| c.selected)
                        .unwrap_or(false)
                });

                // Toggle: if all selected -> deselect all, otherwise select all
                let new_state = !all_selected;
                for &idx in &indices {
                    if let Some(c) = self.components.get_mut(idx) {
                        c.selected = new_state;
                    }
                }
            }
        }
    }

    pub fn toggle_all_selected(&mut self) {
        if self.tab == Tab::McpServers {
            let shown: Vec<_> = self
                .visible_mcp_servers()
                .iter()
                .map(|(index, _)| *index)
                .collect();
            let selected = shown.iter().any(|&index| !self.mcp_servers[index].selected);
            for index in shown {
                self.mcp_servers[index].selected = selected;
            }
        } else if self.tab == Tab::Plugins {
            let shown: Vec<_> = self
                .visible_plugins()
                .iter()
                .map(|(index, _)| *index)
                .collect();
            let selected = shown.iter().any(|&index| !self.plugins[index].selected);
            for index in shown {
                self.plugins[index].selected = selected;
            }
        } else if self.tab.to_component_type().is_some() {
            let shown: Vec<_> = self
                .visible_components()
                .iter()
                .map(|(index, _)| *index)
                .collect();
            let selected = shown.iter().any(|&index| !self.components[index].selected);
            if self.search_query.trim().is_empty() {
                let component_type = self.tab.to_component_type();
                for (index, component) in self.components.iter_mut().enumerate() {
                    if Some(&component.component_type) == component_type.as_ref() {
                        component.selected = selected && shown.contains(&index);
                    }
                }
            } else {
                for index in shown {
                    self.components[index].selected = selected;
                }
            }
        }
    }

    /// Whether anything is selected anywhere, not just in the visible tab.
    ///
    /// Every other helper here is scoped to `self.tab`, which is right for keys
    /// the user aims at one list. The exit prompt is not one of those: ticks
    /// survive tab switches, so a tab-scoped check would wave the user off the
    /// screen while their choices in another tab were still pending.
    pub fn has_selection(&self) -> bool {
        self.components.iter().any(|c| c.selected)
            || self.mcp_servers.iter().any(|m| m.selected)
            || self.plugins.iter().any(|p| p.selected)
    }

    /// Drop every selection in every tab.
    ///
    /// Leaving the List view for the CLI picker already discards the ticks in
    /// practice, because coming back re-scans and replaces all three lists.
    /// Clearing them here makes that the exit's own doing rather than a
    /// side effect of the loader it happens to be followed by.
    pub fn clear_all_selections(&mut self) {
        for c in &mut self.components {
            c.selected = false;
        }
        for m in &mut self.mcp_servers {
            m.selected = false;
        }
        for p in &mut self.plugins {
            p.selected = false;
        }
    }
}

#[cfg(test)]
mod search_tests {
    use super::*;
    use crate::app::test_support::{fresh_app, make_mcp_server, make_plugin};
    use crate::tree::test_support::make_component;

    #[test]
    fn matching_bulk_selection_preserves_nonmatching_items_in_all_lists() {
        for tab in [Tab::Skills, Tab::McpServers, Tab::Plugins] {
            let mut app = fresh_app();
            app.tab = tab;
            let mut hidden_component = make_component("hidden-component");
            hidden_component.selected = true;
            let mut shown_component = make_component("target-component");
            shown_component.selected = true;
            app.components = vec![hidden_component, shown_component];
            app.mcp_servers = ["hidden-mcp", "target-mcp"]
                .map(|name| {
                    let mut server = make_mcp_server();
                    server.def.name = name.into();
                    server.selected = true;
                    server
                })
                .into();
            app.plugins = ["hidden-plugin", "target-plugin"]
                .map(|name| {
                    let mut plugin = make_plugin();
                    plugin.def.name = name.into();
                    plugin.selected = true;
                    plugin
                })
                .into();
            app.set_search_query("target".into());
            app.toggle_all_selected();
            assert!(app.components[0].selected);
            assert!(app.mcp_servers[0].selected);
            assert!(app.plugins[0].selected);
            match tab {
                Tab::Skills => assert!(!app.components[1].selected),
                Tab::McpServers => assert!(!app.mcp_servers[1].selected),
                Tab::Plugins => assert!(!app.plugins[1].selected),
                _ => unreachable!(),
            }
            app.set_search_query("no-match".into());
            let selections = (
                app.components[0].selected,
                app.mcp_servers[0].selected,
                app.plugins[0].selected,
            );
            app.toggle_all_selected();
            app.toggle_selected();
            assert_eq!(
                selections,
                (
                    app.components[0].selected,
                    app.mcp_servers[0].selected,
                    app.plugins[0].selected
                )
            );
        }
    }

    #[test]
    fn stale_mcp_and_plugin_cursor_cannot_toggle_nonmatching_item() {
        for tab in [Tab::McpServers, Tab::Plugins] {
            let mut app = fresh_app();
            app.tab = tab;
            app.mcp_servers = ["hidden", "target"]
                .map(|name| {
                    let mut server = make_mcp_server();
                    server.def.name = name.into();
                    server
                })
                .into();
            app.plugins = ["hidden", "target"]
                .map(|name| {
                    let mut plugin = make_plugin();
                    plugin.def.name = name.into();
                    plugin
                })
                .into();
            app.set_search_query("target".into());
            app.mcp_index = 0;
            app.plugin_index = 0;
            app.toggle_selected();
            assert!(!app.mcp_servers[0].selected);
            assert!(!app.plugins[0].selected);
        }
    }
}
