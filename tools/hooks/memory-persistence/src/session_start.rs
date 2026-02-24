use memory_persistence_hooks::{get_sessions_dir, get_learned_dir, log_hook};
use std::fs;
use std::time::{SystemTime, Duration, UNIX_EPOCH};

fn main() {
    // Run silently - log to file only, no stderr output
    // This prevents Windows from blocking on stderr
    let _ = run();
}

fn run() -> std::io::Result<()> {
    let sessions_dir = get_sessions_dir()?;
    let _ = log_hook("SessionStart", &format!("Sessions dir: {}", sessions_dir.display()));

    let learned_dir = get_learned_dir()?;
    let _ = log_hook("SessionStart", &format!("Learned dir: {}", learned_dir.display()));

    // Check for recent session files (last 7 days)
    let seven_days_ago = SystemTime::now()
        .checked_sub(Duration::from_secs(7 * 24 * 60 * 60))
        .unwrap_or(UNIX_EPOCH);

    let mut recent_sessions = Vec::new();

    if let Ok(entries) = fs::read_dir(&sessions_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("tmp") {
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        if modified > seven_days_ago {
                            recent_sessions.push((modified, path));
                        }
                    }
                }
            }
        }
    }

    if !recent_sessions.is_empty() {
        // Sort by modification time (newest first)
        recent_sessions.sort_by(|a, b| b.0.cmp(&a.0));

        let msg = format!("Found {} recent session(s)", recent_sessions.len());
        let _ = log_hook("SessionStart", &msg);

        if let Some((_, latest_path)) = recent_sessions.first() {
            let msg = format!("Latest session: {}", latest_path.display());
            let _ = log_hook("SessionStart", &msg);
        }
    } else {
        let _ = log_hook("SessionStart", "No recent sessions found");
    }

    // Check for learned skills
    let learned_count = if let Ok(entries) = fs::read_dir(&learned_dir) {
        entries
            .flatten()
            .filter(|e| {
                e.path()
                    .extension()
                    .and_then(|s| s.to_str()) == Some("md")
            })
            .count()
    } else {
        0
    };

    if learned_count > 0 {
        let msg = format!(
            "{} learned skill(s) available in {}",
            learned_count,
            learned_dir.display()
        );
        let _ = log_hook("SessionStart", &msg);
    } else {
        let _ = log_hook("SessionStart", "No learned skills found");
    }

    Ok(())
}
