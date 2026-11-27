mod filter;
mod input;
mod navigation;
mod processing;
mod removal;
mod search;
mod selection;
mod settings;
mod source_wizard;
pub mod sources;
#[cfg(test)]
pub(crate) mod test_support;
mod types;
mod visibility;

pub use crate::target::TargetCli;
#[cfg(any(feature = "gui", test))]
pub use filter::ComponentFilter;
pub use filter::ComponentFilters;
#[cfg(feature = "gui")]
pub(crate) use selection::WorkSelection;
pub(crate) use types::CliChoice;
pub use types::{FocusArea, SyncStatus, Tab, View};
pub use visibility::{HiddenVisibility, HideConfirmation};

use anyhow::Result;
use std::collections::{BTreeSet, HashMap};
use std::path::PathBuf;

use crate::component::Component;
use crate::mcp::{McpScope, McpServer};
use crate::plugin::Plugin;
use crate::source::{ResolvedSource, SourceEntry, SourceKind};
use crate::theme::Theme;
use crate::tree::TreeView;

pub struct App {
    pub target_cli: Option<TargetCli>,
    pub available_tabs: Vec<Tab>,
    pub tab: Tab,
    pub current_view: View,
    pub focus: FocusArea,
    pub(crate) cli_choice: CliChoice,
    pub should_quit: bool,
    pub theme: Theme,

    pub components: Vec<Component>,
    pub component_filter: ComponentFilters,
    pub search_query: String,
    pub hidden_folders: BTreeSet<PathBuf>,
    pub hidden_visibility: HiddenVisibility,
    pub visibility_path: PathBuf,
    pub hide_confirmation: Option<HideConfirmation>,
    pending_removal: Option<removal::RemovalPlan>,
    pub list_index: usize, // Index within current tab's filtered list (legacy, for MCP/Plugins)
    pub tree_views: HashMap<Tab, TreeView>, // Tree views for component tabs

    pub mcp_servers: Vec<McpServer>,
    pub mcp_index: usize,
    pub mcp_scope: McpScope,
    pub mcp_project_path: String, // Project path for local scope

    pub plugins: Vec<Plugin>,
    pub plugin_index: usize,

    pub diff_content: Option<String>,
    pub diff_scroll: u16,

    /// First visible row of the `?` overlay's binding table.
    pub help_scroll: u16,

    pub source_dir: PathBuf,
    pub sources: Vec<ResolvedSource>,
    pub dest_dir: PathBuf,

    pub status_message: Option<String>,

    // Current defaults from settings.json
    pub current_output_style: Option<String>,
    pub current_statusline: Option<String>,

    // Processing state (install/remove)
    pub processing_progress: Option<usize>,
    pub processing_total: Option<usize>,
    pub processing_log: Vec<String>,
    pub processing_queue: Vec<usize>, // Indices of items to process
    pub is_removing: bool,            // true = removing, false = installing
    pub animation_frame: usize,       // For spinner animation
    pub needs_refresh: bool,          // True after processing, before refresh
    pub refreshing: bool,             // True while refresh thread is running
    pub processing_complete: bool,    // True when everything is done (including refresh)
    pub cancelling: bool,             // True when cancel signal sent, waiting for process to stop

    // Env input state (for MCP servers requiring env vars)
    pub env_input_server_idx: Option<usize>, // Index of MCP server being configured
    pub env_input_vars: Vec<String>,         // List of env var names to collect
    pub env_input_current: usize,            // Current env var index
    pub env_input_buffer: String,            // Current input text
    pub env_input_values: Vec<(String, String)>, // Collected (name, value) pairs
    pub mcp_env_values: HashMap<usize, Vec<(String, String)>>,

    // Project path input state (for local scope MCP)
    pub project_path_buffer: String, // Current project path input

    // Sources management state
    pub source_entries: Vec<SourceEntry>, // Raw config entries (from YAML)
    pub source_auto_update: bool,         // auto_update flag
    pub source_list_index: usize,         // Cursor in sources list (0 = bundled)
    pub source_add_kind: Option<SourceKind>, // Git or Local (wizard selection)
    pub source_input_buffer: String,      // Text input buffer (URL/path/branch)
    pub source_edit_index: Option<usize>, // Some(idx) when editing existing source
    pub source_sync_status: Option<SyncStatus>, // Typed status after sync
    pub source_sync_cancel_tx: Option<std::sync::mpsc::Sender<()>>,
    pub source_input_error: Option<String>, // Validation error for current input
    pub source_pending_url: String,         // URL saved between wizard steps (Git flow)
    pub source_pending_branch: Option<String>, // Branch saved between wizard steps
    pub source_pending_root: Option<String>, // Root saved between wizard steps
    pub source_sync_rx: Option<std::sync::mpsc::Receiver<sources::SyncPayload>>,
}

struct InitData {
    source_dir: PathBuf,
    sources: Vec<ResolvedSource>,
    init_warnings: Option<String>,
    source_entries: Vec<SourceEntry>,
    source_auto_update: bool,
    dest_dir: PathBuf,
    default_project: String,
    visibility_path: PathBuf,
}

