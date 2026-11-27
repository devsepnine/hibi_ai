use std::path::PathBuf;

use anyhow::{Result, ensure};

use super::{App, Tab, TargetCli, View};
use crate::component::InstallStatus;
use crate::mcp::McpScope;

#[derive(Debug, PartialEq)]
pub(super) struct RemovalPlan {
    tab: Tab,
    target: Option<TargetCli>,
    destination: PathBuf,
    mcp_scope: Option<McpScope>,
    mcp_project_path: Option<String>,
    items: Vec<RemovalItem>,
}

#[derive(Debug, PartialEq)]
pub(crate) struct RemovalItem {
    index: usize,
    name: String,
    destination: Option<PathBuf>,
    setting_keys: Option<Vec<String>>,
    hook_event: Option<String>,
    pub status: Option<InstallStatus>,
}

impl RemovalItem {
    pub fn display_name(&self) -> String {
        self.name
            .chars()
            .map(|ch| if ch.is_control() { '\u{fffd}' } else { ch })
            .collect()
    }
}

impl App {
    pub(crate) fn request_removal(&mut self) {
        if self.current_view != View::List {
            return;
        }
        let selection = self.work_selection();
        if selection.remove.is_empty() {
            self.status_message = Some(
                if selection.selected == 0 {
                    "Select items in this category to remove."
                } else if self.is_claude_config() {
                    "No selected configuration items can be removed. Check the diff for details."
                } else {
                    "No selected items are installed and removable in this view."
                }
                .into(),
            );
            return;
        }
        self.pending_removal = Some(self.removal_plan(&selection.remove));
        self.removal_scroll = 0;
        self.current_view = View::ConfirmRemove;
    }

    pub(crate) fn pending_removal_items(&self) -> &[RemovalItem] {
        self.pending_removal
            .as_ref()
            .map_or(&[], |plan| &plan.items)
    }

    pub(crate) fn cancel_removal(&mut self) {
        self.pending_removal = None;
        self.removal_scroll = 0;
        if self.current_view == View::ConfirmRemove {
            self.current_view = View::List;
        }
    }

    pub(crate) fn confirm_removal(&mut self) -> Result<()> {
        const CHANGED: &str = "Selection changed. Review your selection before removing items.";
        ensure!(self.current_view == View::ConfirmRemove, "{CHANGED}");
        let confirmed = self.pending_removal.take();
        self.cancel_removal();

        let indices = self.work_selection().remove;
        ensure!(
            !indices.is_empty() && confirmed == Some(self.removal_plan(&indices)),
            "{CHANGED}"
        );
        self.start_removal(indices);
        Ok(())
    }

    fn removal_plan(&self, indices: &[usize]) -> RemovalPlan {
        let mcp_scope = (self.tab == Tab::McpServers).then_some(self.mcp_scope);
        let mcp_project_path =
            (mcp_scope == Some(McpScope::Local)).then(|| self.mcp_project_path.clone());
        RemovalPlan {
            tab: self.tab,
            target: self.target_cli,
            destination: self.dest_dir.clone(),
            mcp_scope,
            mcp_project_path,
            items: indices
                .iter()
                .map(|&index| self.removal_item(index))
                .collect(),
        }
    }

    fn removal_item(&self, index: usize) -> RemovalItem {
        match self.tab {
            Tab::McpServers | Tab::Plugins => RemovalItem {
                index,
                name: if self.tab == Tab::McpServers {
                    self.mcp_servers[index].def.name.clone()
                } else {
                    self.plugins[index].def.id()
                },
                destination: None,
                setting_keys: None,
                hook_event: None,
                status: None,
            },
            _ => {
                let component = &self.components[index];
                RemovalItem {
                    index,
                    name: component.name.clone(),
                    destination: Some(component.dest_path.clone()),
                    setting_keys: component
                        .setting_entry
                        .as_ref()
                        .map(|entry| entry.keys.clone()),
                    hook_event: component
                        .hook_config
                        .as_ref()
                        .map(|hook| hook.event.clone()),
                    status: Some(component.status.clone()),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::test_support::{fresh_app, make_mcp_server};
    use crate::mcp::McpStatus;
    use crate::tree::test_support::make_component;

    #[test]
    fn different_items_with_the_same_count_require_confirmation_again() {
        let mut app = fresh_app();
        app.components = ["first.md", "second.md"].map(make_component).into();
        for item in &mut app.components {
            item.status = InstallStatus::Modified;
        }
        app.components[1].selected = false;
        app.request_removal();
        assert_eq!(app.pending_removal_items()[0].display_name(), "first.md");

        app.components[0].selected = false;
        app.components[1].selected = true;
        assert!(app.confirm_removal().is_err());
        assert_eq!(app.current_view, View::List);
        assert!(app.processing_queue.is_empty());
        assert!(app.pending_removal_items().is_empty());
        assert!(app.components[1].selected);
        app.request_removal();
        assert_eq!(app.pending_removal_items()[0].display_name(), "second.md");
        app.confirm_removal().unwrap();
        assert_eq!(app.processing_queue, [1]);
        assert!(app.confirm_removal().is_err());
        assert_eq!(app.processing_queue, [1]);
    }

    #[test]
    fn replaced_file_destination_requires_confirmation_again() {
        let mut app = fresh_app();
        let mut item = make_component("same.md");
        item.status = InstallStatus::Modified;
        app.components.push(item);
        app.request_removal();

        app.components[0].dest_path = PathBuf::from("other/same.md");
        assert!(app.confirm_removal().is_err());
        assert!(app.processing_queue.is_empty());
    }

    #[test]
    fn changed_mcp_project_requires_confirmation_again() {
        let mut app = fresh_app();
        app.target_cli = Some(TargetCli::Claude);
        app.tab = Tab::McpServers;
        app.mcp_scope = McpScope::Local;
        app.mcp_project_path = "/project/first".into();
        let mut server = make_mcp_server();
        server.status = McpStatus::Installed;
        server.selected = true;
        app.mcp_servers.push(server);
        app.request_removal();

        app.mcp_project_path = "/project/second".into();
        assert!(app.confirm_removal().is_err());
        assert!(app.processing_queue.is_empty());
        assert_eq!(app.current_view, View::List);
    }
}
