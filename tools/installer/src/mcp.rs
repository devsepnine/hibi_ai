use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum McpScope {
    User,
    Local,
}

impl McpScope {
    pub fn display(&self) -> &str {
        match self {
            Self::User => "user",
            Self::Local => "local",
        }
    }

    pub fn toggle(&self) -> Self {
        match self {
            Self::User => Self::Local,
            Self::Local => Self::User,
        }
    }
}

impl Default for McpScope {
    fn default() -> Self {
        Self::User
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum McpType {
    Command,
    Http,
}

impl Default for McpType {
    fn default() -> Self {
        Self::Command
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum McpStatus {
    Unknown,
    Installed,
    NotInstalled,
}

impl McpStatus {
    pub fn display(&self) -> &str {
        match self {
            Self::Unknown => "unknown",
            Self::Installed => "installed",
            Self::NotInstalled => "not installed",
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct McpServerDef {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub r#type: Option<McpType>,
    pub command: Option<String>,
    pub url: Option<String>,
    pub category: String,
    #[serde(default)]
    pub env: Vec<String>,
    pub setup: Option<McpSetup>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum McpSetup {
    Bearer {
        variable: String,
        #[serde(default)]
        if_set: bool,
    },
    FileEnv {
        variable: String,
    },
}

impl McpSetup {
    pub fn variable(&self) -> &str {
        match self {
            Self::Bearer { variable, .. } | Self::FileEnv { variable } => variable,
        }
    }

    pub fn input(&self) -> Option<&str> {
        match self {
            Self::FileEnv { .. } => Some(self.variable()),
            Self::Bearer { .. } => None,
        }
    }

    pub fn hint(&self) -> &'static str {
        match self {
            Self::FileEnv { .. } => {
                "Enter an absolute file path in a writable directory for persistent storage."
            }
            Self::Bearer { .. } => "Set the token in the environment of your MCP client.",
        }
    }

    pub fn accepts(&self, value: &str) -> bool {
        match self {
            Self::FileEnv { .. } => writable_storage_file(value).is_some(),
            Self::Bearer { .. } => false,
        }
    }
}

pub(crate) fn writable_storage_file(value: &str) -> Option<PathBuf> {
    let path = Path::new(value);
    if !path.is_absolute()
        || matches!(
            value.rsplit(std::path::is_separator).next(),
            None | Some("" | "." | "..")
        )
    {
        return None;
    }
    let target = match path.symlink_metadata() {
        Ok(metadata) => {
            // Memory commits by rename. Registering the link itself splits the graph on its first write.
            let target = if metadata.is_symlink() {
                path.canonicalize().ok()?
            } else {
                path.to_owned()
            };
            if !target.is_file() {
                return None;
            }
            std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(&target)
                .ok()?;
            target
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => path.to_owned(),
        Err(_) => return None,
    };
    // Locks and replacement files are created beside the graph that will be registered.
    tempfile::NamedTempFile::new_in(target.parent()?).ok()?;
    Some(target)
}

impl McpServerDef {
    pub fn registration_setup(
        &self,
        env: impl FnOnce(&str) -> Option<std::ffi::OsString>,
    ) -> Option<&McpSetup> {
        self.setup.as_ref().filter(|setup| match setup {
            McpSetup::Bearer {
                variable,
                if_set: true,
            } => env(variable).is_some_and(|value| !value.is_empty()),
            _ => true,
        })
    }

    pub fn inputs(&self) -> impl Iterator<Item = &str> {
        self.env
            .iter()
            .map(String::as_str)
            .chain(self.setup.as_ref().and_then(McpSetup::input))
    }
}

#[derive(Clone, Debug)]
pub struct McpServer {
    pub def: McpServerDef,
    pub selected: bool,
    pub status: McpStatus,
    pub source_name: String,
}

impl McpServer {
    pub fn new(def: McpServerDef, status: McpStatus) -> Self {
        Self {
            def,
            selected: false,
            status,
            source_name: "bundled".to_string(),
        }
    }

    pub fn is_http(&self) -> bool {
        matches!(self.def.r#type, Some(McpType::Http))
    }
}

#[derive(Debug, Deserialize)]
pub struct McpCatalog {
    pub servers: Vec<McpServerDef>,
}
