//! Files an earlier install wrote that the source no longer ships.

use std::path::{Component as PathPart, Path, PathBuf};

use crate::component::ComponentType;
use crate::fs::manifest::{InstallRecord, content_hash};

/// What the cleanup did: status lines for the user, and the recorded IDs it
/// settled. The caller drops the settled IDs from the install record, so the
/// same notice does not come back on every launch. A failed removal is not
/// settled and is tried again next time.
#[derive(Debug, Default, PartialEq)]
pub struct UnshippedCleanup {
    pub report: Vec<String>,
    pub settled: Vec<String>,
}

enum Verdict {
    Removed,
    AlreadyGone,
    Edited,
    Unverified,
    Failed,
}

/// Remove the files an earlier install wrote that the source no longer ships.
///
/// A file goes only when the record holds its hash and the file still matches
/// it. A file the user edited, one the record has no hash for, and any file
/// the record does not list all stay. Nothing is judged under a directory the
/// source lacks, because a missing source directory would make every recorded
/// file look dropped. Links below the type directory are never followed.
///
/// Only the bundled source is consulted. A file another configured source
/// now ships at the same path is removed when it is still exactly what hibi
/// recorded, and the next scan offers it again as new.
pub fn auto_cleanup_unshipped_files(
    source_dir: &Path,
    dest_dir: &Path,
    record: &InstallRecord,
) -> UnshippedCleanup {
    let mut cleanup = UnshippedCleanup::default();
    let (mut removed, mut edited, mut unverified, mut failed) = (0, 0, 0, 0);

    for id in &record.ids {
        let Some((type_dir, relative)) = mirrored_path(id) else {
            continue;
        };
        if !source_dir.join(&type_dir).is_dir() || source_dir.join(&relative).exists() {
            continue;
        }
        let verdict = judge(
            &dest_dir.join(&type_dir),
            &dest_dir.join(&relative),
            record.hashes.get(id),
        );
        match verdict {
            Verdict::Removed => removed += 1,
            Verdict::Edited => edited += 1,
            Verdict::Unverified => unverified += 1,
            Verdict::AlreadyGone => {}
            Verdict::Failed => {
                failed += 1;
                continue;
            }
        }
        cleanup.settled.push(id.clone());
    }

    let lines = [
        (removed, format!("{removed} file(s) no longer shipped")),
        (
            failed,
            format!("{failed} file(s) no longer shipped could not be removed"),
        ),
        (
            edited,
            format!("kept {edited} edited file(s) no longer shipped"),
        ),
        (
            unverified,
            format!("kept {unverified} unverified file(s) no longer shipped"),
        ),
    ];
    cleanup.report = lines
        .into_iter()
        .filter(|(count, _)| *count > 0)
        .map(|(_, line)| line)
        .collect();
    cleanup
}

/// Split a recorded ID such as `skills/a\b.md` into its type directory and
/// its relative path. Only a tree-mirrored type with plain path parts
/// qualifies; a `..` or a drive prefix could point outside the destination,
/// and the record is a file anyone can edit.
fn mirrored_path(id: &str) -> Option<(PathBuf, PathBuf)> {
    let normalized = id.replace('\\', "/");
    let mut parts = normalized.split('/');
    let type_dir = parts.next()?;
    if !ComponentType::TREE_MIRRORED
        .iter()
        .any(|t| t.display_name() == type_dir)
    {
        return None;
    }
    let relative: PathBuf = std::iter::once(type_dir)
        .chain(parts.filter(|part| !part.is_empty()))
        .collect();
    if relative
        .components()
        .any(|part| !matches!(part, PathPart::Normal(_)))
    {
        return None;
    }
    Some((PathBuf::from(type_dir), relative))
}

fn judge(type_root: &Path, file: &Path, recorded: Option<&String>) -> Verdict {
    if let Some(verdict) = blocked_by_ancestor(type_root, file) {
        return verdict;
    }
    let meta = match std::fs::symlink_metadata(file) {
        Ok(meta) => meta,
        Err(e) => return gone_or_failed(&e),
    };
    if !meta.is_file() {
        return Verdict::Unverified;
    }
    let Some(recorded) = recorded else {
        return Verdict::Unverified;
    };
    let Ok(content) = std::fs::read(file) else {
        return Verdict::Failed;
    };
    if content_hash(&content) != *recorded {
        return Verdict::Edited;
    }
    if std::fs::remove_file(file).is_err() {
        return Verdict::Failed;
    }
    remove_empty_parents(type_root, file);
    Verdict::Removed
}

