use chrono::Local;
use regex::Regex;
use serde::Deserialize;
use std::collections::HashSet;
use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};
use std::path::PathBuf;
use walkdir::WalkDir;

#[derive(Deserialize)]
struct HookInput {
    prompt: Option<String>,
}

#[derive(Deserialize)]
struct Frontmatter {
    keywords: Vec<String>,
}

struct AgentInfo {
    filename: String,
    keywords: Vec<String>,
    content: String,
}

fn get_agents_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".claude")
        .join("agents")
}

fn get_log_file() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_default()
        .join("inject-guide.log")
}

fn rotate_log_if_needed(log_path: &PathBuf) {
    const MAX_LOG_SIZE: u64 = 10 * 1024 * 1024; // 10MB
    const MAX_BACKUPS: u8 = 5;

    // Check if log file exists and its size
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
            let _ = fs::rename(log_path, &backup);

            // Log rotation info to new file
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

    // Check and rotate log if needed
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

fn parse_frontmatter(content: &str) -> Option<(Frontmatter, String)> {
    let content = content.trim_start();

    if !content.starts_with("---") {
        return None;
    }

    let after_first = &content[3..];
    let end_pos = after_first.find("\n---")?;

    let yaml_str = &after_first[..end_pos];
    let rest = &after_first[end_pos + 4..];

    let frontmatter: Frontmatter = serde_yaml::from_str(yaml_str).ok()?;

    Some((frontmatter, rest.trim_start().to_string()))
}

fn load_all_agents() -> Vec<AgentInfo> {
    let agents_dir = get_agents_dir();
    let mut agents = Vec::new();

    for entry in WalkDir::new(&agents_dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        if path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }

        // agents/ 기준 상대 경로 사용 (하위폴더 포함)
        let relative_path = match path.strip_prefix(&agents_dir) {
            Ok(p) => p.to_string_lossy().to_string(),
            Err(_) => continue,
        };

        let file_content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        if let Some((frontmatter, content)) = parse_frontmatter(&file_content) {
            agents.push(AgentInfo {
                filename: relative_path,
                keywords: frontmatter.keywords,
                content,
            });
        }
    }

    agents
}

fn find_matching_agents<'a>(prompt: &str, agents: &'a [AgentInfo]) -> Vec<&'a AgentInfo> {
    let prompt_lower = prompt.to_lowercase();
    let mut matched: Vec<&AgentInfo> = Vec::new();
    let mut seen: HashSet<&str> = HashSet::new();

    for agent in agents {
        for keyword in &agent.keywords {
            let pattern = format!(r"(?i){}", regex::escape(keyword));
            if let Ok(re) = Regex::new(&pattern) {
                if re.is_match(&prompt_lower) && !seen.contains(agent.filename.as_str()) {
                    matched.push(agent);
                    seen.insert(&agent.filename);
                    break;
                }
            }
        }
    }

    matched
}

fn main() {
    let mut input = String::new();
    if io::stdin().read_to_string(&mut input).is_err() {
        return;
    }

    let hook_input: HookInput = match serde_json::from_str(&input) {
        Ok(v) => v,
        Err(_) => {
            log("ERROR: JSON parsing failed.");
            return;
        }
    };

    let prompt = match hook_input.prompt {
        Some(p) if !p.is_empty() => p,
        _ => return,
    };

    let agents = load_all_agents();
    let matched_agents = find_matching_agents(&prompt, &agents);

    if matched_agents.is_empty() {
        let truncated: String = prompt.chars().take(50).collect();
        log(&format!("NO MATCH: '{}...'", truncated));
        return;
    }

    let filenames: Vec<&str> = matched_agents.iter().map(|a| a.filename.as_str()).collect();
    let truncated: String = prompt.chars().take(50).collect();
    log(&format!("MATCHED: {:?} <- '{}...'", filenames, truncated));

    let mut output = String::from("\n<injected-agent>\n");
    output.push_str("You MUST follow these agent instructions:");

    for agent in &matched_agents {
        output.push_str(&format!("\n\n## {}\n\n", agent.filename));
        output.push_str(&agent.content);
    }

    output.push_str("\n</injected-agent>\n");

    print!("{}", output);
}
