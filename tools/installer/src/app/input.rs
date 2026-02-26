use anyhow::Result;

use super::types::View;
use super::App;
use crate::mcp::McpScope;

impl App {
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

    pub fn current_env_var(&self) -> Option<&str> {
        self.env_input_vars.get(self.env_input_current).map(|s| s.as_str())
    }

    pub fn current_env_server_name(&self) -> Option<&str> {
        self.env_input_server_idx
            .and_then(|idx| self.mcp_servers.get(idx))
            .map(|s| s.def.name.as_str())
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
}