/// Only a missing path proves the file is gone; any other error, such as a
/// denied permission, leaves it in place, so it is retried on the next launch.
fn gone_or_failed(error: &std::io::Error) -> Verdict {
    if error.kind() == std::io::ErrorKind::NotFound {
        Verdict::AlreadyGone
    } else {
        Verdict::Failed
    }
}

/// The verdict a directory between `type_root` and `file` forces, if any. A
/// link the user made, or a file where a directory belongs, is never
/// followed. `type_root` itself is not judged, since hibi installs through it
/// whatever it is.
fn blocked_by_ancestor(type_root: &Path, file: &Path) -> Option<Verdict> {
    let outermost_first: Vec<&Path> = dirs_between(type_root, file).collect();
    for dir in outermost_first.into_iter().rev() {
        match std::fs::symlink_metadata(dir) {
            Ok(meta) if meta.file_type().is_symlink() || !meta.is_dir() => {
                return Some(Verdict::Unverified);
            }
            Ok(_) => {}
            Err(e) => return Some(gone_or_failed(&e)),
        }
    }
    None
}

/// Directories strictly between `type_root` and `file`, innermost first.
fn dirs_between<'a>(type_root: &'a Path, file: &'a Path) -> impl Iterator<Item = &'a Path> {
    file.ancestors()
        .skip(1)
        .take_while(move |dir| *dir != type_root && dir.starts_with(type_root))
}

