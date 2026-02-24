use memory_persistence_hooks::{get_sessions_dir, format_date, format_time, log_hook};
use std::fs;
use std::io::Read;

fn main() {
    // Run silently - log to file only, no stderr output
    // This prevents Windows from blocking on stderr
    let _ = run();
}

fn run() -> std::io::Result<()> {
    let sessions_dir = get_sessions_dir()?;
    let _ = log_hook("SessionEnd", &format!("Sessions dir: {}", sessions_dir.display()));

    let today = format_date();
    let session_file = sessions_dir.join(format!("{}-session.tmp", today));
    let _ = log_hook("SessionEnd", &format!("Target session file: {}", session_file.display()));

    if session_file.exists() {
        let _ = log_hook("SessionEnd", "Session file exists - updating timestamp");

        // Update Last Updated timestamp
        let mut content = String::new();
        {
            let mut file = fs::File::open(&session_file)?;
            file.read_to_string(&mut content)?;
        }

        // Replace the Last Updated line
        let current_time = format_time();
        let updated_content = if let Some(pos) = content.find("**Last Updated:**") {
            let line_start = content[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
            let line_end = content[pos..].find('\n').map(|i| pos + i).unwrap_or(content.len());

            format!(
                "{}**Last Updated:** {}{}",
                &content[..line_start],
                current_time,
                &content[line_end..]
            )
        } else {
            let _ = log_hook("SessionEnd", "Warning: Could not find 'Last Updated' line");
            content
        };

        fs::write(&session_file, updated_content)?;
        let _ = log_hook("SessionEnd", &format!("Updated timestamp to {}", current_time));
    } else {
        let _ = log_hook("SessionEnd", "Session file does not exist - creating new");

        // Create new session file with template
        let current_time = format_time();
        let template = format!(
r#"# Session: {}
**Date:** {}
**Started:** {}
**Last Updated:** {}

---

## Current State

[Session context goes here]

### Completed
- [ ]

### In Progress
- [ ]

### Notes for Next Session
-

### Context to Load
```
[relevant files]
```
"#,
            today, today, current_time, current_time
        );

        fs::write(&session_file, template)?;
        let _ = log_hook("SessionEnd", &format!("Created new session file at {}", current_time));
    }

    Ok(())
}
