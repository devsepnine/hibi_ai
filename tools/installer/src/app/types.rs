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
