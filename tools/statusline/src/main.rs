use chrono::Local;
use serde::Deserialize;
use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;
use std::process::Command;

#[derive(Deserialize)]
struct StatusInput {
    model: Option<Model>,
    workspace: Option<Workspace>,
    context_window: Option<ContextWindow>,
    transcript_path: Option<String>,
}

#[derive(Deserialize)]
struct Model {
    display_name: Option<String>,
}

#[derive(Deserialize)]
struct Workspace {
    current_dir: Option<String>,
}

#[derive(Deserialize)]
struct ContextWindow {
    remaining_percentage: Option<u32>,
}

// ANSI color codes
const CYAN: &str = "\x1b[38;2;23;146;153m";
const BLUE: &str = "\x1b[38;2;30;102;245m";
const GREEN: &str = "\x1b[38;2;64;160;43m";
const YELLOW: &str = "\x1b[38;2;223;142;29m";
const MAGENTA: &str = "\x1b[38;2;136;57;239m";
const GRAY: &str = "\x1b[38;2;76;79;105m";
const RESET: &str = "\x1b[0m";

fn get_username() -> String {
    env::var("USER")
        .or_else(|_| env::var("USERNAME"))
        .unwrap_or_else(|_| "unknown".to_string())
}

fn replace_home_with_tilde(path: &str) -> String {
    if let Ok(home) = env::var("HOME") {
        if path.starts_with(&home) {
            return path.replacen(&home, "~", 1);
        }
    }
    path.to_string()
}

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

    let status = if !branch.is_empty() {
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
    } else {
        String::new()
    };

    (branch, status)
}

fn count_todos(transcript_path: &str) -> usize {
    fs::read_to_string(transcript_path)
        .ok()
        .map(|content| {
            content
                .matches(r#""type":"todo""#)
                .count()
        })
        .unwrap_or(0)
}

fn get_current_time() -> String {
    // Get current time in HH:MM format using chrono for cross-platform compatibility
    Local::now().format("%H:%M").to_string()
}

fn main() {
    let mut input = String::new();
    if io::stdin().read_to_string(&mut input).is_err() {
        return;
    }

    let status: StatusInput = match serde_json::from_str(&input) {
        Ok(v) => v,
        Err(_) => return,
    };

    // Extract values
    let user = get_username();

    let cwd_raw = status
        .workspace
        .as_ref()
        .and_then(|w| w.current_dir.as_ref())
        .map(|s| s.as_str())
        .unwrap_or("~");

    let cwd = replace_home_with_tilde(cwd_raw);

    let model = status
        .model
        .and_then(|m| m.display_name)
        .unwrap_or_else(|| "Unknown".to_string());

    let time = get_current_time();

    let remaining = status
        .context_window
        .and_then(|c| c.remaining_percentage);

    let transcript_path = status.transcript_path.as_deref().unwrap_or("");
    let todo_count = if !transcript_path.is_empty() && Path::new(transcript_path).exists() {
        count_todos(transcript_path)
    } else {
        0
    };

    let (branch, git_status) = get_git_info(cwd_raw);

    // Build output
    let mut output = String::new();

    // user:cwd
    output.push_str(&format!("{}{}{RESET}:{}{}{RESET}", CYAN, user, BLUE, cwd));

    // git branch and status
    if !branch.is_empty() {
        output.push_str(&format!(" {}{}{}{}{RESET}", GREEN, branch, YELLOW, git_status));
    }

    // context remaining
    if let Some(pct) = remaining {
        output.push_str(&format!(" {}ctx:{}%{RESET}", MAGENTA, pct));
    }

    // model and time
    output.push_str(&format!(" {}{}{RESET} {}{}{RESET}", GRAY, model, YELLOW, time));

    // todos
    if todo_count > 0 {
        output.push_str(&format!(" {}todos:{}{RESET}", CYAN, todo_count));
    }

    println!("{}", output);
    let _ = io::stdout().flush();
}
