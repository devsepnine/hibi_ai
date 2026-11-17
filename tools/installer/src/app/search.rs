use unicode_normalization::UnicodeNormalization;

use super::App;
use crate::component::Component;
use crate::mcp::McpServer;
use crate::plugin::Plugin;

pub(super) struct SearchTerms(Vec<String>);

fn comparison_text(text: &str) -> String {
    text.nfc().collect::<String>().to_lowercase()
}

impl SearchTerms {
    pub(super) fn new(query: &str) -> Self {
        Self(query.split_whitespace().map(comparison_text).collect())
    }

    fn matches(&self, fields: &[&str]) -> bool {
        if self.0.is_empty() {
            return true;
        }
        let fields: Vec<_> = fields.iter().map(|field| comparison_text(field)).collect();
        self.0
            .iter()
            .all(|term| fields.iter().any(|field| field.contains(term)))
    }

    pub(super) fn matches_component(&self, component: &Component) -> bool {
        let hook = component.hook_config.as_ref();
        self.matches(&[
            &component.name,
            component.source_path.to_string_lossy().as_ref(),
            component.dest_path.to_string_lossy().as_ref(),
            &component.source_name,
            hook.map(|hook| hook.event.as_str()).unwrap_or_default(),
            hook.and_then(|hook| hook.description.as_deref())
                .unwrap_or_default(),
        ])
    }
}

impl App {
    pub fn set_search_query(&mut self, query: String) {
        if self.search_query == query {
            return;
        }
        self.search_query = query;
        self.rebuild_tree_views();
        self.reset_visible_list_cursors();
    }

    pub fn visible_mcp_servers(&self) -> Vec<(usize, &McpServer)> {
        let query = SearchTerms::new(&self.search_query);
        self.mcp_servers
            .iter()
            .enumerate()
            .filter(|(_, server)| {
                let mut fields = vec![
                    server.def.name.as_str(),
                    server.def.description.as_str(),
                    server.def.category.as_str(),
                    server.source_name.as_str(),
                    server.def.command.as_deref().unwrap_or_default(),
                    server.def.url.as_deref().unwrap_or_default(),
                ];
                fields.extend(server.def.env.iter().map(String::as_str));
                query.matches(&fields)
            })
            .collect()
    }

