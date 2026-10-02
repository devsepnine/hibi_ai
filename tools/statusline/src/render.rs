//! Output rendering: segment assembly, ANSI-aware width measurement, and
//! width-based line wrapping.

use unicode_width::UnicodeWidthChar;

use crate::StatusValues;

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

/// Claude Code documents COLUMNS as the full terminal width, but notifications
/// can share the status row outside fullscreen mode, so wrap slightly early.
const WRAP_MARGIN: usize = 2;

/// Width available to the statusline, or None to disable wrapping.
/// Claude Code captures the script's stdout, so tty-based size queries cannot
/// work; it sets COLUMNS/LINES to the terminal dimensions for the statusline
/// process instead (documented statusline contract).
pub(crate) fn terminal_width() -> Option<usize> {
    let width: usize = std::env::var("COLUMNS").ok()?.parse().ok()?;
    // A tiny COLUMNS (<= margin) disables wrapping rather than putting every
    // segment on its own line.
    width.checked_sub(WRAP_MARGIN).filter(|w| *w > 0)
}

/// Display width of a string, ignoring ANSI escape sequences.
pub(crate) fn visible_width(s: &str) -> usize {
    let mut width = 0;
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // Only CSI sequences (ESC [ params final-byte) occur in this output,
            // and SGR params stay below 0x40, so the first byte in 0x40..=0x7E
            // after '[' terminates the sequence. '[' itself (0x5B) is in that
            // range and must be consumed first.
            for c in chars.by_ref() {
                if ('\x40'..='\x7e').contains(&c) && c != '[' {
                    break;
                }
            }
        } else {
            width += UnicodeWidthChar::width(c).unwrap_or(0);
        }
    }
    width
}

/// Join segments with single spaces, starting a new line when the next segment
/// would exceed `width`. None disables wrapping. A segment wider than the
/// width stays intact on its own line — splitting mid-path or mid-branch-name
/// would make it unreadable.
pub(crate) fn wrap_segments(segments: &[String], width: Option<usize>) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    let mut current_width = 0usize;

    for segment in segments {
        let segment_width = visible_width(segment);
        if !current.is_empty() {
            let would_be = current_width + 1 + segment_width;
            if width.is_some_and(|w| would_be > w) {
                lines.push(std::mem::take(&mut current));
                current_width = 0;
            }
        }
        if !current.is_empty() {
            current.push(' ');
            current_width += 1;
        }
        current.push_str(segment);
        current_width += segment_width;
    }

    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

pub(crate) fn line1_segments(v: &StatusValues) -> Vec<String> {
    let mut segments = vec![format!("{CYAN}{}{RESET}:{BLUE}{}{RESET}", v.user, v.cwd)];
    if !v.branch.is_empty() {
        segments.push(format!("{GREEN}{}{YELLOW}{}{RESET}", v.branch, v.git_status));
    }
    segments.push(format!("{GRAY}{}{RESET}", v.model));
    segments.push(format!("{YELLOW}{}{RESET}", v.time));
    segments
}

pub(crate) fn line2_segments(v: &StatusValues) -> Vec<String> {
    let mut segments = Vec::new();

    if let Some(pct) = v.remaining {
        let mut segment = format!("{MAGENTA}ctx:{pct}%{RESET}");
        if let Some(tokens) = format_token_summary(v) {
            segment.push_str(&format!(" {GRAY}({tokens}){RESET}"));
        }
        segments.push(segment);
    }

    if let Some(pct) = v.five_hour_pct {
        segments.push(format!("{}5h:{pct:.0}%{RESET}", rate_limit_color(pct)));
    }
    if let Some(pct) = v.seven_day_pct {
        segments.push(format!("{}7d:{pct:.0}%{RESET}", rate_limit_color(pct)));
    }

    if let Some(version) = v.version.as_deref() {
        segments.push(format!("{GRAY}v{version}{RESET}"));
    }

    if v.todo_count > 0 {
        segments.push(format!("{CYAN}todos:{}{RESET}", v.todo_count));
    }

    segments
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

#[cfg(test)]
mod tests {
    use super::*;

    fn values_fixture() -> StatusValues {
        StatusValues {
            user: "user".to_string(),
            cwd: "~/project".to_string(),
            branch: String::new(),
            git_status: String::new(),
            model: "Model".to_string(),
            time: "12:00".to_string(),
            remaining: None,
            total_input_tokens: None,
            total_output_tokens: None,
            context_window_size: None,
            five_hour_pct: None,
            seven_day_pct: None,
            version: None,
            todo_count: 0,
        }
    }

    #[test]
    fn visible_width_strips_ansi_codes() {
        assert_eq!(visible_width("\x1b[38;2;23;146;153muser\x1b[0m"), 4);
    }

    #[test]
    fn visible_width_counts_cjk_as_two_cells() {
        assert_eq!(visible_width("사용자"), 6);
    }

    #[test]
    fn visible_width_plain_ascii() {
        assert_eq!(visible_width("main*"), 5);
    }

    #[test]
    fn wrap_without_width_joins_all_segments() {
        let segments = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        assert_eq!(wrap_segments(&segments, None), vec!["a b c"]);
    }

    #[test]
    fn wrap_breaks_at_width_boundary() {
        let segments = vec!["aaa".to_string(), "bbb".to_string(), "ccc".to_string()];
        // "aaa bbb" is 7 cells and fits in 8; adding " ccc" would reach 11.
        assert_eq!(wrap_segments(&segments, Some(8)), vec!["aaa bbb", "ccc"]);
    }

    #[test]
    fn wrap_exact_fit_stays_on_line() {
        let segments = vec!["aaa".to_string(), "bbb".to_string()];
        assert_eq!(wrap_segments(&segments, Some(7)), vec!["aaa bbb"]);
    }

    #[test]
    fn wrap_keeps_overlong_segment_intact() {
        let segments = vec![
            "a".to_string(),
            "very-long-segment".to_string(),
            "b".to_string(),
        ];
        assert_eq!(
            wrap_segments(&segments, Some(5)),
            vec!["a", "very-long-segment", "b"]
        );
    }

    #[test]
    fn wrap_measures_visible_width_not_raw_len() {
        let red = format!("{RED}aaa{RESET}");
        let segments = vec![red.clone(), red.clone()];
        // Visible width of "aaa aaa" is 7; the raw string is far longer.
        assert_eq!(wrap_segments(&segments, Some(7)), vec![format!("{red} {red}")]);
    }

    #[test]
    fn wrap_empty_input_yields_no_lines() {
        assert!(wrap_segments(&[], Some(10)).is_empty());
    }

    #[test]
    fn line1_segments_omit_branch_when_empty() {
        assert_eq!(line1_segments(&values_fixture()).len(), 3);
    }

    #[test]
    fn line1_segments_include_branch_when_present() {
        let mut v = values_fixture();
        v.branch = "main".to_string();
        assert_eq!(line1_segments(&v).len(), 4);
    }

    #[test]
    fn line2_segments_empty_without_optional_values() {
        assert!(line2_segments(&values_fixture()).is_empty());
    }

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
}