fn load_init_data() -> Result<InitData> {
    let source_dir = crate::source::find_source_dir()?;

    // One-time cleanup of the orphaned bundled-source cache (now package-embedded).
    // Don't fail init if cleanup errors — collect it as a warning instead.
    let mut warnings = collect_startup_warnings();

    let resolve_result = crate::source::resolve_all_sources(&source_dir)?;
    let sources = resolve_result.sources;
    warnings.extend(resolve_result.warnings);
    let init_warnings = if warnings.is_empty() {
        None
    } else {
        Some(warnings.join("; "))
    };
    let (source_entries, source_auto_update) =
        crate::source::config::load_config().unwrap_or((Vec::new(), true));
    let dest_dir = crate::paths::require_home_dir()?.join(".claude");
    let default_project = std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    Ok(InitData {
        source_dir,
        sources,
        init_warnings,
        source_entries,
        source_auto_update,
        dest_dir,
        default_project,
        visibility_path: crate::paths::require_home_dir()?
            .join(".hibi")
            .join("visibility.json"),
    })
}

/// Run the one-time orphaned-bundled-cache cleanup, returning any non-fatal
/// message as a warning. Never fails init.
fn collect_startup_warnings() -> Vec<String> {
    let mut warnings = Vec::new();
    match crate::source::git::cleanup_bundled_cache() {
        Ok(true) => {
            warnings.push("Removed orphaned bundled cache (~/.hibi/cache/bundled)".to_string())
        }
        Ok(false) => {}
        Err(e) => warnings.push(format!("Failed to clean bundled cache: {}", e)),
    }
    warnings
}

impl App {
    pub fn new() -> Result<Self> {
        let d = load_init_data()?;
        Ok(Self {
            target_cli: None,
            available_tabs: Vec::new(), // Will be set after CLI selection
            tab: Tab::Agents,
            current_view: View::CliSelection,
            focus: FocusArea::default(),
            cli_choice: CliChoice::default(),
            should_quit: false,
            theme: Theme::default(),
            components: Vec::new(),
            component_filter: ComponentFilters::default(),
            search_query: String::new(),
            hidden_folders: BTreeSet::new(),
            hidden_visibility: HiddenVisibility::default(),
            visibility_path: d.visibility_path,
            hide_confirmation: None,
            pending_removal: None,
            list_index: 0,
            tree_views: HashMap::new(),
            mcp_servers: Vec::new(),
            mcp_index: 0,
            mcp_scope: McpScope::default(),
            mcp_project_path: d.default_project.clone(),
            plugins: Vec::new(),
            plugin_index: 0,
            diff_content: None,
            diff_scroll: 0,
            help_scroll: 0,
            source_dir: d.source_dir,
            sources: d.sources,
            dest_dir: d.dest_dir,
            status_message: d.init_warnings,
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
            mcp_env_values: HashMap::new(),
            project_path_buffer: d.default_project,
            source_entries: d.source_entries,
            source_auto_update: d.source_auto_update,
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
        })
    }

    pub fn select_cli(&mut self, cli: TargetCli) -> Result<()> {
        self.target_cli = Some(cli);
        self.dest_dir = cli.get_dest_dir()?;
        self.cli_choice = match cli {
            TargetCli::Claude => CliChoice::Claude,
            TargetCli::Codex => CliChoice::Codex,
        };
        if cli == TargetCli::Codex {
            self.mcp_scope = McpScope::User;
        }

        // Set available tabs based on CLI
        self.available_tabs = Tab::for_cli(cli);

        // Switch to first available tab
        self.tab = self.available_tabs.first().copied().unwrap_or(Tab::Skills);
        self.component_filter = ComponentFilters::default();
        self.search_query.clear();

        // These three index into lists the scan below is about to replace, and a
        // shorter result leaves them past the end. ratatui clamps the highlight
        // it draws but not the index behind it, so `Space` would silently toggle
        // nothing under a cursor the user can plainly see.
        self.list_index = 0;
        self.mcp_index = 0;
        self.plugin_index = 0;

        // Switch to loading view - actual scanning will be done in background
        self.current_view = View::Loading;

        Ok(())
    }

    pub fn finish_loading(
        &mut self,
        components: Vec<Component>,
        mcp_servers: Vec<McpServer>,
        plugins: Vec<Plugin>,
        cleaned_items: Vec<String>,
    ) {
        self.components = components;
        self.mcp_servers = mcp_servers;
        self.plugins = plugins;

        self.refresh_settings_defaults();

        let visibility_warning = self.load_folder_visibility();

        // Build tree views
        self.rebuild_tree_views();

        // Switch to list view
        self.current_view = View::List;
        if !cleaned_items.is_empty() {
            self.status_message = Some(format!(
                "Auto-cleaned {} outdated item(s): {}",
                cleaned_items.len(),
                cleaned_items.join(", ")
            ));
        } else if let Some(cli) = self.target_cli {
            self.status_message = Some(format!("Selected {}", cli.display_name()));
        }
        if let Some(warning) = visibility_warning {
            let message = self.status_message.get_or_insert_with(String::new);
            if !message.is_empty() {
                message.push_str("; ");
            }
            message.push_str(&warning);
        }
    }

    pub fn has_multiple_sources(&self) -> bool {
        self.sources.len() > 1
    }
}
