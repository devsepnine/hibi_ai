use anyhow::Result;

use super::types::{Tab, View};
use super::App;
use crate::fs;

impl App {
    pub fn set_default_style(&mut self) -> Result<()> {
        // Only works for OutputStyles tab
        if self.tab != Tab::OutputStyles {
            self.status_message = Some("Switch to Styles tab to set default".to_string());
            return Ok(());
        }

        if let Some(idx) = self.selected_component_index() {
            if let Some(component) = self.components.get(idx) {
                // Get style name without extension
                let style_name = component.name
                    .strip_suffix(".md")
                    .unwrap_or(&component.name)
                    .to_string();

                fs::installer::set_output_style(&self.dest_dir, &style_name)?;
                self.current_output_style = Some(style_name.clone());
                self.status_message = Some(format!("Set default output style: {}", style_name));
            }
        }
        Ok(())
    }

    pub fn unset_default_style(&mut self) -> Result<()> {
        // Only works for OutputStyles tab
        if self.tab != Tab::OutputStyles {
            self.status_message = Some("Switch to OutputStyles tab to unset default".to_string());
            return Ok(());
        }

        fs::installer::unset_output_style(&self.dest_dir)?;
        self.current_output_style = None;
        self.status_message = Some("Unset default output style".to_string());
        Ok(())
    }

    pub fn set_statusline(&mut self) -> Result<()> {
        // Only works for Statusline tab
        if self.tab != Tab::Statusline {
            self.status_message = Some("Switch to Statusline tab to set default".to_string());
            return Ok(());
        }

        if let Some(idx) = self.selected_component_index() {
            if let Some(component) = self.components.get(idx) {
                fs::installer::set_statusline(&self.dest_dir, &component.name)?;
                self.current_statusline = Some(component.name.clone());
                self.status_message = Some(format!("Set statusline: {}", component.name));
            }
        }
        Ok(())
    }

    pub fn unset_statusline(&mut self) -> Result<()> {
        // Only works for Statusline tab
        if self.tab != Tab::Statusline {
            self.status_message = Some("Switch to Statusline tab to unset default".to_string());
            return Ok(());
        }

        fs::installer::unset_statusline(&self.dest_dir)?;
        self.current_statusline = None;
        self.status_message = Some("Unset statusline".to_string());
        Ok(())
    }

    pub fn show_diff(&mut self) -> Result<()> {
        if self.tab == Tab::McpServers || self.tab == Tab::Plugins {
            return Ok(());
        }

        if let Some(idx) = self.selected_component_index() {
            if let Some(c) = self.components.get(idx) {
                let diff = fs::diff::compare_files(&c.source_path, &c.dest_path)?;
                self.diff_content = Some(diff);
                self.diff_scroll = 0;
                self.current_view = View::Diff;
            }
        }
        Ok(())
    }

    pub fn close_diff(&mut self) {
        self.diff_content = None;
        self.current_view = View::List;
    }

    pub fn scroll_diff_down(&mut self) {
        self.diff_scroll = self.diff_scroll.saturating_add(1);
    }

    pub fn scroll_diff_up(&mut self) {
        self.diff_scroll = self.diff_scroll.saturating_sub(1);
    }
}