    pub fn visible_plugins(&self) -> Vec<(usize, &Plugin)> {
        let query = SearchTerms::new(&self.search_query);
        self.plugins
            .iter()
            .enumerate()
            .filter(|(_, plugin)| {
                query.matches(&[
                    &plugin.def.name,
                    &plugin.def.marketplace,
                    plugin.def.comment.as_deref().unwrap_or_default(),
                    &plugin.def.source,
                    &plugin.source_name,
                ])
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::test_support::{fresh_app, make_mcp_server, make_plugin};
    use crate::app::{ComponentFilter, Tab};
    use crate::component::{ComponentType, InstallStatus};
    use crate::tree::TreeNode;

    fn component(name: &str, status: InstallStatus) -> Component {
        Component::new(
            ComponentType::Skills,
            name.into(),
            std::path::PathBuf::from("/source/Team Library").join(name),
            std::path::PathBuf::from("/destination/설정").join(name),
            status,
        )
        .with_source_name("Shared Library")
    }

    fn shown_components(app: &App) -> Vec<usize> {
        app.visible_components()
            .iter()
            .map(|(index, _)| *index)
            .collect()
    }

    #[test]
    fn whitespace_query_shows_all_and_terms_are_case_insensitive_and_combined() {
        let mut app = fresh_app();
        app.components = vec![
            component("nested/CAFÉ-검색.md", InstallStatus::New),
            component("other/plain.md", InstallStatus::New),
        ];
        app.set_search_query(" \t\n\u{2003}".into());
        assert_eq!(shown_components(&app), vec![0, 1]);
        app.set_search_query("café 검색 SHARED".into());
        assert_eq!(shown_components(&app), vec![0]);
        app.set_search_query("café missing".into());
        assert!(app.visible_components().is_empty());
        assert!(app.tree_views[&Tab::Skills].visible_indices.is_empty());
    }

    #[test]
    fn source_and_destination_full_paths_and_nested_names_are_searchable() {
        let mut app = fresh_app();
        app.components
            .push(component("nested/Rule.md", InstallStatus::Modified));
        for query in [
            "nested/rule",
            "source/team",
            "destination/설정",
            "rule LIBRARY",
        ] {
            app.set_search_query(query.into());
            assert_eq!(shown_components(&app), vec![0], "query {query}");
        }
    }

    #[test]
    fn search_combines_status_and_hidden_rules_without_changing_selections() {
        let mut app = fresh_app();
        app.dest_dir = "/destination".into();
        app.components = vec![
            component("hidden/Match.md", InstallStatus::External),
            component("visible/Match.md", InstallStatus::Modified),
            component("new/Match.md", InstallStatus::New),
        ];
        app.components[0].selected = true;
        app.hidden_folders
            .insert(app.dest_dir.join("skills/hidden"));
        app.set_search_query("match".into());
        assert_eq!(shown_components(&app), vec![1, 2]);
        app.component_filter = ComponentFilter::External;
        app.set_search_query("MATCH".into());
        assert!(app.visible_components().is_empty());
        assert!(app.components[0].selected);
        app.show_hidden = true;
        app.set_search_query("Match".into());
        assert_eq!(shown_components(&app), vec![0]);
        app.component_filter = ComponentFilter::Modified;
        app.set_search_query("match shared".into());
        assert_eq!(shown_components(&app), vec![1]);
        assert!(app.components.iter().all(|component| component.selected));
    }

    #[test]
    fn matching_descendants_keep_their_ancestor_folders_and_global_indices() {
        let mut app = fresh_app();
        app.components = vec![
            component("excluded.md", InstallStatus::New),
            component("deep/folder/Needle.md", InstallStatus::New),
        ];
        app.set_search_query("needle".into());
        let tree = &app.tree_views[&Tab::Skills];
        assert_eq!(tree.nodes.iter().filter(|node| node.is_folder()).count(), 2);
        assert!(tree.nodes.iter().any(|node| matches!(
            node,
            TreeNode::File {
                component_idx: 1,
                ..
            }
        )));
        assert!(!tree.nodes.iter().any(|node| matches!(
            node,
            TreeNode::File {
                component_idx: 0,
                ..
            }
        )));
    }

    #[test]
    fn mcp_search_uses_metadata_and_preserves_global_indices_and_selection() {
        let mut app = fresh_app();
        let first = make_mcp_server();
        let mut matching = make_mcp_server();
        matching.def.name = "Database".into();
        matching.def.description = "한글 문서 Query".into();
        matching.def.category = "Storage".into();
        matching.source_name = "Team Library".into();
        matching.selected = true;
        app.mcp_servers = vec![first, matching];
        app.set_search_query("DATABASE 한글 storage TEAM".into());
        assert_eq!(
            app.visible_mcp_servers()
                .iter()
                .map(|(index, _)| *index)
                .collect::<Vec<_>>(),
            vec![1]
        );
        assert_eq!(app.mcp_index, 1);
        assert!(app.mcp_servers[1].selected);
        app.set_search_query("not-present".into());
        assert!(app.visible_mcp_servers().is_empty());
        assert_eq!(app.mcp_index, 0);
        assert!(app.mcp_servers[1].selected);
    }

    #[test]
    fn plugin_search_includes_marketplace_comment_source_and_source_url() {
        let mut app = fresh_app();
        let first = make_plugin();
        let mut matching = make_plugin();
        matching.def.name = "Review".into();
        matching.def.marketplace = "Official Tools".into();
        matching.def.comment = Some("문서 Analysis".into());
        matching.def.source = "https://example.invalid/Team/Repository.git".into();
        matching.source_name = "Custom".into();
        app.plugins = vec![first, matching];
        app.set_search_query("review OFFICIAL 문서 team/repository CUSTOM".into());
        assert_eq!(
            app.visible_plugins()
                .iter()
                .map(|(index, _)| *index)
                .collect::<Vec<_>>(),
            vec![1]
        );
        assert_eq!(app.plugin_index, 1);
        app.set_search_query(" \n".into());
        assert_eq!(app.visible_plugins().len(), 2);
        app.set_search_query("review demo-market".into());
        assert!(app.visible_plugins().is_empty());
    }

    #[test]
    fn query_survives_category_changes_and_a_new_target_clears_it() {
        let mut app = fresh_app();
        app.set_search_query("Needle".into());
        app.tab = Tab::McpServers;
        assert_eq!(app.search_query, "Needle");
        app.select_cli(crate::app::TargetCli::Codex).unwrap();
        assert!(app.search_query.is_empty());
    }

    #[test]
    fn hooks_are_searchable_by_event_and_description() {
        let mut app = fresh_app();
        app.tab = Tab::Hooks;
        let mut hook = component("generic.sh", InstallStatus::New);
        hook.component_type = ComponentType::Hooks;
        hook.hook_config = Some(crate::component::HookConfig {
            name: "generic".into(),
            event: "PreToolUse".into(),
            hook_type: "command".into(),
            timeout: None,
            description: Some("문서 Preview".into()),
            deprecated: None,
        });
        app.components.push(hook);
        app.set_search_query("pretooluse 문서 PREVIEW".into());
        assert_eq!(shown_components(&app), vec![0]);
        app.set_search_query("PostToolUse".into());
        assert!(app.visible_components().is_empty());
    }

    #[test]
    fn mcp_definition_keys_commands_and_urls_are_searchable_but_values_are_not() {
        let mut app = fresh_app();
        let mut server = make_mcp_server();
        server.def.env = vec!["SERVICE_TOKEN".into()];
        server.def.command = Some("provider-cli".into());
        server.def.url = Some("https://mcp.example.invalid/endpoint".into());
        app.mcp_servers.push(server);
        let secret = "private-collected-value-42";
        app.env_input_buffer = secret.into();
        app.env_input_values = vec![("SERVICE_TOKEN".into(), secret.into())];
        app.mcp_env_values
            .insert(0, vec![("SERVICE_TOKEN".into(), secret.into())]);
        app.set_search_query("service_token MCP.EXAMPLE.INVALID endpoint PROVIDER-CLI".into());
        assert_eq!(app.visible_mcp_servers().len(), 1);
        app.set_search_query(secret.into());
        assert!(app.visible_mcp_servers().is_empty());
        app.set_search_query("service_token".into());
        assert_eq!(app.visible_mcp_servers().len(), 1);
    }

    #[test]
    fn canonical_normalization_matches_composed_and_decomposed_filenames_both_ways() {
        for composed in ["한글/검색.md", "CAFÉ/Guide.md"] {
            let decomposed: String = composed.nfd().collect();
            assert_ne!(composed, decomposed);
            for (field, query) in [
                (decomposed.as_str(), composed.to_lowercase()),
                (composed, decomposed.to_lowercase()),
            ] {
                let mut app = fresh_app();
                app.components.push(component(field, InstallStatus::New));
                let source_path = app.components[0].source_path.clone();
                let dest_path = app.components[0].dest_path.clone();
                app.set_search_query(query.clone());
                assert_eq!(shown_components(&app), vec![0]);
                assert_eq!(app.search_query, query);
                assert_eq!(app.components[0].name, field);
                assert_eq!(app.components[0].source_path, source_path);
                assert_eq!(app.components[0].dest_path, dest_path);
            }
        }
    }

    #[test]
    fn metadata_normalization_preserves_accents_and_original_query() {
        let mut app = fresh_app();
        let mut server = make_mcp_server();
        server.def.description = "한글 café".nfd().collect();
        app.mcp_servers.push(server);
        let mut plugin = make_plugin();
        plugin.def.comment = Some("한글 café".into());
        app.plugins.push(plugin);
        for query in ["한글 CAFÉ".to_string(), "한글 CAFÉ".nfd().collect()] {
            app.set_search_query(query.clone());
            assert_eq!(app.visible_mcp_servers().len(), 1);
            assert_eq!(app.visible_plugins().len(), 1);
            assert_eq!(app.search_query, query);
        }
        app.set_search_query("cafe".into());
        assert!(app.visible_mcp_servers().is_empty());
        assert!(app.visible_plugins().is_empty());
    }
}
