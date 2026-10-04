use std::path::PathBuf;

use anyhow::Result;

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
        let home = crate::paths::require_home_dir()?;
        Ok(home.join(self.config_dir_name()))
    }
}
