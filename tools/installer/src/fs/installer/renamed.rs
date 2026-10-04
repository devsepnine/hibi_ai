//! Old skill directories left behind when a bundled skill is renamed.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// `(old, new)` directory names of renamed bundled skills.
pub(crate) const RENAMED_SKILLS: &[(&str, &str)] =
    &[("iced_rs", "iced-rs"), ("ratatui_rs", "ratatui-rs")];

/// Remove what hibi installed under each renamed skill's old directory.
///
/// Only files the install record lists are deleted. Any other file is the
/// user's own and stays, together with the directories that still hold it.
/// Returns the old directories that were removed completely, as
/// `skills/<old>`.
pub fn auto_cleanup_renamed_skills(dest_dir: &Path, installed_ids: &[String]) -> Vec<String> {
    let recorded: HashSet<String> = installed_ids
        .iter()
        .map(|id| id.replace('\\', "/"))
        .collect();
    let mut removed = Vec::new();

    for (old, _new) in RENAMED_SKILLS {
        let old_dir = dest_dir.join("skills").join(old);
        if !old_dir.is_dir() {
            continue;
        }
        for file in files_under(&old_dir) {
            let Ok(relative) = file.strip_prefix(dest_dir) else {
                continue;
            };
            let id = relative.to_string_lossy().replace('\\', "/");
            if recorded.contains(&id) {
                let _ = std::fs::remove_file(&file);
            }
        }
        remove_empty_dirs(&old_dir);
        if !old_dir.exists() {
            removed.push(format!("skills/{old}"));
        }
    }
    removed
}

fn files_under(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return files;
    };
    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_dir() {
            files.extend(files_under(&path));
        } else {
            files.push(path);
        }
    }
    files
}

/// Depth-first, so a directory empties before its parent is tried.
/// `remove_dir` refuses a non-empty directory, which is what keeps user files.
fn remove_empty_dirs(dir: &Path) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            if entry.path().is_dir() {
                remove_empty_dirs(&entry.path());
            }
        }
    }
    let _ = std::fs::remove_dir(dir);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_dest(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("hibi_renamed_{label}_{nanos}"));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn put(dest: &Path, rel: &str) {
        let path = dest.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, "x").unwrap();
    }

    #[test]
    fn a_fully_recorded_old_directory_is_removed() {
        let dest = unique_dest("full");
        put(&dest, "skills/iced_rs/SKILL.md");
        put(&dest, "skills/iced_rs/references/widgets.md");
        let ids = vec![
            String::from("skills/iced_rs\\SKILL.md"),
            String::from("skills/iced_rs/references/widgets.md"),
        ];

        let removed = auto_cleanup_renamed_skills(&dest, &ids);

        assert_eq!(removed, vec![String::from("skills/iced_rs")]);
        assert!(!dest.join("skills/iced_rs").exists());
        let _ = std::fs::remove_dir_all(&dest);
    }

    #[test]
    fn a_file_the_record_does_not_list_is_kept_with_its_directory() {
        let dest = unique_dest("user_file");
        put(&dest, "skills/ratatui_rs/SKILL.md");
        put(&dest, "skills/ratatui_rs/notes/mine.md");
        let ids = vec![String::from("skills/ratatui_rs/SKILL.md")];

        let removed = auto_cleanup_renamed_skills(&dest, &ids);

        assert!(removed.is_empty(), "{removed:?}");
        assert!(!dest.join("skills/ratatui_rs/SKILL.md").exists());
        assert!(dest.join("skills/ratatui_rs/notes/mine.md").exists());
        let _ = std::fs::remove_dir_all(&dest);
    }

    #[test]
    fn without_a_record_nothing_is_deleted() {
        let dest = unique_dest("no_record");
        put(&dest, "skills/iced_rs/SKILL.md");

        let removed = auto_cleanup_renamed_skills(&dest, &[]);

        assert!(removed.is_empty());
        assert!(dest.join("skills/iced_rs/SKILL.md").exists());
        let _ = std::fs::remove_dir_all(&dest);
    }

    #[test]
    fn a_skill_that_was_not_renamed_is_never_touched() {
        let dest = unique_dest("other");
        put(&dest, "skills/why/SKILL.md");
        let ids = vec![String::from("skills/why/SKILL.md")];

        let removed = auto_cleanup_renamed_skills(&dest, &ids);

        assert!(removed.is_empty());
        assert!(dest.join("skills/why/SKILL.md").exists());
        let _ = std::fs::remove_dir_all(&dest);
    }
}
