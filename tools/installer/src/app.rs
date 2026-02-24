use std::collections::HashMap;
use std::path::{Path, PathBuf};
use anyhow::Result;

use crate::component::Component;
use crate::mcp::{McpServer, McpScope};
use crate::plugin::Plugin;
use crate::fs;
use crate::component::ComponentType;
use crate::tree::TreeView;
use crate::theme::Theme;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TargetCli {
    Claude,
    Codex,
}

impl TargetCli {
    pub fn display_name(&self) -> &str {
        match self {
            Self::Claude => "Claude Code",
            Self::Codex => "Codex CLI",
        }
    }

    pub fn config_dir_name(&self) -> &str {
        match self {
            Self::Claude => ".claude",
            Self::Codex => ".codex",
        }
    }

    pub fn get_dest_dir(&self) -> Result<PathBuf> {
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;
        Ok(home.join(self.config_dir_name()))
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Tab {
    Agents,
    Commands,
    Contexts,
    Rules,
    Skills,
    Hooks,
    OutputStyles,
    Statusline,
    Config,
    McpServers,
    Plugins,
}

impl Tab {
    pub fn all() -> &'static [Tab] {
        &[
            Tab::Agents,
            Tab::Commands,
            Tab::Contexts,
            Tab::Rules,
            Tab::Skills,
            Tab::Hooks,
            Tab::OutputStyles,
            Tab::Statusline,
            Tab::Config,
            Tab::McpServers,
            Tab::Plugins,
        ]
    }

    pub fn for_cli(cli: TargetCli) -> Vec<Tab> {
        match cli {
            TargetCli::Claude => Self::all().to_vec(),
            TargetCli::Codex => vec![Tab::Skills, Tab::Config, Tab::McpServers],
        }
    }

    pub fn display_name(&self) -> &str {
        match self {
            Tab::Agents => "Agents",
            Tab::Commands => "Commands",
            Tab::Contexts => "Contexts",
            Tab::Rules => "Rules",
            Tab::Skills => "Skills",
            Tab::Hooks => "Hooks",
            Tab::OutputStyles => "Styles",
            Tab::Statusline => "Statusline",
            Tab::Config => "Config",
            Tab::McpServers => "MCP",
            Tab::Plugins => "Plugins",
        }
    }

    pub fn to_component_type(&self) -> Option<ComponentType> {
        match self {
            Tab::Agents => Some(ComponentType::Agents),
            Tab::Commands => Some(ComponentType::Commands),
            Tab::Contexts => Some(ComponentType::Contexts),
            Tab::Rules => Some(ComponentType::Rules),
            Tab::Skills => Some(ComponentType::Skills),
            Tab::Hooks => Some(ComponentType::Hooks),
            Tab::OutputStyles => Some(ComponentType::OutputStyles),
            Tab::Statusline => Some(ComponentType::Statusline),
            Tab::Config => Some(ComponentType::ConfigFile),
            Tab::McpServers => None,
            Tab::Plugins => None,
        }
    }

}

#[derive(Clone, Copy, PartialEq)]
pub enum View {
    CliSelection,
    Loading,
    List,
    Diff,
    EnvInput,
    ProjectPath,
    Installing,
}

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

    // Env input state (for MCP servers requiring env vars)
    pub env_input_server_idx: Option<usize>,   // Index of MCP server being configured
    pub env_input_vars: Vec<String>,           // List of env var names to collect
    pub env_input_current: usize,              // Current env var index
    pub env_input_buffer: String,              // Current input text
    pub env_input_values: Vec<(String, String)>, // Collected (name, value) pairs

    // Project path input state (for local scope MCP)
    pub project_path_buffer: String,           // Current project path input
}

