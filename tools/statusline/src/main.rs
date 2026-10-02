use chrono::Local;
use serde::Deserialize;
use std::env;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::Path;
use std::process::Command;

mod render;

// ============================================================================
// Input schema (subset of Claude Code statusline JSON)
// ============================================================================

#[derive(Deserialize)]
struct StatusInput {
    model: Option<Model>,
    cwd: Option<String>,
    context_window: Option<ContextWindow>,
    transcript_path: Option<String>,
    version: Option<String>,
    rate_limits: Option<RateLimits>,
}

#[derive(Deserialize)]
struct Model {
    display_name: Option<String>,
}

#[derive(Deserialize)]
struct ContextWindow {
    used_percentage: Option<u32>,
    remaining_percentage: Option<u32>,
    total_input_tokens: Option<u64>,
    total_output_tokens: Option<u64>,
    context_window_size: Option<u64>,
}

#[derive(Deserialize)]
struct RateLimits {
    five_hour: Option<RateLimitWindow>,
    seven_day: Option<RateLimitWindow>,
}

#[derive(Deserialize)]
struct RateLimitWindow {
    used_percentage: Option<f64>,
}

// ============================================================================
// Display values (extracted/computed for rendering)
// ============================================================================

pub(crate) struct StatusValues {
    pub(crate) user: String,
    pub(crate) cwd: String,
    pub(crate) branch: String,
    pub(crate) git_status: String,
    pub(crate) model: String,
    pub(crate) time: String,
    pub(crate) remaining: Option<u32>,
    pub(crate) total_input_tokens: Option<u64>,
    pub(crate) total_output_tokens: Option<u64>,
    pub(crate) context_window_size: Option<u64>,
    pub(crate) five_hour_pct: Option<f64>,
    pub(crate) seven_day_pct: Option<f64>,
    pub(crate) version: Option<String>,
    pub(crate) todo_count: usize,
}

// ============================================================================
// Environment helpers
// ============================================================================

fn get_username() -> String {
    env::var("USER")
        .or_else(|_| env::var("USERNAME"))
        .unwrap_or_else(|_| "unknown".to_string())
}

fn get_home_dir() -> Option<String> {
    env::var("HOME")
        .ok()
        .or_else(|| env::var("USERPROFILE").ok())
}

fn replace_home_with_tilde(path: &str) -> String {
    if let Some(home) = get_home_dir() {
        if path.starts_with(&home) {
            return path.replacen(&home, "~", 1);
        }
    }
    path.to_string()
}

fn get_current_time() -> String {
    Local::now().format("%H:%M").to_string()
}

// ============================================================================
// Git helpers
// ============================================================================

fn get_git_info(cwd: &str) -> (String, String) {
    let branch = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(cwd)
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout)
                    .ok()
                    .map(|s| s.trim().to_string())
            } else {
                None
            }
        })
        .unwrap_or_default();

    let status = if branch.is_empty() {
        String::new()
    } else {
        Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(cwd)
            .output()
            .ok()
            .and_then(|output| {
                if output.status.success() && !output.stdout.is_empty() {
                    Some("*".to_string())
                } else {
                    Some(String::new())
                }
            })
            .unwrap_or_default()
    };

    (branch, status)
}

// ============================================================================
// Transcript helpers
// ============================================================================

fn count_todos(transcript_path: &str) -> usize {
    fs::read_to_string(transcript_path)
        .ok()
        .map(|content| content.matches(r#""type":"todo""#).count())
        .unwrap_or(0)
}

fn read_todo_count(transcript_path: &str) -> usize {
    if transcript_path.is_empty() || !Path::new(transcript_path).exists() {
        return 0;
    }
    count_todos(transcript_path)
}

// ============================================================================
// Value extraction
// ============================================================================

fn extract_values(status: StatusInput) -> StatusValues {
    let cwd_raw = status.cwd.unwrap_or_else(|| "~".to_string());
    let cwd = replace_home_with_tilde(&cwd_raw);
    let (branch, git_status) = get_git_info(&cwd_raw);

    let ctx = status.context_window;
    let remaining = ctx.as_ref().and_then(|c| {
        c.remaining_percentage
            .or_else(|| c.used_percentage.map(|u| 100u32.saturating_sub(u)))
    });
    let total_input_tokens = ctx.as_ref().and_then(|c| c.total_input_tokens);
    let total_output_tokens = ctx.as_ref().and_then(|c| c.total_output_tokens);
    let context_window_size = ctx.as_ref().and_then(|c| c.context_window_size);

    let rate_limits = status.rate_limits;
    let five_hour_pct = rate_limits
        .as_ref()
        .and_then(|r| r.five_hour.as_ref().and_then(|w| w.used_percentage));
    let seven_day_pct = rate_limits
        .as_ref()
        .and_then(|r| r.seven_day.as_ref().and_then(|w| w.used_percentage));

    let todo_count = read_todo_count(status.transcript_path.as_deref().unwrap_or(""));

    let model = status
        .model
        .and_then(|m| m.display_name)
        .unwrap_or_else(|| "Unknown".to_string());

    StatusValues {
        user: get_username(),
        cwd,
        branch,
        git_status,
        model,
        time: get_current_time(),
        remaining,
        total_input_tokens,
        total_output_tokens,
        context_window_size,
        five_hour_pct,
        seven_day_pct,
        version: status.version,
        todo_count,
    }
}

// ============================================================================
// Entry point
// ============================================================================

fn main() {
    let mut input = String::new();
    if io::stdin().lock().read_line(&mut input).is_err() || input.is_empty() {
        return;
    }

    let status: StatusInput = match serde_json::from_str(&input) {
        Ok(v) => v,
        Err(_) => return,
    };

    let values = extract_values(status);
    let width = render::terminal_width();

    let mut lines = render::wrap_segments(&render::line1_segments(&values), width);
    let line2 = render::line2_segments(&values);
    if !line2.is_empty() {
        lines.extend(render::wrap_segments(&line2, width));
    }

    println!("{}", lines.join("\n"));
    let _ = io::stdout().flush();
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replace_home_with_tilde_rewrites_prefix() {
        if let Some(home) = get_home_dir() {
            assert_eq!(replace_home_with_tilde(&format!("{home}/work")), "~/work");
        }
    }

    #[test]
    fn replace_home_with_tilde_leaves_other_paths() {
        assert_eq!(replace_home_with_tilde("/etc/hosts"), "/etc/hosts");
    }
}
