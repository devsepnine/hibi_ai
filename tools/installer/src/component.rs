use std::path::{Path, PathBuf};
use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ComponentType {
    Agents,
    Commands,
    Contexts,
    Rules,
    Skills,
    Hooks,
    OutputStyles,
    Statusline,
    ConfigFile,
}

impl ComponentType {
    pub fn display_name(&self) -> &str {
        match self {
            Self::Agents => "agents",
            Self::Commands => "commands",
            Self::Contexts => "contexts",
            Self::Rules => "rules",
            Self::Skills => "skills",
            Self::Hooks => "hooks",
            Self::OutputStyles => "output-styles",
            Self::Statusline => "statusline",
            Self::ConfigFile => "config",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum InstallStatus {
    New,
    Modified,
    Unchanged,
    Managed,
}

impl InstallStatus {
    pub fn display(&self) -> &str {
        match self {
            Self::New => "new",
            Self::Modified => "modified",
            Self::Unchanged => "installed",
            Self::Managed => "managed",
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct HookConfig {
    pub name: String,
    pub event: String,
    #[serde(rename = "type")]
    pub hook_type: String,
    #[serde(default)]
    pub timeout: Option<u32>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub deprecated: Option<bool>,
}

impl HookConfig {
    /// Returns whether this hook is deprecated
    pub fn is_deprecated(&self) -> bool {
        self.deprecated.unwrap_or(false)
    }

    /// Returns the platform-specific binary name for this hook
    pub fn binary_name(&self) -> String {
        if cfg!(windows) {
            format!("{}.exe", self.name)
        } else if cfg!(target_os = "macos") {
            format!("{}_macos", self.name)
        } else {
            format!("{}_linux", self.name)
        }
    }

    /// Returns the full command path for this hook in settings.json
    pub fn hook_command_path(&self, dest_dir: &Path) -> String {
        let binary_name = self.binary_name();

        if cfg!(windows) {
            dest_dir
                .join("hooks")
                .join(&binary_name)
                .to_string_lossy()
                .to_string()
        } else {
            format!("~/.claude/hooks/{}", binary_name)
        }
    }
}

#[derive(Clone, Debug)]
pub struct Component {
    pub component_type: ComponentType,
    pub name: String,
    pub source_path: PathBuf,
    pub dest_path: PathBuf,
    pub selected: bool,
    pub status: InstallStatus,
    pub hook_config: Option<HookConfig>,
    pub source_name: String,
}

impl Component {
    pub fn new(
        component_type: ComponentType,
        name: String,
        source_path: PathBuf,
        dest_path: PathBuf,
        status: InstallStatus,
    ) -> Self {
        Self {
            component_type,
            name,
            source_path,
            dest_path,
            selected: status != InstallStatus::Unchanged,
            status,
            hook_config: None,
            source_name: "bundled".to_string(),
        }
    }

    pub fn with_hook_config(mut self, config: HookConfig) -> Self {
        self.hook_config = Some(config);
        self
    }

    pub fn display_name(&self) -> String {
        format!("{}/{}", self.component_type.display_name(), self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_config_deprecated_true() {
        let yaml = r#"
name: test-hook
event: PreToolUse
type: command
deprecated: true
"#;
        let config: HookConfig = serde_yaml::from_str(yaml).unwrap();
        assert!(config.is_deprecated());
        assert_eq!(config.deprecated, Some(true));
    }

    #[test]
    fn test_hook_config_deprecated_false() {
        let yaml = r#"
name: test-hook
event: PreToolUse
type: command
deprecated: false
"#;
        let config: HookConfig = serde_yaml::from_str(yaml).unwrap();
        assert!(!config.is_deprecated());
    }

    #[test]
    fn test_hook_config_deprecated_missing() {
        let yaml = r#"
name: test-hook
event: PreToolUse
type: command
"#;
        let config: HookConfig = serde_yaml::from_str(yaml).unwrap();
        assert!(!config.is_deprecated());
        assert_eq!(config.deprecated, None);
    }
}
