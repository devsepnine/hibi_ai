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
/// Symlinks and junctions are never followed, so a directory the user linked
/// in from elsewhere is left alone. A rename is cleaned only when the source
/// ships the new name, so the old skill is never removed without a successor.
/// Returns one status line per old directory that was removed or that kept
/// files it could not delete.
pub fn auto_cleanup_renamed_skills(
    source_dir: &Path,
    dest_dir: &Path,
    installed_ids: &[String],
) -> Vec<String> {
    let recorded: HashSet<String> = installed_ids
        .iter()
        .map(|id| id.replace('\\', "/"))
        .collect();
    let mut report = Vec::new();

    for (old, new) in RENAMED_SKILLS {
        let old_dir = dest_dir.join("skills").join(old);
        if !is_real_dir(&old_dir) || !source_dir.join("skills").join(new).is_dir() {
            continue;
        }
        let (files, links) = entries_under(&old_dir);
        let (mut removed, mut failed, mut kept) = (0, 0, links);
        for file in files {
            let id = file
                .strip_prefix(dest_dir)
                .map(|relative| relative.to_string_lossy().replace('\\', "/"));
            match id {
                Ok(id) if recorded.contains(&id) => match std::fs::remove_file(&file) {
                    Ok(()) => removed += 1,
                    Err(_) => failed += 1,
                },
                _ => kept += 1,
            }
        }
        // Without one recorded file, nothing here is provably hibi's, not
        // even an empty directory tree.
        if removed == 0 {
            continue;
        }
        remove_empty_dirs(&old_dir);
        if !old_dir.exists() {
            report.push(format!("skills/{old}"));
        } else {
            report.push(format!(
                "skills/{old}: kept {kept} file(s) not installed by hibi, {failed} could not be removed"
            ));
        }
    }
    report
}

/// A directory that is not a symlink or junction, judged without following
/// the link.
fn is_real_dir(path: &Path) -> bool {
    std::fs::symlink_metadata(path).is_ok_and(|meta| meta.is_dir())
}

/// Regular files under `dir`, and the number of links skipped on the way.
/// A link is never descended into and never offered for deletion, because
/// removing it could reach whatever the user linked in.
fn entries_under(dir: &Path) -> (Vec<PathBuf>, usize) {
    let (mut files, mut links) = (Vec::new(), 0);
    let Ok(entries) = std::fs::read_dir(dir) else {
        return (files, links);
    };
    for entry in entries.filter_map(|e| e.ok()) {
        let Ok(kind) = entry.file_type() else {
            continue;
        };
        if kind.is_symlink() {
            links += 1;
        } else if kind.is_dir() {
            let (nested, nested_links) = entries_under(&entry.path());
            files.extend(nested);
            links += nested_links;
        } else {
            files.push(entry.path());
        }
    }
    (files, links)
}

/// Depth-first, so a directory empties before its parent is tried.
/// `remove_dir` refuses a non-empty directory, which is what keeps user files.
fn remove_empty_dirs(dir: &Path) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            if entry
                .file_type()
                .is_ok_and(|kind| kind.is_dir() && !kind.is_symlink())
            {
                remove_empty_dirs(&entry.path());
            }
        }
    }
    let _ = std::fs::remove_dir(dir);
}

#[cfg(test)]
mod tests {
    use super::super::test_support::{link_dir, make_undeletable, put, unique_dir};
    use super::*;

    /// A source that ships both new names.
    fn source_with_new_names() -> PathBuf {
        let source = unique_dir("source");
        put(&source, "skills/iced-rs/SKILL.md");
        put(&source, "skills/ratatui-rs/SKILL.md");
        source
    }

    #[test]
    fn a_file_that_cannot_be_removed_is_reported() {
        let (source, dest) = (source_with_new_names(), unique_dir("locked"));
        put(&dest, "skills/iced_rs/SKILL.md");
        put(&dest, "skills/iced_rs/references/locked.md");
        let _guard = make_undeletable(&dest.join("skills/iced_rs/references/locked.md"));
        let ids = vec![
            String::from("skills/iced_rs/SKILL.md"),
            String::from("skills/iced_rs/references/locked.md"),
        ];

        let report = auto_cleanup_renamed_skills(&source, &dest, &ids);

        assert_eq!(
            report,
            vec![String::from(
                "skills/iced_rs: kept 0 file(s) not installed by hibi, 1 could not be removed"
            )]
        );
        assert!(dest.join("skills/iced_rs/references/locked.md").exists());
    }

