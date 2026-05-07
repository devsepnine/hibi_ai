use chrono::Local;
use serde::Deserialize;
use std::env;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::Path;
use std::process::Command;

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

struct StatusValues {
    user: String,
    cwd: String,
    branch: String,
    git_status: String,
    model: String,
    time: String,
    remaining: Option<u32>,
    total_input_tokens: Option<u64>,
    total_output_tokens: Option<u64>,
    context_window_size: Option<u64>,
    five_hour_pct: Option<f64>,
    seven_day_pct: Option<f64>,
    version: Option<String>,
    todo_count: usize,
}

// ============================================================================
// Constants
// ============================================================================

const CYAN: &str = "\x1b[38;2;23;146;153m";
const BLUE: &str = "\x1b[38;2;30;102;245m";
const GREEN: &str = "\x1b[38;2;64;160;43m";
const YELLOW: &str = "\x1b[38;2;223;142;29m";
const MAGENTA: &str = "\x1b[38;2;136;57;239m";
const GRAY: &str = "\x1b[38;2;76;79;105m";
const RED: &str = "\x1b[38;2;210;15;57m";
const RESET: &str = "\x1b[0m";

const RATE_LIMIT_CRITICAL: f64 = 90.0;
const RATE_LIMIT_WARNING: f64 = 70.0;

const TOKEN_MILLION: u64 = 1_000_000;
const TOKEN_TEN_THOUSAND: u64 = 10_000;
const TOKEN_THOUSAND: u64 = 1_000;

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
// Formatting helpers
// ============================================================================

fn format_tokens(n: u64) -> String {
    if n >= TOKEN_MILLION {
        format!("{:.1}M", n as f64 / TOKEN_MILLION as f64)
    } else if n >= TOKEN_TEN_THOUSAND {
        format!("{}k", n / TOKEN_THOUSAND)
    } else if n >= TOKEN_THOUSAND {
        format!("{:.1}k", n as f64 / TOKEN_THOUSAND as f64)
    } else {
        n.to_string()
    }
}

fn rate_limit_color(pct: f64) -> &'static str {
    if pct >= RATE_LIMIT_CRITICAL {
        RED
    } else if pct >= RATE_LIMIT_WARNING {
        YELLOW
    } else {
        GREEN
    }
}

fn append_segment(line: &mut String, sep: &mut bool, segment: &str) {
    if *sep {
        line.push(' ');
    }
    line.push_str(segment);
    *sep = true;
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
// Output rendering
// ============================================================================

fn format_line1(v: &StatusValues) -> String {
    let mut line = format!("{CYAN}{}{RESET}:{BLUE}{}{RESET}", v.user, v.cwd);
    if !v.branch.is_empty() {
        line.push_str(&format!(
            " {GREEN}{}{YELLOW}{}{RESET}",
            v.branch, v.git_status
        ));
    }
    line.push_str(&format!(" {GRAY}{}{RESET} {YELLOW}{}{RESET}", v.model, v.time));
    line
}

fn format_token_summary(v: &StatusValues) -> Option<String> {
    let input = v.total_input_tokens?;
    let summary = match (v.context_window_size, v.total_output_tokens) {
        (Some(size), _) => format!("{}/{}", format_tokens(input), format_tokens(size)),
        (None, Some(out)) => format!("{}↓{}↑", format_tokens(input), format_tokens(out)),
        (None, None) => format_tokens(input),
    };
    Some(summary)
}

fn format_line2(v: &StatusValues) -> String {
    let mut line = String::new();
    let mut sep = false;

    if let Some(pct) = v.remaining {
        let mut segment = format!("{MAGENTA}ctx:{pct}%{RESET}");
        if let Some(tokens) = format_token_summary(v) {
            segment.push_str(&format!(" {GRAY}({tokens}){RESET}"));
        }
        append_segment(&mut line, &mut sep, &segment);
    }

    if let Some(pct) = v.five_hour_pct {
        let color = rate_limit_color(pct);
        append_segment(&mut line, &mut sep, &format!("{color}5h:{pct:.0}%{RESET}"));
    }
    if let Some(pct) = v.seven_day_pct {
        let color = rate_limit_color(pct);
        append_segment(&mut line, &mut sep, &format!("{color}7d:{pct:.0}%{RESET}"));
    }

    if let Some(version) = v.version.as_deref() {
        append_segment(&mut line, &mut sep, &format!("{GRAY}v{version}{RESET}"));
    }

    if v.todo_count > 0 {
        let segment = format!("{CYAN}todos:{}{RESET}", v.todo_count);
        append_segment(&mut line, &mut sep, &segment);
    }

    line
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

    println!("{}", format_line1(&values));
    let line2 = format_line2(&values);
    if !line2.is_empty() {
        println!("{line2}");
    }
    let _ = io::stdout().flush();
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_tokens_under_thousand_returns_raw() {
        assert_eq!(format_tokens(0), "0");
        assert_eq!(format_tokens(500), "500");
        assert_eq!(format_tokens(999), "999");
    }

    #[test]
    fn format_tokens_thousands_uses_one_decimal() {
        assert_eq!(format_tokens(1_000), "1.0k");
        assert_eq!(format_tokens(1_500), "1.5k");
        assert_eq!(format_tokens(9_500), "9.5k");
    }

    #[test]
    fn format_tokens_ten_thousand_drops_decimal() {
        assert_eq!(format_tokens(10_000), "10k");
        assert_eq!(format_tokens(15_500), "15k");
        assert_eq!(format_tokens(200_000), "200k");
    }

    #[test]
    fn format_tokens_millions_uses_one_decimal() {
        assert_eq!(format_tokens(1_000_000), "1.0M");
        assert_eq!(format_tokens(1_500_000), "1.5M");
        assert_eq!(format_tokens(2_300_000), "2.3M");
    }

    #[test]
    fn rate_limit_color_under_warning_is_green() {
        assert_eq!(rate_limit_color(0.0), GREEN);
        assert_eq!(rate_limit_color(50.0), GREEN);
        assert_eq!(rate_limit_color(69.9), GREEN);
    }

    #[test]
    fn rate_limit_color_warning_band_is_yellow() {
        assert_eq!(rate_limit_color(70.0), YELLOW);
        assert_eq!(rate_limit_color(80.0), YELLOW);
        assert_eq!(rate_limit_color(89.9), YELLOW);
    }

    #[test]
    fn rate_limit_color_critical_band_is_red() {
        assert_eq!(rate_limit_color(90.0), RED);
        assert_eq!(rate_limit_color(95.0), RED);
        assert_eq!(rate_limit_color(100.0), RED);
    }

    #[test]
    fn append_segment_first_call_omits_separator() {
        let mut line = String::new();
        let mut sep = false;
        append_segment(&mut line, &mut sep, "first");
        assert_eq!(line, "first");
        assert!(sep);
    }

    #[test]
    fn append_segment_subsequent_call_inserts_space() {
        let mut line = String::from("first");
        let mut sep = true;
        append_segment(&mut line, &mut sep, "second");
        assert_eq!(line, "first second");
        assert!(sep);
    }

    #[test]
    fn append_segment_chained_three_calls() {
        let mut line = String::new();
        let mut sep = false;
        append_segment(&mut line, &mut sep, "a");
        append_segment(&mut line, &mut sep, "b");
        append_segment(&mut line, &mut sep, "c");
        assert_eq!(line, "a b c");
    }
}
