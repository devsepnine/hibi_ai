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
    /// File exists in dest_dir but no source produces it (user-added or
    /// orphaned from older hibi versions). source_path = dest_path; never
    /// installed, only removable.
    External,
}

impl InstallStatus {
    pub fn display(&self) -> &str {
        match self {
            Self::New => "new",
            Self::Modified => "modified",
            Self::Unchanged => "installed",
            Self::Managed => "managed",
            Self::External => "external",
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
            // Derive from dest_dir to support both ~/.claude and ~/.codex
            let dir_name = dest_dir.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| ".claude".to_string());
            format!("~/{}/hooks/{}", dir_name, binary_name)
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
        let selected = !matches!(status, InstallStatus::Unchanged | InstallStatus::External);
        Self {
            component_type,
            name,
            source_path,
            dest_path,
            selected,
            status,
            hook_config: None,
            source_name: "bundled".to_string(),
        }
    }

    pub fn with_hook_config(mut self, config: HookConfig) -> Self {
        self.hook_config = Some(config);
        self
    }

    /// Override the default `bundled` source label (used for externals and
    /// any future labeled-source workflow). Mirrors `with_hook_config`.
    pub fn with_source_name(mut self, name: &str) -> Self {
        self.source_name = name.to_string();
        self
    }

    pub fn display_name(&self) -> String {
        format!("{}/{}", self.component_type.display_name(), self.name)
    }

    /// Whether this component is eligible to enter the install queue.
    ///
    /// External components must not be installed (their `source_path` equals
    /// `dest_path`, so copying would be a no-op or self-corruption depending
    /// on the OS). Deprecated hooks are blocked at install time.
    /// Selection and component-type matching are handled by the caller.
    pub fn is_install_eligible(&self) -> bool {
        if self.status == InstallStatus::External {
            return false;
        }
        if let Some(config) = &self.hook_config {
            if config.is_deprecated() {
                return false;
            }
        }
        true
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
        let config: HookConfig = serde_yaml_bw::from_str(yaml).unwrap();
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
        let config: HookConfig = serde_yaml_bw::from_str(yaml).unwrap();
        assert!(!config.is_deprecated());
    }

    #[test]
    fn test_hook_config_deprecated_missing() {
        let yaml = r#"
name: test-hook
event: PreToolUse
type: command
"#;
        let config: HookConfig = serde_yaml_bw::from_str(yaml).unwrap();
        assert!(!config.is_deprecated());
        assert_eq!(config.deprecated, None);
    }

    fn make_component(status: InstallStatus) -> Component {
        Component::new(
            ComponentType::Agents,
            "test.md".to_string(),
            PathBuf::from("/src/test.md"),
            PathBuf::from("/dest/test.md"),
            status,
        )
    }

    #[test]
    fn install_eligible_for_normal_statuses() {
        // New / Modified / Unchanged / Managed must all pass install gate;
        // selection is the caller's concern, not eligibility.
        for status in [
            InstallStatus::New,
            InstallStatus::Modified,
            InstallStatus::Unchanged,
            InstallStatus::Managed,
        ] {
            let c = make_component(status.clone());
            assert!(
                c.is_install_eligible(),
                "{status:?} must be install-eligible"
            );
        }
    }

    #[test]
    fn install_not_eligible_for_external() {
        // External components have source_path == dest_path; copying would
        // self-corrupt. Must be excluded from install regardless of selection.
        let c = make_component(InstallStatus::External);
        assert!(!c.is_install_eligible());
    }

    #[test]
    fn install_not_eligible_for_deprecated_hook() {
        let yaml = r#"
name: old-hook
event: PreToolUse
type: command
deprecated: true
"#;
        let config: HookConfig = serde_yaml_bw::from_str(yaml).unwrap();
        let c = Component::new(
            ComponentType::Hooks,
            "old-hook".to_string(),
            PathBuf::from("/src/old-hook"),
            PathBuf::from("/dest/old-hook"),
            InstallStatus::Unchanged,
        ).with_hook_config(config);

        assert!(!c.is_install_eligible(), "deprecated hooks must be blocked");
    }

    #[test]
    fn with_source_name_overrides_default_bundled_label() {
        // Default label is "bundled"; builder must replace it without
        // disturbing other fields (mirrors with_hook_config behavior).
        let c = Component::new(
            ComponentType::Skills,
            "x.md".to_string(),
            PathBuf::from("/dest/x.md"),
            PathBuf::from("/dest/x.md"),
            InstallStatus::External,
        )
        .with_source_name("external");

        assert_eq!(c.source_name, "external");
        assert_eq!(c.status, InstallStatus::External);
        assert_eq!(c.name, "x.md");
    }

    #[test]
    fn install_eligible_for_active_hook() {
        let yaml = r#"
name: active-hook
event: PreToolUse
type: command
"#;
        let config: HookConfig = serde_yaml_bw::from_str(yaml).unwrap();
        let c = Component::new(
            ComponentType::Hooks,
            "active-hook".to_string(),
            PathBuf::from("/src/active-hook"),
            PathBuf::from("/dest/active-hook"),
            InstallStatus::New,
        ).with_hook_config(config);

        assert!(c.is_install_eligible());
    }
}
