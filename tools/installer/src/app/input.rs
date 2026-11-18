use anyhow::Result;

use super::App;
use super::types::View;
use crate::mcp::McpScope;

impl App {
    pub fn env_input_char(&mut self, c: char) {
        self.env_input_buffer.push(c);
    }

    pub fn env_input_backspace(&mut self) {
        self.env_input_buffer.pop();
    }

    pub fn env_input_submit(&mut self) -> Result<()> {
        if self.current_view != View::EnvInput || self.env_input_buffer.is_empty() {
            return Ok(()); // Don't allow empty values
        }

        // Save current value
        let var_name = self
            .env_input_vars
            .get(self.env_input_current)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("No environment variable is awaiting a value"))?;
        if let Some(setup) = self.current_setup_input() {
            if !setup.accepts(&self.env_input_buffer) {
                self.status_message = Some(setup.hint().into());
                return Ok(());
            }
        }
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
        self.clear_env_inputs();
        self.processing_queue.clear();
        self.current_view = View::List;
    }

    pub fn current_env_var(&self) -> Option<&str> {
        self.env_input_vars
            .get(self.env_input_current)
            .map(|s| s.as_str())
    }

    pub fn current_setup_input(&self) -> Option<&crate::mcp::McpSetup> {
        let setup = self
            .mcp_servers
            .get(self.env_input_server_idx?)?
            .def
            .setup
            .as_ref()?;
        self.current_env_var()
            .filter(|&key| setup.input() == Some(key))
            .map(|_| setup)
    }

    pub fn current_env_server_name(&self) -> Option<&str> {
        self.env_input_server_idx
            .and_then(|idx| self.mcp_servers.get(idx))
            .map(|s| s.def.name.as_str())
    }

    pub(crate) fn local_mcp_project(&self) -> Option<&str> {
        (self.target_cli == Some(crate::target::TargetCli::Claude)
            && self.mcp_scope == McpScope::Local)
            .then_some(self.mcp_project_path.as_str())
    }

    pub fn toggle_mcp_scope(&mut self) {
        if self.target_cli == Some(crate::target::TargetCli::Codex) {
            self.status_message = Some("Codex MCP uses user scope.".into());
            return;
        }
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
        if self.current_view != View::ProjectPath {
            return;
        }
        if self.project_path_buffer.is_empty() {
            self.status_message = Some("Enter an existing project directory.".into());
            return;
        }
        let path =
            crate::source::config::expand_tilde(std::path::Path::new(&self.project_path_buffer));
        let canonical = match path.canonicalize() {
            Ok(path) => path,
            Err(error) => {
                self.status_message = Some(format!(
                    "Cannot open project directory: {error}. Choose an existing directory."
                ));
                return;
            }
        };
        if !canonical.is_dir() {
            self.status_message =
                Some("The project path is not a directory. Choose an existing directory.".into());
            return;
        }
        self.mcp_project_path = canonical.to_string_lossy().into_owned();
        self.mcp_scope = McpScope::Local;
        self.status_message = Some(format!("MCP scope: local ({})", self.mcp_project_path));
        self.current_view = View::List;
    }

    pub fn project_path_cancel(&mut self) {
        // Revert to user scope if cancelled
        self.mcp_scope = McpScope::User;
        self.status_message = Some("MCP scope: user".to_string());
        self.current_view = View::List;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::test_support::fresh_app;

    #[test]
    fn stale_env_submit_returns_error_instead_of_panicking() {
        let mut app = fresh_app();
        app.current_view = View::EnvInput;
        app.env_input_buffer = "secret".into();
        app.env_input_current = usize::MAX;
        assert!(app.env_input_submit().is_err());
        assert!(app.env_input_values.is_empty());
    }

    #[test]
    fn empty_and_stale_view_submits_do_not_advance() {
        let mut app = fresh_app();
        app.current_view = View::EnvInput;
        app.env_input_vars = vec!["TOKEN".into()];
        app.env_input_submit().unwrap();
        assert_eq!(app.env_input_current, 0);
        app.current_view = View::List;
        app.env_input_buffer = "secret".into();
        app.env_input_submit().unwrap();
        assert!(app.env_input_values.is_empty());
        assert_eq!(app.current_view, View::List);
    }

    #[test]
    fn korean_input_backspace_removes_one_character() {
        let mut app = fresh_app();
        app.env_input_char('한');
        app.env_input_char('글');
        app.env_input_backspace();
        assert_eq!(app.env_input_buffer, "한");
    }

    struct ProjectFixture(std::path::PathBuf);

    impl ProjectFixture {
        fn new() -> Self {
            let stamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = std::env::temp_dir()
                .join(format!("hibi_mcp_project_{}_{stamp}", std::process::id()));
            std::fs::create_dir(&path).unwrap();
            Self(path)
        }
    }

    impl Drop for ProjectFixture {
        fn drop(&mut self) {
            std::fs::remove_dir_all(&self.0).unwrap();
        }
    }

    #[test]
    fn invalid_project_paths_keep_form_previous_directory_and_input() {
        let fixture = ProjectFixture::new();
        let file = fixture.0.join("not-a-directory");
        std::fs::write(&file, "file").unwrap();
        let missing = fixture.0.join("missing-directory");
        let mut app = fresh_app();
        app.mcp_scope = McpScope::Local;
        app.current_view = View::ProjectPath;
        app.mcp_project_path = "previous project".into();
        for value in [
            String::new(),
            missing.to_string_lossy().into_owned(),
            file.to_string_lossy().into_owned(),
            "invalid\0path".into(),
        ] {
            app.project_path_buffer = value.clone();
            app.project_path_submit();
            assert_eq!(app.current_view, View::ProjectPath);
            assert_eq!(app.mcp_scope, McpScope::Local);
            assert_eq!(app.mcp_project_path, "previous project");
            assert_eq!(app.project_path_buffer, value);
            assert!(app.status_message.as_deref().unwrap().contains("directory"));
            assert!(!missing.exists());
        }
    }

    #[test]
    fn korean_project_directory_with_leading_and_trailing_spaces_is_committed() {
        let fixture = ProjectFixture::new();
        let project = fixture.0.join(" 프로젝트 폴더 ");
        std::fs::create_dir(&project).unwrap();
        let mut app = fresh_app();
        app.current_view = View::ProjectPath;
        app.mcp_scope = McpScope::Local;
        app.project_path_buffer = project.to_string_lossy().into_owned();
        let entered = app.project_path_buffer.clone();
        app.project_path_submit();
        assert_eq!(app.current_view, View::List);
        assert_eq!(app.mcp_scope, McpScope::Local);
        assert_eq!(
            app.mcp_project_path,
            project.canonicalize().unwrap().to_string_lossy()
        );
        assert_eq!(app.project_path_buffer, entered);
    }

    #[test]
    fn cancelling_invalid_project_form_reverts_to_user_and_keeps_previous_directory() {
        let mut app = fresh_app();
        app.current_view = View::ProjectPath;
        app.mcp_scope = McpScope::Local;
        app.mcp_project_path = "previous project".into();
        app.project_path_submit();
        assert_eq!(app.current_view, View::ProjectPath);
        app.project_path_cancel();
        assert_eq!(app.current_view, View::List);
        assert_eq!(app.mcp_scope, McpScope::User);
        assert_eq!(app.mcp_project_path, "previous project");
    }

    #[test]
    fn stale_project_submission_does_not_commit_a_new_directory() {
        let fixture = ProjectFixture::new();
        let mut app = fresh_app();
        app.current_view = View::List;
        app.mcp_scope = McpScope::User;
        app.mcp_project_path = "previous project".into();
        app.project_path_buffer = fixture.0.to_string_lossy().into_owned();
        app.project_path_submit();
        assert_eq!(app.mcp_scope, McpScope::User);
        assert_eq!(app.mcp_project_path, "previous project");
        assert_eq!(app.current_view, View::List);
    }
}
