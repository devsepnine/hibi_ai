use serde::Deserialize;

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
    Installed,
    NotInstalled,
}

impl McpStatus {
    pub fn display(&self) -> &str {
        match self {
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
}

#[derive(Clone, Debug)]
pub struct McpServer {
    pub def: McpServerDef,
    pub selected: bool,
    pub status: McpStatus,
}

impl McpServer {
    pub fn new(def: McpServerDef, status: McpStatus) -> Self {
        Self {
            def,
            selected: false,
            status,
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
