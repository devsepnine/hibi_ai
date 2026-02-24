use memory_persistence_hooks::{get_sessions_dir, format_datetime, format_time, log_hook};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::time::SystemTime;

fn main() {
    // Run silently - log to file only, no stderr output
    // This prevents Windows from blocking on stderr
    let _ = run();
}

fn run() -> std::io::Result<()> {
    let sessions_dir = get_sessions_dir()?;
    let _ = log_hook("PreCompact", &format!("Sessions dir: {}", sessions_dir.display()));

    let compaction_log = sessions_dir.join("compaction-log.txt");

    // Log compaction event with timestamp
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&compaction_log)?;

    let timestamp = format_datetime();
    writeln!(file, "[{}] Context compaction triggered", timestamp)?;
    let _ = log_hook("PreCompact", &format!("Compaction triggered at {}", timestamp));

    // If there's an active session file, note the compaction
    // Find the most recently modified .tmp file
    let mut active_session = None;
    let mut latest_time = SystemTime::UNIX_EPOCH;

    if let Ok(entries) = fs::read_dir(&sessions_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("tmp") {
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        if modified > latest_time {
                            latest_time = modified;
                            active_session = Some(path);
                        }
                    }
                }
            }
        }
    }

    if let Some(session_path) = active_session {
        let _ = log_hook("PreCompact", &format!("Marking active session: {}", session_path.display()));

        let mut file = OpenOptions::new()
            .append(true)
            .open(&session_path)?;

        writeln!(file)?;
        writeln!(file, "---")?;
        writeln!(
            file,
            "**[Compaction occurred at {}]** - Context was summarized",
            format_time()
        )?;

        let _ = log_hook("PreCompact", "Compaction marker added to session file");
    } else {
        let _ = log_hook("PreCompact", "No active session found to mark");
    }

    Ok(())
}
