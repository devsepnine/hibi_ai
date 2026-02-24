use chrono::Local;
use std::env;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::PathBuf;

const DEFAULT_THRESHOLD: u32 = 50;
const INTERVAL: u32 = 25;

fn get_counter_file() -> PathBuf {
    let pid = std::process::id();
    PathBuf::from(format!("/tmp/claude-tool-count-{}", pid))
}

fn get_log_file() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_default()
        .join("strategic-compact.log")
}

fn rotate_log_if_needed(log_path: &PathBuf) {
    const MAX_LOG_SIZE: u64 = 10 * 1024 * 1024; // 10MB
    const MAX_BACKUPS: u8 = 5;

    if let Ok(metadata) = fs::metadata(log_path) {
        if metadata.len() > MAX_LOG_SIZE {
            // Rotate existing backups
            for i in (1..MAX_BACKUPS).rev() {
                let old_backup = log_path.with_extension(format!("log.{}", i));
                let new_backup = log_path.with_extension(format!("log.{}", i + 1));

                if old_backup.exists() {
                    let _ = fs::rename(&old_backup, &new_backup);
                }
            }

            // Move current log to .1
            let backup = log_path.with_extension("log.1");
            let _ = fs::rename(log_path, &backup);

            // Log rotation info
            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
            let rotation_msg = format!(
                "[{}] Log rotated (previous file exceeded {}MB)\n",
                timestamp,
                MAX_LOG_SIZE / 1024 / 1024
            );
            let _ = fs::write(log_path, rotation_msg);
        }
    }
}

fn log(message: &str) {
    let log_path = get_log_file();
    rotate_log_if_needed(&log_path);

    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
    let log_line = format!("[{}] {}\n", timestamp, message);

    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
    {
        let _ = file.write_all(log_line.as_bytes());
    }
}

fn read_count(path: &PathBuf) -> u32 {
    fs::read_to_string(path)
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0)
}

fn write_count(path: &PathBuf, count: u32) -> io::Result<()> {
    fs::write(path, count.to_string())
}

fn main() {
    let threshold = env::var("COMPACT_THRESHOLD")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_THRESHOLD);

    let counter_file = get_counter_file();
    let count = read_count(&counter_file) + 1;

    log(&format!("Tool call count: {} (threshold: {})", count, threshold));

    if let Err(e) = write_count(&counter_file, count) {
        log(&format!("ERROR: Failed to update counter: {}", e));
        return;
    }

    // Suggest compact at threshold (log only, no terminal output)
    if count == threshold {
        log(&format!(
            "SUGGESTION: {} tool calls reached - consider /compact if transitioning phases",
            threshold
        ));
    }

    // Suggest at regular intervals after threshold (log only)
    if count > threshold && count % INTERVAL == 0 {
        log(&format!(
            "SUGGESTION: {} tool calls - good checkpoint for /compact if context is stale",
            count
        ));
    }
}
