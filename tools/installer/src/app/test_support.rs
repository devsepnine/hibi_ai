use super::types::{FocusArea, Tab, View};
use super::App;
use crate::mcp::{McpServer, McpServerDef, McpStatus};
use crate::plugin::{Plugin, PluginDef, PluginStatus};

/// An unselected MCP server, for tests that need that list non-empty.
pub(crate) fn make_mcp_server() -> McpServer {
    McpServer::new(
        McpServerDef {
            name: "demo".to_string(),
            description: String::new(),
            r#type: None,
            command: Some("demo".to_string()),
            url: None,
            category: "util".to_string(),
            env: Vec::new(),
        },
        McpStatus::NotInstalled,
    )
}

/// An unselected plugin, for tests that need that list non-empty.
pub(crate) fn make_plugin() -> Plugin {
    Plugin::new(
        PluginDef {
            name: "demo".to_string(),
            marketplace: "demo-market".to_string(),
            source: "https://example.invalid".to_string(),
            comment: None,
        },
        PluginStatus::NotInstalled,
    )
}

/// An inert `App` for unit tests.
///
/// `App::new()` does filesystem I/O for source resolution, which would make
/// every test that only needs in-memory state brittle and slow — so the
/// struct is hand-built with empty collections instead. Tests set the few
/// fields they care about and leave the rest untouched. Listing every field
/// is deliberate: a new field on `App` then fails to compile here until it
/// gets a considered test default.
pub(crate) fn fresh_app() -> App {
    App {
        target_cli: None,
        available_tabs: Vec::new(),
        tab: Tab::Skills,
        current_view: View::List,
        focus: FocusArea::Content,
        cli_selection_index: 0,
        should_quit: false,
        theme: crate::theme::Theme::default(),
        components: Vec::new(),
        list_index: 0,
        tree_views: std::collections::HashMap::new(),
        mcp_servers: Vec::new(),
        mcp_index: 0,
        mcp_scope: crate::mcp::McpScope::default(),
        mcp_project_path: String::new(),
        plugins: Vec::new(),
        plugin_index: 0,
        diff_content: None,
        diff_scroll: 0,
        help_scroll: 0,
        source_dir: std::path::PathBuf::new(),
        sources: Vec::new(),
        dest_dir: std::path::PathBuf::new(),
        status_message: None,
        current_output_style: None,
        current_statusline: None,
        processing_progress: None,
        processing_total: None,
        processing_log: Vec::new(),
        processing_queue: Vec::new(),
        is_removing: false,
        animation_frame: 0,
        needs_refresh: false,
        refreshing: false,
        processing_complete: false,
        cancelling: false,
        env_input_server_idx: None,
        env_input_vars: Vec::new(),
        env_input_current: 0,
        env_input_buffer: String::new(),
        env_input_values: Vec::new(),
        project_path_buffer: String::new(),
        source_entries: Vec::new(),
        source_auto_update: false,
        source_list_index: 0,
        source_add_kind: None,
        source_input_buffer: String::new(),
        source_edit_index: None,
        source_sync_status: None,
        source_sync_cancel_tx: None,
        source_input_error: None,
        source_pending_url: String::new(),
        source_pending_branch: None,
        source_pending_root: None,
        source_sync_rx: None,
    }
}
