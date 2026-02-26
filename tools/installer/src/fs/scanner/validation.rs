use crate::mcp::McpServerDef;

/// Pattern for safe identifiers: alphanumeric, underscore, hyphen. Must not start with `-`.
pub(super) fn is_safe_identifier(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 100
        && !s.starts_with('-')
        && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

/// Validate that a URL uses HTTPS scheme only.
pub(super) fn is_https_url(s: &str) -> bool {
    s.starts_with("https://") && s.len() > 8
}

/// Validate that a command string doesn't contain shell metacharacters.
pub(super) fn is_safe_command(s: &str) -> bool {
    !s.is_empty() && !s.contains(['&', '|', '>', '<', ';', '`', '$', '(', ')'])
}

/// Validate an MCP server definition from YAML.
/// Returns an error description if invalid, None if valid.
pub(super) fn validate_mcp_server(server: &McpServerDef) -> Option<String> {
    if !is_safe_identifier(&server.name) {
        return Some(format!(
            "MCP server '{}': name must be alphanumeric/underscore/hyphen, 1-100 chars, not starting with '-'",
            server.name
        ));
    }

    if let Some(url) = &server.url {
        if !is_https_url(url) {
            return Some(format!(
                "MCP server '{}': url must use https:// scheme, got '{}'",
                server.name, url
            ));
        }
    }

    if let Some(cmd) = &server.command {
        if !is_safe_command(cmd) {
            return Some(format!(
                "MCP server '{}': command contains disallowed shell metacharacters",
                server.name
            ));
        }
    }

    None
}

/// Validate a plugin definition from YAML.
/// Returns an error description if invalid, None if valid.
pub(super) fn validate_plugin(name: &str, marketplace: &str, source: &str) -> Option<String> {
    if !is_safe_identifier(name) {
        return Some(format!(
            "Plugin '{}': name must be alphanumeric/underscore/hyphen",
            name
        ));
    }

    if !is_safe_identifier(marketplace) {
        return Some(format!(
            "Plugin '{}': marketplace '{}' must be alphanumeric/underscore/hyphen",
            name, marketplace
        ));
    }

    if !is_https_url(source) {
        return Some(format!(
            "Plugin '{}': source must use https:// scheme, got '{}'",
            name, source
        ));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_safe_identifier_valid() {
        assert!(is_safe_identifier("context7"));
        assert!(is_safe_identifier("cloudflare-docs"));
        assert!(is_safe_identifier("my_server_123"));
        assert!(is_safe_identifier("a"));
    }

    #[test]
    fn test_is_safe_identifier_invalid() {
        assert!(!is_safe_identifier(""));
        assert!(!is_safe_identifier("-starts-with-dash"));
        assert!(!is_safe_identifier("has spaces"));
        assert!(!is_safe_identifier("has;semicolon"));
        assert!(!is_safe_identifier("has&amp"));
        assert!(!is_safe_identifier(&"a".repeat(101)));
    }

    #[test]
    fn test_is_https_url_valid() {
        assert!(is_https_url("https://example.com"));
        assert!(is_https_url("https://mcp.sentry.dev/mcp"));
        assert!(is_https_url("https://github.com/repo.git"));
    }

    #[test]
    fn test_is_https_url_invalid() {
        assert!(!is_https_url("http://example.com"));
        assert!(!is_https_url("file:///etc/passwd"));
        assert!(!is_https_url("javascript:alert(1)"));
        assert!(!is_https_url("https://")); // too short
        assert!(!is_https_url(""));
    }

    #[test]
    fn test_is_safe_command_valid() {
        assert!(is_safe_command("npx -y @upstash/context7-mcp"));
        assert!(is_safe_command("uvx mcp-atlassian"));
        assert!(is_safe_command("npx -y @supabase/mcp-server-supabase@latest --project-ref=YOUR_PROJECT_REF"));
    }

    #[test]
    fn test_is_safe_command_invalid() {
        assert!(!is_safe_command(""));
        assert!(!is_safe_command("cmd & echo hacked"));
        assert!(!is_safe_command("cmd | cat /etc/passwd"));
        assert!(!is_safe_command("cmd; rm -rf /"));
        assert!(!is_safe_command("cmd > /tmp/out"));
        assert!(!is_safe_command("cmd < /etc/passwd"));
        assert!(!is_safe_command("echo `whoami`"));
        assert!(!is_safe_command("echo $(whoami)"));
    }

    #[test]
    fn test_validate_mcp_server_valid() {
        let def = McpServerDef {
            name: "context7".to_string(),
            description: "Test".to_string(),
            r#type: None,
            command: Some("npx -y @upstash/context7-mcp".to_string()),
            url: None,
            category: "docs".to_string(),
            env: vec![],
        };
        assert!(validate_mcp_server(&def).is_none());
    }

    #[test]
    fn test_validate_mcp_server_bad_name() {
        let def = McpServerDef {
            name: "bad;name".to_string(),
            description: "Test".to_string(),
            r#type: None,
            command: Some("npx test".to_string()),
            url: None,
            category: "test".to_string(),
            env: vec![],
        };
        assert!(validate_mcp_server(&def).is_some());
    }

    #[test]
    fn test_validate_mcp_server_bad_url() {
        let def = McpServerDef {
            name: "test".to_string(),
            description: "Test".to_string(),
            r#type: None,
            command: None,
            url: Some("file:///etc/passwd".to_string()),
            category: "test".to_string(),
            env: vec![],
        };
        assert!(validate_mcp_server(&def).is_some());
    }

    #[test]
    fn test_validate_mcp_server_bad_command() {
        let def = McpServerDef {
            name: "test".to_string(),
            description: "Test".to_string(),
            r#type: None,
            command: Some("npx test & echo hacked".to_string()),
            url: None,
            category: "test".to_string(),
            env: vec![],
        };
        assert!(validate_mcp_server(&def).is_some());
    }

    #[test]
    fn test_validate_plugin_valid() {
        assert!(validate_plugin("rust-analyzer-lsp", "claude-plugins-official", "https://github.com/repo.git").is_none());
    }

    #[test]
    fn test_validate_plugin_bad_name() {
        assert!(validate_plugin("bad;name", "marketplace", "https://example.com").is_some());
    }

    #[test]
    fn test_validate_plugin_bad_source() {
        assert!(validate_plugin("good-name", "marketplace", "http://insecure.com").is_some());
    }
}
