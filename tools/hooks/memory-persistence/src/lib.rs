use std::path::PathBuf;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};

/// Get the sessions directory path (~/.claude/sessions)
pub fn get_sessions_dir() -> io::Result<PathBuf> {
    let home = dirs::home_dir()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Home directory not found"))?;

    let sessions_dir = home.join(".claude").join("sessions");
    fs::create_dir_all(&sessions_dir)?;

    Ok(sessions_dir)
}

/// Rotate log file if it exceeds size limit
fn rotate_log_if_needed(log_path: &PathBuf) -> io::Result<()> {
    const MAX_LOG_SIZE: u64 = 10 * 1024 * 1024; // 10MB
    const MAX_BACKUPS: u8 = 5;

    if let Ok(metadata) = fs::metadata(log_path) {
        if metadata.len() > MAX_LOG_SIZE {
            // Rotate existing backups: .4 -> .5, .3 -> .4, etc.
            for i in (1..MAX_BACKUPS).rev() {
                let old_backup = log_path.with_extension(format!("log.{}", i));
                let new_backup = log_path.with_extension(format!("log.{}", i + 1));

                if old_backup.exists() {
                    let _ = fs::rename(&old_backup, &new_backup);
                }
            }

            // Move current log to .1
            let backup = log_path.with_extension("log.1");
            fs::rename(log_path, &backup)?;

            // Log rotation info to new file
            let timestamp = format_datetime();
            let rotation_msg = format!(
                "[{}] Log rotated (previous file exceeded {}MB)\n",
                timestamp,
                MAX_LOG_SIZE / 1024 / 1024
            );
            fs::write(log_path, rotation_msg)?;
        }
    }

    Ok(())
}

/// Write log entry to hooks.log
pub fn log_hook(hook_name: &str, message: &str) -> io::Result<()> {
    let sessions_dir = get_sessions_dir()?;
    let log_file = sessions_dir.join("hooks.log");

    // Rotate log if needed before writing
    let _ = rotate_log_if_needed(&log_file);

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file)?;

    let timestamp = format_datetime();
    writeln!(file, "[{}] [{}] {}", timestamp, hook_name, message)?;

    Ok(())
}

/// Log hook start
pub fn log_start(hook_name: &str) {
    let _ = log_hook(hook_name, "Started");
    eprintln_hook(&format!("[{}] Started", hook_name));
}

/// Log hook end
pub fn log_end(hook_name: &str) {
    let _ = log_hook(hook_name, "Completed");
    eprintln_hook(&format!("[{}] Completed", hook_name));
}

/// Log hook error
pub fn log_error(hook_name: &str, error: &str) {
    let _ = log_hook(hook_name, &format!("ERROR: {}", error));
    eprintln!("[{}] ERROR: {}", hook_name, error);
}

/// Get the learned skills directory path (~/.claude/skills/learned)
pub fn get_learned_dir() -> io::Result<PathBuf> {
    let home = dirs::home_dir()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Home directory not found"))?;

    let learned_dir = home.join(".claude").join("skills").join("learned");
    fs::create_dir_all(&learned_dir)?;

    Ok(learned_dir)
}

/// Format current timestamp as HH:MM
pub fn format_time() -> String {
    chrono::Local::now().format("%H:%M").to_string()
}

/// Format current date as YYYY-MM-DD
pub fn format_date() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

/// Format current datetime as YYYY-MM-DD HH:MM:SS
pub fn format_datetime() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

/// Print to stderr with hook prefix
pub fn eprintln_hook(message: &str) {
    eprintln!("{}", message);
}