/// `remove_dir` refuses a directory that still holds anything, which is what
/// keeps a user's file and the directories above it.
fn remove_empty_parents(type_root: &Path, file: &Path) {
    for dir in dirs_between(type_root, file) {
        if std::fs::remove_dir(dir).is_err() {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_support::{link_dir, make_undeletable, put, put_text, unique_dir};
    use super::*;
    use crate::fs::manifest::content_hash;
    use std::collections::BTreeMap;

    /// A source that still ships `skills/kept/SKILL.md`, so its `skills`
    /// directory proves what the source contains.
    fn source() -> std::path::PathBuf {
        let source = unique_dir("unshipped_source");
        put(&source, "skills/kept/SKILL.md");
        source
    }

    fn record(entries: &[(&str, Option<&str>)]) -> InstallRecord {
        InstallRecord {
            ids: entries.iter().map(|(id, _)| id.to_string()).collect(),
            hashes: entries
                .iter()
                .filter_map(|(id, text)| text.map(|t| (id.to_string(), content_hash(t.as_bytes()))))
                .collect::<BTreeMap<_, _>>(),
        }
    }

    #[test]
    fn an_unchanged_file_the_source_dropped_is_removed_with_its_empty_dirs() {
        let (source, dest) = (source(), unique_dir("dropped"));
        put_text(&dest, "skills/kept/SKILL.md", "x");
        put_text(&dest, "skills/kept/resources/old.sh", "old");
        let record = record(&[
            ("skills/kept\\SKILL.md", Some("x")),
            ("skills/kept\\resources\\old.sh", Some("old")),
        ]);

        let cleanup = auto_cleanup_unshipped_files(&source, &dest, &record);

        assert_eq!(cleanup.report, vec!["1 file(s) no longer shipped"]);
        assert_eq!(cleanup.settled, vec!["skills/kept\\resources\\old.sh"]);
        assert!(!dest.join("skills/kept/resources").exists());
        assert!(dest.join("skills/kept/SKILL.md").exists());
    }

    #[test]
    fn a_dropped_skill_goes_but_the_skills_directory_stays() {
        let (source, dest) = (source(), unique_dir("dropped_skill"));
        put_text(&dest, "skills/gone/SKILL.md", "gone");
        put_text(&dest, "skills/gone/references/a.md", "a");
        let record = record(&[
            ("skills/gone/SKILL.md", Some("gone")),
            ("skills/gone/references/a.md", Some("a")),
        ]);

        let cleanup = auto_cleanup_unshipped_files(&source, &dest, &record);

        assert_eq!(cleanup.report, vec!["2 file(s) no longer shipped"]);
        assert!(!dest.join("skills/gone").exists());
        assert!(dest.join("skills").is_dir());
    }

    #[test]
    fn a_file_the_user_edited_is_kept() {
        let (source, dest) = (source(), unique_dir("edited"));
        put_text(&dest, "commands/old.md", "edited by the user");
        std::fs::create_dir_all(source.join("commands")).unwrap();
        let record = record(&[("commands/old.md", Some("as hibi wrote it"))]);

        let cleanup = auto_cleanup_unshipped_files(&source, &dest, &record);

        assert_eq!(
            cleanup.report,
            vec!["kept 1 edited file(s) no longer shipped"]
        );
        assert_eq!(cleanup.settled, vec!["commands/old.md"]);
        assert!(dest.join("commands/old.md").exists());
    }

    #[test]
    fn a_file_without_a_recorded_hash_is_kept() {
        let (source, dest) = (source(), unique_dir("legacy"));
        put_text(&dest, "skills/kept/README.md", "x");
        let record = record(&[("skills/kept/README.md", None)]);

        let cleanup = auto_cleanup_unshipped_files(&source, &dest, &record);

        assert_eq!(
            cleanup.report,
            vec!["kept 1 unverified file(s) no longer shipped"]
        );
        assert_eq!(cleanup.settled, vec!["skills/kept/README.md"]);
        assert!(dest.join("skills/kept/README.md").exists());
    }

    #[test]
    fn a_file_the_record_does_not_list_is_never_touched() {
        let (source, dest) = (source(), unique_dir("users_own"));
        put_text(&dest, "skills/kept/mine.md", "x");

        let cleanup = auto_cleanup_unshipped_files(&source, &dest, &record(&[]));

        assert_eq!(cleanup, UnshippedCleanup::default());
        assert!(dest.join("skills/kept/mine.md").exists());
    }

    #[test]
    fn a_file_the_source_still_ships_is_left_to_the_installer() {
        let (source, dest) = (source(), unique_dir("shipped"));
        put_text(&dest, "skills/kept/SKILL.md", "x");
        let record = record(&[("skills/kept/SKILL.md", Some("x"))]);

        let cleanup = auto_cleanup_unshipped_files(&source, &dest, &record);

        assert_eq!(cleanup, UnshippedCleanup::default());
        assert!(dest.join("skills/kept/SKILL.md").exists());
    }

    #[test]
    fn nothing_is_judged_when_the_source_lacks_the_directory() {
        let (source, dest) = (unique_dir("empty_source"), unique_dir("no_proof"));
        put_text(&dest, "skills/kept/SKILL.md", "x");
        let record = record(&[("skills/kept/SKILL.md", Some("x"))]);

        let cleanup = auto_cleanup_unshipped_files(&source, &dest, &record);

        assert_eq!(cleanup, UnshippedCleanup::default());
        assert!(dest.join("skills/kept/SKILL.md").exists());
    }

    #[test]
    fn ids_outside_the_mirrored_directories_are_ignored() {
        let (source, dest) = (source(), unique_dir("foreign_ids"));
        put_text(&dest, "settings.json", "x");
        put_text(&dest, "escape.md", "x");
        let record = record(&[
            ("config/settings.json", Some("x")),
            ("statusline/statusline", Some("x")),
            ("skills/../escape.md", Some("x")),
            ("skills", Some("x")),
            ("/skills/kept/abs.md", Some("x")),
        ]);

        let cleanup = auto_cleanup_unshipped_files(&source, &dest, &record);

        assert_eq!(cleanup, UnshippedCleanup::default());
        assert!(dest.join("settings.json").exists());
        assert!(dest.join("escape.md").exists());
    }

    #[test]
    fn a_file_behind_a_linked_directory_is_not_followed() {
        let (source, dest, outside) = (source(), unique_dir("linked"), unique_dir("outside"));
        put_text(&outside, "old.sh", "old");
        std::fs::create_dir_all(dest.join("skills/kept")).unwrap();
        assert!(
            link_dir(&outside, &dest.join("skills/kept/resources")),
            "the test needs a directory link"
        );
        let record = record(&[("skills/kept/resources/old.sh", Some("old"))]);

        let cleanup = auto_cleanup_unshipped_files(&source, &dest, &record);

        assert!(
            outside.join("old.sh").exists(),
            "a file behind a link must survive"
        );
        assert_eq!(
            cleanup.report,
            vec!["kept 1 unverified file(s) no longer shipped"]
        );
    }

    #[test]
    fn a_file_that_cannot_be_removed_stays_recorded() {
        let (source, dest) = (source(), unique_dir("locked"));
        put_text(&dest, "skills/gone/SKILL.md", "gone");
        let _guard = make_undeletable(&dest.join("skills/gone/SKILL.md"));
        let record = record(&[("skills/gone/SKILL.md", Some("gone"))]);

        let cleanup = auto_cleanup_unshipped_files(&source, &dest, &record);

        assert_eq!(
            cleanup.report,
            vec!["1 file(s) no longer shipped could not be removed"]
        );
        assert!(
            cleanup.settled.is_empty(),
            "a failed removal is retried later"
        );
        assert!(dest.join("skills/gone/SKILL.md").exists());
    }

    #[cfg(unix)]
    #[test]
    fn an_unreadable_directory_is_retried_rather_than_read_as_gone() {
        use super::super::test_support::make_unreadable;
        let (source, dest) = (source(), unique_dir("unreadable"));
        put_text(&dest, "skills/gone/references/a.md", "a");
        let _guard = make_unreadable(&dest.join("skills/gone"));
        let record = record(&[("skills/gone/references/a.md", Some("a"))]);

        let cleanup = auto_cleanup_unshipped_files(&source, &dest, &record);

        assert_eq!(
            cleanup.report,
            vec!["1 file(s) no longer shipped could not be removed"]
        );
        assert!(
            cleanup.settled.is_empty(),
            "the id stays recorded for a retry"
        );
    }

    #[test]
    fn a_recorded_file_that_is_already_gone_is_settled_quietly() {
        let (source, dest) = (source(), unique_dir("already_gone"));
        let record = record(&[("skills/gone/SKILL.md", Some("gone"))]);

        let cleanup = auto_cleanup_unshipped_files(&source, &dest, &record);

        assert!(cleanup.report.is_empty());
        assert_eq!(cleanup.settled, vec!["skills/gone/SKILL.md"]);
    }

    #[test]
    fn a_directory_at_a_recorded_path_is_left_alone() {
        let (source, dest) = (source(), unique_dir("dir_at_id"));
        put_text(&dest, "skills/gone/SKILL.md/inside.md", "x");
        let record = record(&[("skills/gone/SKILL.md", Some("x"))]);

        let cleanup = auto_cleanup_unshipped_files(&source, &dest, &record);

        assert_eq!(
            cleanup.report,
            vec!["kept 1 unverified file(s) no longer shipped"]
        );
        assert!(dest.join("skills/gone/SKILL.md/inside.md").exists());
    }

    #[test]
    fn a_file_where_a_directory_belongs_is_not_followed() {
        let (source, dest) = (source(), unique_dir("file_as_dir"));
        put_text(&dest, "skills/gone", "x");
        let record = record(&[("skills/gone/SKILL.md", Some("x"))]);

        let cleanup = auto_cleanup_unshipped_files(&source, &dest, &record);

        assert_eq!(
            cleanup.report,
            vec!["kept 1 unverified file(s) no longer shipped"]
        );
        assert!(dest.join("skills/gone").is_file());
    }

    #[test]
    fn a_missing_file_under_existing_directories_is_settled_quietly() {
        let (source, dest) = (source(), unique_dir("missing_leaf"));
        std::fs::create_dir_all(dest.join("skills/gone/references")).unwrap();
        let record = record(&[("skills/gone/references/a.md", Some("a"))]);

        let cleanup = auto_cleanup_unshipped_files(&source, &dest, &record);

        assert!(cleanup.report.is_empty());
        assert_eq!(cleanup.settled, vec!["skills/gone/references/a.md"]);
        assert!(
            dest.join("skills/gone/references").is_dir(),
            "nothing was removed, so no directory is either"
        );
    }

    #[test]
    fn empty_parts_in_an_id_name_the_same_file() {
        let (source, dest) = (source(), unique_dir("empty_parts"));
        put_text(&dest, "skills/gone/SKILL.md", "gone");
        let record = record(&[("skills//gone\\\\SKILL.md", Some("gone"))]);

        let cleanup = auto_cleanup_unshipped_files(&source, &dest, &record);

        assert_eq!(cleanup.report, vec!["1 file(s) no longer shipped"]);
        assert!(!dest.join("skills/gone").exists());
    }

    #[test]
    fn line_endings_do_not_make_a_file_look_edited() {
        let (source, dest) = (source(), unique_dir("crlf"));
        put_text(&dest, "skills/gone/SKILL.md", "a\r\nb\r\n");
        let record = record(&[("skills/gone/SKILL.md", Some("a\nb\n"))]);

        let cleanup = auto_cleanup_unshipped_files(&source, &dest, &record);

        assert_eq!(cleanup.report, vec!["1 file(s) no longer shipped"]);
        assert!(!dest.join("skills/gone").exists());
    }
}