impl App {
    pub fn new() -> Result<Self> {
        let source_dir = find_source_dir()?;
        // Start with temporary dest_dir, will be set after CLI selection
        let dest_dir = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?
            .join(".claude");

        // Initialize with empty data, will scan after CLI selection
        let components = Vec::new();
        let mcp_servers = Vec::new();
        let plugins = Vec::new();
        let tree_views = HashMap::new();

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
            components,
            list_index: 0,
            tree_views,
            mcp_servers,
            mcp_index: 0,
            mcp_scope: McpScope::default(),
            mcp_project_path: default_project.clone(),
            plugins,
            plugin_index: 0,
            diff_content: None,
            diff_scroll: 0,
            source_dir,
            dest_dir,
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
            env_input_server_idx: None,
            env_input_vars: Vec::new(),
            env_input_current: 0,
            env_input_buffer: String::new(),
            env_input_values: Vec::new(),
            project_path_buffer: default_project,
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

    pub fn finish_loading(&mut self, components: Vec<Component>, mcp_servers: Vec<McpServer>, plugins: Vec<Plugin>) {
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
        if let Some(cli) = self.target_cli {
            self.status_message = Some(format!("Selected {}", cli.display_name()));
        }
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

    pub fn next_tab(&mut self) {
        if let Some(current_idx) = self.available_tabs.iter().position(|t| *t == self.tab) {
            let next_idx = (current_idx + 1) % self.available_tabs.len();
            self.tab = self.available_tabs[next_idx];
            self.list_index = 0;
        }
    }

    pub fn prev_tab(&mut self) {
        if let Some(current_idx) = self.available_tabs.iter().position(|t| *t == self.tab) {
            let prev_idx = if current_idx == 0 {
                self.available_tabs.len() - 1
            } else {
                current_idx - 1
            };
            self.tab = self.available_tabs[prev_idx];
            self.list_index = 0;
        }
    }

    pub fn set_tab(&mut self, idx: usize) {
        if let Some(&tab) = self.available_tabs.get(idx) {
            self.tab = tab;
            self.list_index = 0;
        }
    }

    pub fn next_item(&mut self) {
        if self.tab == Tab::McpServers {
            let len = self.mcp_servers.len();
            if len > 0 {
                self.mcp_index = (self.mcp_index + 1) % len;
            }
        } else if self.tab == Tab::Plugins {
            let len = self.plugins.len();
            if len > 0 {
                self.plugin_index = (self.plugin_index + 1) % len;
            }
        } else if let Some(tree) = self.tree_views.get_mut(&self.tab) {
            tree.next();
        }
    }

    pub fn prev_item(&mut self) {
        if self.tab == Tab::McpServers {
            let len = self.mcp_servers.len();
            if len > 0 {
                self.mcp_index = if self.mcp_index == 0 { len - 1 } else { self.mcp_index - 1 };
            }
        } else if self.tab == Tab::Plugins {
            let len = self.plugins.len();
            if len > 0 {
                self.plugin_index = if self.plugin_index == 0 { len - 1 } else { self.plugin_index - 1 };
            }
        } else if let Some(tree) = self.tree_views.get_mut(&self.tab) {
            tree.prev();
        }
    }

    /// Get the actual component index in self.components for current selection
    pub fn selected_component_index(&self) -> Option<usize> {
        if let Some(tree) = self.tree_views.get(&self.tab) {
            tree.current_component_idx()
        } else {
            // Fallback for non-tree tabs
            let filtered = self.current_components();
            filtered.get(self.list_index).map(|(idx, _)| *idx)
        }
    }

    /// Check if cursor is on a folder
    pub fn is_cursor_on_folder(&self) -> bool {
        self.tree_views.get(&self.tab)
            .map(|t| t.is_on_folder())
            .unwrap_or(false)
    }

    /// Check if current folder is expanded
    pub fn is_current_folder_expanded(&self) -> bool {
        self.tree_views.get(&self.tab)
            .map(|t| t.is_current_folder_expanded())
            .unwrap_or(false)
    }

    /// Toggle expand/collapse for current folder
    pub fn toggle_folder_expand(&mut self) {
        if let Some(tree) = self.tree_views.get_mut(&self.tab) {
            tree.toggle_expand();
        }
    }

    /// Expand current folder
    pub fn expand_folder(&mut self) {
        if let Some(tree) = self.tree_views.get_mut(&self.tab) {
            tree.expand();
        }
    }

    /// Collapse current folder
    pub fn collapse_folder(&mut self) {
        if let Some(tree) = self.tree_views.get_mut(&self.tab) {
            tree.collapse();
        }
    }

    /// Collapse parent folder (when cursor is on a file or subfolder)
    pub fn collapse_parent_folder(&mut self) {
        if let Some(tree) = self.tree_views.get_mut(&self.tab) {
            tree.collapse_parent();
        }
    }

    /// Get current tree view
    pub fn get_tree_view(&self) -> Option<&TreeView> {
        self.tree_views.get(&self.tab)
    }

    pub fn toggle_mcp_scope(&mut self) {
        self.mcp_scope = self.mcp_scope.toggle();
        if self.mcp_scope == McpScope::Local {
            // Show project path input dialog
            self.project_path_buffer = self.mcp_project_path.clone();
            self.current_view = View::ProjectPath;
        } else {
            self.status_message = Some(format!("MCP scope: {}", self.mcp_scope.display()));
        }
    }

    pub fn project_path_char(&mut self, c: char) {
        self.project_path_buffer.push(c);
    }

    pub fn project_path_backspace(&mut self) {
        self.project_path_buffer.pop();
    }

    pub fn project_path_submit(&mut self) {
        if !self.project_path_buffer.is_empty() {
            self.mcp_project_path = self.project_path_buffer.clone();
            self.status_message = Some(format!("MCP scope: local ({})", self.mcp_project_path));
        }
        self.current_view = View::List;
    }

    pub fn project_path_cancel(&mut self) {
        // Revert to user scope if cancelled
        self.mcp_scope = McpScope::User;
        self.status_message = Some("MCP scope: user".to_string());
        self.current_view = View::List;
    }

    pub fn toggle_selected(&mut self) {
        if self.tab == Tab::McpServers {
            if let Some(m) = self.mcp_servers.get_mut(self.mcp_index) {
                m.selected = !m.selected;
            }
        } else if self.tab == Tab::Plugins {
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
                let all_selected = indices.iter()
                    .all(|&idx| self.components.get(idx).map(|c| c.selected).unwrap_or(false));

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

    pub fn select_all(&mut self) {
        if self.tab == Tab::McpServers {
            for m in &mut self.mcp_servers {
                m.selected = true;
            }
        } else if self.tab == Tab::Plugins {
            for p in &mut self.plugins {
                p.selected = true;
            }
        } else if let Some(comp_type) = self.tab.to_component_type() {
            for c in &mut self.components {
                if c.component_type == comp_type {
                    c.selected = true;
                }
            }
        }
    }

    pub fn deselect_all(&mut self) {
        if self.tab == Tab::McpServers {
            for m in &mut self.mcp_servers {
                m.selected = false;
            }
        } else if self.tab == Tab::Plugins {
            for p in &mut self.plugins {
                p.selected = false;
            }
        } else if let Some(comp_type) = self.tab.to_component_type() {
            for c in &mut self.components {
                if c.component_type == comp_type {
                    c.selected = false;
                }
            }
        }
    }

    pub fn show_diff(&mut self) -> Result<()> {
        if self.tab == Tab::McpServers || self.tab == Tab::Plugins {
            return Ok(());
        }

        if let Some(idx) = self.selected_component_index() {
            if let Some(c) = self.components.get(idx) {
                let diff = fs::diff::compare_files(&c.source_path, &c.dest_path)?;
                self.diff_content = Some(diff);
                self.diff_scroll = 0;
                self.current_view = View::Diff;
            }
        }
        Ok(())
    }

    pub fn close_diff(&mut self) {
        self.diff_content = None;
        self.current_view = View::List;
    }

    pub fn scroll_diff_down(&mut self) {
        self.diff_scroll = self.diff_scroll.saturating_add(1);
    }

    pub fn scroll_diff_up(&mut self) {
        self.diff_scroll = self.diff_scroll.saturating_sub(1);
    }

    pub fn install_selected(&mut self) -> Result<()> {
        // Build install queue
        let indices: Vec<usize> = if self.tab == Tab::McpServers {
            self.mcp_servers
                .iter()
                .enumerate()
                .filter(|(_, m)| m.selected)
                .map(|(i, _)| i)
                .collect()
        } else if self.tab == Tab::Plugins {
            self.plugins
                .iter()
                .enumerate()
                .filter(|(_, p)| p.selected)
                .map(|(i, _)| i)
                .collect()
        } else if let Some(comp_type) = self.tab.to_component_type() {
            self.components
                .iter()
                .enumerate()
                .filter(|(_, c)| c.selected && c.component_type == comp_type)
                .map(|(i, _)| i)
                .collect()
        } else {
            Vec::new()
        };

        if indices.is_empty() {
            self.status_message = Some("No items selected".to_string());
            return Ok(());
        }

        // For MCP servers, check if any have missing env vars
        if self.tab == Tab::McpServers {
            for &idx in &indices {
                if let Some(server) = self.mcp_servers.get(idx) {
                    let missing: Vec<String> = server.def.env.iter()
                        .filter(|e| std::env::var(e).is_err())
                        .cloned()
                        .collect();

                    if !missing.is_empty() {
                        // Start env input for this server
                        self.processing_queue = indices;
                        self.start_env_input(idx, missing);
                        return Ok(());
                    }
                }
            }
        }

        // Initialize install state (no env input needed)
        self.processing_queue = indices;
        self.processing_total = Some(self.processing_queue.len());
        self.processing_progress = Some(0);
        self.processing_log.clear();
        self.processing_log.push(format!("Starting installation of {} items...", self.processing_queue.len()));
        self.is_removing = false;
        self.current_view = View::Installing;

        Ok(())
    }

    fn start_env_input(&mut self, server_idx: usize, missing_vars: Vec<String>) {
        self.env_input_server_idx = Some(server_idx);
        self.env_input_vars = missing_vars;
        self.env_input_current = 0;
        self.env_input_buffer.clear();
        self.env_input_values.clear();
        self.current_view = View::EnvInput;
    }

    pub fn env_input_char(&mut self, c: char) {
        self.env_input_buffer.push(c);
    }

    pub fn env_input_backspace(&mut self) {
        self.env_input_buffer.pop();
    }

    pub fn env_input_submit(&mut self) -> Result<()> {
        if self.env_input_buffer.is_empty() {
            return Ok(()); // Don't allow empty values
        }

        // Save current value
        let var_name = self.env_input_vars[self.env_input_current].clone();
        let value = self.env_input_buffer.clone();
        self.env_input_values.push((var_name, value));
        self.env_input_buffer.clear();
        self.env_input_current += 1;

        // Check if all vars collected
        if self.env_input_current >= self.env_input_vars.len() {
            // All env vars collected, continue with installation
            self.continue_mcp_install()?;
        }

        Ok(())
    }

    pub fn env_input_cancel(&mut self) {
        self.env_input_server_idx = None;
        self.env_input_vars.clear();
        self.env_input_values.clear();
        self.env_input_buffer.clear();
        self.processing_queue.clear();
        self.current_view = View::List;
    }

    fn continue_mcp_install(&mut self) -> Result<()> {
        // Initialize install state
        self.processing_total = Some(self.processing_queue.len());
        self.processing_progress = Some(0);
        self.processing_log.clear();
        self.processing_log.push(format!("Starting installation of {} items...", self.processing_queue.len()));
        self.is_removing = false;
        self.current_view = View::Installing;
        Ok(())
    }

    pub fn current_env_var(&self) -> Option<&str> {
        self.env_input_vars.get(self.env_input_current).map(|s| s.as_str())
    }

    pub fn current_env_server_name(&self) -> Option<&str> {
        self.env_input_server_idx
            .and_then(|idx| self.mcp_servers.get(idx))
            .map(|s| s.def.name.as_str())
    }

    pub fn start_finish_processing(&mut self) {
        let action = if self.is_removing { "Removal" } else { "Installation" };
        self.processing_log.push(format!("[OK] {} complete!", action));
        self.processing_log.push("".to_string());  // Empty line for spacing
        self.processing_log.push("Refreshing status...".to_string());
        self.needs_refresh = true;
    }

    pub fn apply_refresh_result(&mut self, components: Vec<Component>, mcp_servers: Vec<McpServer>, plugins: Vec<Plugin>) {
        self.components = components;
        self.mcp_servers = mcp_servers;
        self.plugins = plugins;

        // Rebuild tree views with new components
        self.tree_views = build_tree_views(&self.components);

        let verb = if self.is_removing { "Removed" } else { "Installed" };
        self.status_message = Some(format!("{} {} items", verb, self.processing_total.unwrap_or(0)));
        self.processing_log.push("[OK] Status refresh complete!".to_string());
        self.needs_refresh = false;
        self.refreshing = false;
        self.processing_complete = true;
    }

    pub fn close_processing(&mut self) {
        self.current_view = View::List;
        self.processing_queue.clear();
        self.processing_progress = None;
        self.processing_total = None;
        self.processing_log.clear();
        self.is_removing = false;
        self.needs_refresh = false;
        self.refreshing = false;
        self.processing_complete = false;
    }

    pub fn tick(&mut self) {
        // Update animation frame for spinner
        self.animation_frame = (self.animation_frame + 1) % 10;
    }

    pub fn remove_selected(&mut self) -> Result<()> {
        // Build remove queue
        let indices: Vec<usize> = if self.tab == Tab::McpServers {
            self.mcp_servers
                .iter()
                .enumerate()
                .filter(|(_, m)| m.selected)
                .map(|(i, _)| i)
                .collect()
        } else if self.tab == Tab::Plugins {
            self.plugins
                .iter()
                .enumerate()
                .filter(|(_, p)| p.selected)
                .map(|(i, _)| i)
                .collect()
        } else if let Some(comp_type) = self.tab.to_component_type() {
            self.components
                .iter()
                .enumerate()
                .filter(|(_, c)| c.selected && c.component_type == comp_type)
                .map(|(i, _)| i)
                .collect()
        } else {
            Vec::new()
        };

        if indices.is_empty() {
            self.status_message = Some("No items selected".to_string());
            return Ok(());
        }

        // Initialize remove state
        self.processing_queue = indices;
        self.processing_total = Some(self.processing_queue.len());
        self.processing_progress = Some(0);
        self.processing_log.clear();
        self.processing_log.push(format!("Starting removal of {} items...", self.processing_queue.len()));
        self.is_removing = true;
        self.current_view = View::Installing;

        Ok(())
    }

    pub fn set_default_style(&mut self) -> Result<()> {
        // Only works for OutputStyles tab
        if self.tab != Tab::OutputStyles {
            self.status_message = Some("Switch to Styles tab to set default".to_string());
            return Ok(());
        }

        if let Some(idx) = self.selected_component_index() {
            if let Some(component) = self.components.get(idx) {
                // Get style name without extension
                let style_name = component.name
                    .strip_suffix(".md")
                    .unwrap_or(&component.name)
                    .to_string();

                fs::installer::set_output_style(&self.dest_dir, &style_name)?;
                self.current_output_style = Some(style_name.clone());
                self.status_message = Some(format!("Set default output style: {}", style_name));
            }
        }
        Ok(())
    }

    pub fn set_statusline(&mut self) -> Result<()> {
        // Only works for Statusline tab
        if self.tab != Tab::Statusline {
            self.status_message = Some("Switch to Statusline tab to set default".to_string());
            return Ok(());
        }

        if let Some(idx) = self.selected_component_index() {
            if let Some(component) = self.components.get(idx) {
                fs::installer::set_statusline(&self.dest_dir, &component.name)?;
                self.current_statusline = Some(component.name.clone());
                self.status_message = Some(format!("Set statusline: {}", component.name));
            }
        }
        Ok(())
    }

    pub fn unset_default_style(&mut self) -> Result<()> {
        // Only works for OutputStyles tab
        if self.tab != Tab::OutputStyles {
            self.status_message = Some("Switch to OutputStyles tab to unset default".to_string());
            return Ok(());
        }

        fs::installer::unset_output_style(&self.dest_dir)?;
        self.current_output_style = None;
        self.status_message = Some("Unset default output style".to_string());
        Ok(())
    }

    pub fn unset_statusline(&mut self) -> Result<()> {
        // Only works for Statusline tab
        if self.tab != Tab::Statusline {
            self.status_message = Some("Switch to Statusline tab to unset default".to_string());
            return Ok(());
        }

        fs::installer::unset_statusline(&self.dest_dir)?;
        self.current_statusline = None;
        self.status_message = Some("Unset statusline".to_string());
        Ok(())
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

fn build_tree_views(components: &[Component]) -> HashMap<Tab, TreeView> {
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
