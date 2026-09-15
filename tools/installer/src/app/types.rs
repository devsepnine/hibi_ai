use std::path::PathBuf;

use anyhow::Result;

use crate::component::ComponentType;

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

/// Which pane the keyboard is currently driving.
///
/// `1`/`2` address the panes directly and `Tab`/`Shift+Tab` toggles between
/// them. Movement keys (`h`/`l`/`←`/`→`, `j`/`k`/`↑`/`↓`) act on whichever
/// pane is focused, so the same handful of keys serve both tab switching and
/// list navigation.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum FocusArea {
    #[default]
    Content,
    Tabs,
}

impl FocusArea {
    /// The digit key that jumps to this pane.
    ///
    /// The one place the digit is decided: the `cli` dispatch honors it and
    /// every label derives from it, so no label can advertise a key the
    /// dispatch ignores. Tests spell the digits out literally on purpose —
    /// changing this method has to fail loudly rather than quietly rename the
    /// keys everywhere at once.
    pub fn shortcut(self) -> char {
        match self {
            Self::Tabs => '1',
            Self::Content => '2',
        }
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

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum View {
    CliSelection,
    Loading,
    List,
    Diff,
    /// The `?` keybinding reference, overlaid on the List view.
    Help,
    /// The `Esc` "leave for the CLI picker?" prompt, overlaid on the List view.
    ///
    /// Reached only when something is selected — the picker re-scans on the way
    /// back, so leaving with a selection throws it away, and leaving without
    /// one costs nothing worth a keystroke to confirm.
    ConfirmExit,
    EnvInput,
    ProjectPath,
    Preflighting,
    Installing,
    Sources,
    SourceAddType,
    SourceAddUrl,
    SourceAddBranch,
    SourceAddPath,
    SourceAddRoot,
    SourceAddMapTo,
    SourceConfirmRemove,
    SourceSyncing,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SyncStatus {
    Success(String),
    Error(String),
}
