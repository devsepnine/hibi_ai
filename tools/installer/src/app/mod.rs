mod types;
mod navigation;
mod selection;
mod processing;
mod input;
mod settings;
pub mod sources;
mod source_wizard;

pub use types::{TargetCli, Tab, View};

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use anyhow::Result;

use crate::component::{Component, ComponentType};
use crate::mcp::{McpServer, McpScope};
use crate::plugin::Plugin;
use crate::source::{ResolvedSource, SourceEntry, SourceKind};
use crate::tree::TreeView;
use crate::theme::Theme;

pub struct App {
    pub target_cli: Option<TargetCli>,
    pub available_tabs: Vec<Tab>,
    pub tab: Tab,
    pub current_view: View,
    pub should_quit: bool,
    pub theme: Theme,

    pub components: Vec<Component>,
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
    pub env_input_server_idx: Option<usize>,   // Index of MCP server being configured
    pub env_input_vars: Vec<String>,           // List of env var names to collect
    pub env_input_current: usize,              // Current env var index
    pub env_input_buffer: String,              // Current input text
    pub env_input_values: Vec<(String, String)>, // Collected (name, value) pairs

    // Project path input state (for local scope MCP)
    pub project_path_buffer: String,           // Current project path input

    // Sources management state
    pub source_entries: Vec<SourceEntry>,       // Raw config entries (from YAML)
    pub source_auto_update: bool,              // auto_update flag
    pub source_list_index: usize,              // Cursor in sources list (0 = bundled)
    pub source_add_kind: Option<SourceKind>,   // Git or Local (wizard selection)
    pub source_input_buffer: String,           // Text input buffer (URL/path/branch)
    pub source_edit_index: Option<usize>,      // Some(idx) when editing existing source
    pub source_sync_status: Option<String>,    // Status message after sync
    pub source_input_error: Option<String>,    // Validation error for current input
    pub source_pending_url: String,            // URL saved between wizard steps (Git flow)
    pub source_pending_branch: Option<String>, // Branch saved between wizard steps
    pub source_pending_root: Option<String>,   // Root saved between wizard steps
    pub source_sync_rx: Option<std::sync::mpsc::Receiver<(Vec<ResolvedSource>, Vec<String>)>>,
}