    #[test]
    fn a_fully_recorded_old_directory_is_removed() {
        let (source, dest) = (source_with_new_names(), unique_dir("full"));
        put(&dest, "skills/iced_rs/SKILL.md");
        put(&dest, "skills/iced_rs/references/widgets.md");
        let ids = vec![
            String::from("skills/iced_rs\\SKILL.md"),
            String::from("skills/iced_rs/references/widgets.md"),
        ];

        let report = auto_cleanup_renamed_skills(&source, &dest, &ids);

        assert_eq!(report, vec![String::from("skills/iced_rs")]);
        assert!(!dest.join("skills/iced_rs").exists());
    }

    #[test]
    fn a_file_the_record_does_not_list_is_kept_with_its_directory() {
        let (source, dest) = (source_with_new_names(), unique_dir("user_file"));
        put(&dest, "skills/ratatui_rs/SKILL.md");
        put(&dest, "skills/ratatui_rs/notes/mine.md");
        let ids = vec![String::from("skills/ratatui_rs/SKILL.md")];

        let report = auto_cleanup_renamed_skills(&source, &dest, &ids);

        assert_eq!(
            report,
            vec![String::from(
                "skills/ratatui_rs: kept 1 file(s) not installed by hibi, 0 could not be removed"
            )]
        );
        assert!(!dest.join("skills/ratatui_rs/SKILL.md").exists());
        assert!(dest.join("skills/ratatui_rs/notes/mine.md").exists());
    }

    #[test]
    fn without_a_record_nothing_is_deleted() {
        let (source, dest) = (source_with_new_names(), unique_dir("no_record"));
        put(&dest, "skills/iced_rs/SKILL.md");

        let report = auto_cleanup_renamed_skills(&source, &dest, &[]);

        assert!(report.is_empty());
        assert!(dest.join("skills/iced_rs/SKILL.md").exists());
    }

    #[test]
    fn an_empty_old_directory_without_a_record_is_kept() {
        let (source, dest) = (source_with_new_names(), unique_dir("empty_dir"));
        std::fs::create_dir_all(dest.join("skills/iced_rs/references")).unwrap();

        let report = auto_cleanup_renamed_skills(&source, &dest, &[]);

        assert!(report.is_empty());
        assert!(dest.join("skills/iced_rs/references").is_dir());
    }

    #[test]
    fn a_skill_that_was_not_renamed_is_never_touched() {
        let (source, dest) = (source_with_new_names(), unique_dir("other"));
        put(&dest, "skills/why/SKILL.md");
        let ids = vec![String::from("skills/why/SKILL.md")];

        let report = auto_cleanup_renamed_skills(&source, &dest, &ids);

        assert!(report.is_empty());
        assert!(dest.join("skills/why/SKILL.md").exists());
    }

    #[test]
    fn nothing_is_removed_when_the_source_lacks_the_new_name() {
        let (source, dest) = (unique_dir("old_source"), unique_dir("no_successor"));
        put(&dest, "skills/iced_rs/SKILL.md");
        let ids = vec![String::from("skills/iced_rs/SKILL.md")];

        let report = auto_cleanup_renamed_skills(&source, &dest, &ids);

        assert!(report.is_empty());
        assert!(dest.join("skills/iced_rs/SKILL.md").exists());
    }

    #[test]
    fn a_linked_subdirectory_is_not_followed() {
        let (source, dest, outside) = (
            source_with_new_names(),
            unique_dir("linked_sub"),
            unique_dir("outside_sub"),
        );
        put(&outside, "widgets.md");
        put(&dest, "skills/iced_rs/SKILL.md");
        assert!(
            link_dir(&outside, &dest.join("skills/iced_rs/references")),
            "the test needs a directory link"
        );
        let ids = vec![
            String::from("skills/iced_rs/SKILL.md"),
            String::from("skills/iced_rs/references/widgets.md"),
        ];

        let report = auto_cleanup_renamed_skills(&source, &dest, &ids);

        assert!(
            outside.join("widgets.md").exists(),
            "a file behind a link must survive"
        );
        assert_eq!(
            report,
            vec![String::from(
                "skills/iced_rs: kept 1 file(s) not installed by hibi, 0 could not be removed"
            )],
            "the skipped link is counted as kept"
        );
    }

    #[test]
    fn a_linked_old_directory_is_left_alone() {
        let (source, dest, outside) = (
            source_with_new_names(),
            unique_dir("linked_root"),
            unique_dir("outside_root"),
        );
        put(&outside, "SKILL.md");
        std::fs::create_dir_all(dest.join("skills")).unwrap();
        assert!(
            link_dir(&outside, &dest.join("skills/iced_rs")),
            "the test needs a directory link"
        );
        let ids = vec![String::from("skills/iced_rs/SKILL.md")];

        let report = auto_cleanup_renamed_skills(&source, &dest, &ids);

        assert!(report.is_empty());
        assert!(outside.join("SKILL.md").exists());
    }
}