impl App {
    pub fn new() -> Result<Self> {
        let source_dir = find_source_dir()?;
        let resolve_result = crate::source::resolve_all_sources(&source_dir)?;
        let sources = resolve_result.sources;
        // Warnings stored for display after TUI initializes (eprintln would corrupt TUI)
        let init_warnings = if resolve_result.warnings.is_empty() {
            None
        } else {
            Some(resolve_result.warnings.join("; "))
        };
        let (source_entries, source_auto_update) = crate::source::config::load_config()
            .unwrap_or((Vec::new(), true));
        // Start with temporary dest_dir, will be set after CLI selection
        let dest_dir = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?
            .join(".claude");

        // Default project path to current directory
        let default_project = std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        Ok(Self {
            target_cli: None,
            available_tabs: Vec::new(), // Will be set after CLI selection
            tab: Tab::Agents,
            current_view: View::CliSelection,
            should_quit: false,
            theme: Theme::default(),
            components: Vec::new(),
            list_index: 0,
            tree_views: HashMap::new(),
            mcp_servers: Vec::new(),
            mcp_index: 0,
            mcp_scope: McpScope::default(),
            mcp_project_path: default_project.clone(),
            plugins: Vec::new(),
            plugin_index: 0,
            diff_content: None,
            diff_scroll: 0,
            source_dir,
            sources,
            dest_dir,
            status_message: init_warnings,
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
            project_path_buffer: default_project,
            source_entries,
            source_auto_update,
            source_list_index: 0,
            source_add_kind: None,
            source_input_buffer: String::new(),
            source_edit_index: None,
            source_sync_status: None,
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

        // Set available tabs based on CLI
        self.available_tabs = Tab::for_cli(cli);

        // Switch to first available tab
        self.tab = self.available_tabs.first().copied().unwrap_or(Tab::Skills);

        // Switch to loading view - actual scanning will be done in background
        self.current_view = View::Loading;

        Ok(())
    }

    pub fn finish_loading(
        &mut self,
        components: Vec<Component>,
        mcp_servers: Vec<McpServer>,
        plugins: Vec<Plugin>,
        cleaned_hooks: Vec<String>,
    ) {
        self.components = components;
        self.mcp_servers = mcp_servers;
        self.plugins = plugins;

        // Read current settings
        let (current_output_style, current_statusline) = read_current_settings(&self.dest_dir);
        self.current_output_style = current_output_style;
        self.current_statusline = current_statusline;

        // Build tree views
        self.tree_views = build_tree_views(&self.components);

        // Switch to list view
        self.current_view = View::List;
        if !cleaned_hooks.is_empty() {
            self.status_message = Some(format!(
                "Auto-cleaned {} deprecated hook(s): {}",
                cleaned_hooks.len(),
                cleaned_hooks.join(", ")
            ));
        } else if let Some(cli) = self.target_cli {
            self.status_message = Some(format!("Selected {}", cli.display_name()));
        }
    }

    pub fn has_multiple_sources(&self) -> bool {
        self.sources.len() > 1
    }

    /// Get components filtered by current tab
    pub fn current_components(&self) -> Vec<(usize, &Component)> {
        if let Some(comp_type) = self.tab.to_component_type() {
            self.components
                .iter()
                .enumerate()
                .filter(|(_, c)| c.component_type == comp_type)
                .collect()
        } else {
            Vec::new()
        }
    }
}

fn read_current_settings(dest_dir: &Path) -> (Option<String>, Option<String>) {
    use serde_json::Value;

    let settings_path = dest_dir.join("settings.json");
    if !settings_path.exists() {
        return (None, None);
    }

    let content = match std::fs::read_to_string(&settings_path) {
        Ok(c) => c,
        Err(_) => return (None, None),
    };

    let settings: Value = match serde_json::from_str(&content) {
        Ok(s) => s,
        Err(_) => return (None, None),
    };

    let output_style = settings.get("outputStyle")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let statusline = settings.get("statusLine")
        .and_then(|v| v.get("command"))
        .and_then(|v| v.as_str())
        .map(|s| {
            // Extract filename from path like "~/.claude/statusline/statusline.sh"
            // Handle both forward slash and backslash for cross-platform compatibility
            s.rsplit(['/', '\\']).next().unwrap_or(s).to_string()
        });

    (output_style, statusline)
}

fn find_source_dir() -> Result<PathBuf> {
    // Try to find source dir relative to executable
    let exe_dir = std::env::current_exe()?
        .parent()
        .map(|p| p.to_path_buf())
        .ok_or_else(|| anyhow::anyhow!("Cannot get executable directory"))?;

    // Check various possible locations
    let candidates = [
        exe_dir.clone(),  // Scoop: exe and config files in same directory
        exe_dir.join("../share/hibi"),  // Homebrew: /opt/homebrew/bin -> /opt/homebrew/share/hibi
        exe_dir.join("../share/hibi-ai"),  // Homebrew alternative
        exe_dir.join("../../.."),  // From target/release
        exe_dir.join("../.."),     // From target
        std::env::current_dir()?,  // Current directory
        std::env::current_dir()?.join("config/ai/claude"), // From dotfiles root
    ];

    for candidate in candidates {
        let resolved = candidate.canonicalize().unwrap_or(candidate);
        if resolved.join("agents").exists() && resolved.join("settings.json").exists() {
            return Ok(resolved);
        }
    }

    // Default: look for config/ai/claude in current dir
    let default = std::env::current_dir()?.join("config/ai/claude");
    if default.exists() {
        return Ok(default);
    }

    anyhow::bail!("Cannot find source directory. Run from dotfiles root or config/ai/claude/tools/installer")
}

pub(crate) fn build_tree_views(components: &[Component]) -> HashMap<Tab, TreeView> {
    let mut tree_views = HashMap::new();

    // Build tree view for each component-based tab
    let component_tabs = [
        (Tab::Agents, ComponentType::Agents),
        (Tab::Commands, ComponentType::Commands),
        (Tab::Contexts, ComponentType::Contexts),
        (Tab::Rules, ComponentType::Rules),
        (Tab::Skills, ComponentType::Skills),
        (Tab::Hooks, ComponentType::Hooks),
        (Tab::OutputStyles, ComponentType::OutputStyles),
        (Tab::Statusline, ComponentType::Statusline),
        (Tab::Config, ComponentType::ConfigFile),
    ];

    for (tab, comp_type) in component_tabs {
        let filtered: Vec<(usize, &Component)> = components
            .iter()
            .enumerate()
            .filter(|(_, c)| c.component_type == comp_type)
            .collect();

        let tree = TreeView::build_from_components(components, &filtered);
        tree_views.insert(tab, tree);
    }

    tree_views
}
