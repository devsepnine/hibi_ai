//! Records where an installation came from, so an install-only user can find
//! the upstream without cloning it.
//!
//! Nothing in `~/.claude` names this project, so someone who installed via
//! Homebrew or Scoop has no way to tell which repo produced their config, which
//! version they are on, or where to send an improvement. This module writes that
//! provenance to `~/.hibi/install.json` — hibi's own directory, so the
//! agent-owned `~/.claude` tree stays untouched.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::component::{Component, ComponentType, InstallStatus};

/// Upstream this config is distributed from — the only place a user can send
/// an improvement back to.
pub const SOURCE_REPO: &str = "https://github.com/devsepnine/hibi_ai";

/// Source label the installer gives components that came from the bundled
/// (package-embedded) config rather than a user-configured source.
const BUNDLED_SOURCE: &str = "bundled";

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct InstallManifest {
    /// Repository the bundled config came from — where improvements go back.
    pub source: String,
    /// Installer version that wrote this file, e.g. `v1.14.1`. Matches a
    /// release tag, so the exact source tree is recoverable from it.
    pub version: String,
    /// Destination directory name (`.claude` or `.codex`) — the same installer
    /// serves both.
    pub target: String,
    /// UTC timestamp of the last write, ISO-8601.
    pub updated_at: String,
    /// Components from the bundled source as `<type>/<name>`, sorted.
    ///
    /// Filesystem-backed components only. MCP servers and plugins are
    /// installed through the CLI and are not tracked here, so an MCP-only or
    /// plugin-only run leaves this list and `updated_at` unchanged.
    pub components: Vec<String>,
    /// Labels of any user-configured sources that also contributed installed
    /// components. They are listed rather than merged into `components`
    /// because `source` does not describe them — a component from another
    /// source must not be sent upstream to this repository.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub other_sources: Vec<String>,
    /// SHA-256 of each file exactly as hibi wrote it, line endings normalized,
    /// keyed by component ID. It lets a later run tell a file hibi left
    /// untouched from one the user edited. A file hibi cannot vouch for has no
    /// entry, and a record written before this field existed has none.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub hashes: BTreeMap<String, String>,
}

/// What the last install into a directory recorded.
#[derive(Debug, Default, PartialEq)]
pub struct InstallRecord {
    pub ids: Vec<String>,
    pub hashes: BTreeMap<String, String>,
}

/// Lowercase hex SHA-256 of `content` with line endings normalized, matching
/// the scanner, which already treats a CRLF and an LF copy as the same file.
pub fn content_hash(content: &[u8]) -> String {
    Sha256::digest(super::normalize_line_endings(content))
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// `~/.hibi/install.json`.
pub fn manifest_path() -> Result<PathBuf> {
    let home = crate::paths::require_home_dir()?;
    Ok(home.join(".hibi").join("install.json"))
}

/// What the last install into `dest_dir` recorded. A missing or unreadable
/// record, or one written for any other directory, yields an empty one, so a
/// caller that deletes recorded files deletes nothing.
pub fn recorded_install(dest_dir: &Path) -> InstallRecord {
    let (Ok(path), Some(home)) = (manifest_path(), crate::paths::home_dir()) else {
        return InstallRecord::default();
    };
    fs::read_to_string(path)
        .map(|text| record_for_target(&text, dest_dir, &home))
        .unwrap_or_default()
}

/// The record names only the target directory, and an install always goes
/// directly under home, so any other `dest_dir` is not the one it describes.
fn record_for_target(record: &str, dest_dir: &Path, home: &Path) -> InstallRecord {
    let Ok(manifest) = serde_json::from_str::<InstallManifest>(record) else {
        return InstallRecord::default();
    };
    if dest_dir != home.join(&manifest.target) {
        return InstallRecord::default();
    }
    InstallRecord {
        ids: manifest.components,
        hashes: manifest.hashes,
    }
}

fn is_installed(component: &Component) -> bool {
    // `New` never landed, and `External` is a user's own file that no source
    // produces — claiming either as installed would misreport provenance.
    matches!(
        component.status,
        InstallStatus::Unchanged | InstallStatus::Modified | InstallStatus::Managed
    )
}

fn component_id(component: &Component) -> String {
    format!(
        "{}/{}",
        component.component_type.display_name(),
        component.name
    )
}

/// Components present in the destination that came from the bundled source.
pub fn installed_component_ids(components: &[Component]) -> Vec<String> {
    let mut ids: Vec<String> = components
        .iter()
        .filter(|c| is_installed(c) && c.source_name == BUNDLED_SOURCE)
        .map(component_id)
        .collect();
    ids.sort();
    ids.dedup();
    ids
}

/// Hashes of the bundled files that still hold what hibi wrote.
///
/// An `Unchanged` file matched its source when the scan ran, so the source is
/// hashed rather than the destination: an edit the user saves after the scan
/// must not be recorded as hibi's. A `Modified` file is vouched for only while
/// it still holds exactly what an earlier run recorded, which is the content
/// an older hibi wrote. Anything else may hold the user's
/// edits, so it gets no hash, and a later cleanup keeps it.
fn hashes_for(
    components: &[Component],
    previous: &BTreeMap<String, String>,
) -> BTreeMap<String, String> {
    components
        .iter()
        .filter(|c| {
            c.source_name == BUNDLED_SOURCE
                && ComponentType::TREE_MIRRORED.contains(&c.component_type)
        })
        .filter_map(|c| {
            let id = component_id(c);
            let hash = match c.status {
                InstallStatus::Unchanged => content_hash(&fs::read(&c.source_path).ok()?),
                InstallStatus::Modified => {
                    let hash = content_hash(&fs::read(&c.dest_path).ok()?);
                    (previous.get(&id) == Some(&hash)).then_some(hash)?
                }
                _ => return None,
            };
            Some((id, hash))
        })
        .collect()
}

/// Labels of other configured sources that contributed installed components.
///
/// Recorded separately so a reader never reads `source` as the origin of a
/// component that came from somewhere else.
pub fn other_source_labels(components: &[Component]) -> Vec<String> {
    let mut labels: Vec<String> = components
        .iter()
        .filter(|c| is_installed(c) && c.source_name != BUNDLED_SOURCE)
        .map(|c| c.source_name.clone())
        .collect();
    labels.sort();
    labels.dedup();
    labels
}

/// Format epoch seconds as an ISO-8601 UTC timestamp.
///
/// Hand-rolled because the crate carries no date dependency and provenance is
/// read by people — a bare epoch would make the file useless at a glance.
fn format_utc(epoch_secs: u64) -> String {
    let days = (epoch_secs / 86_400) as i64;
    let secs_of_day = epoch_secs % 86_400;

    // Civil-from-days (Howard Hinnant's algorithm), shifted to a 1970 epoch.
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        y,
        m,
        d,
        secs_of_day / 3_600,
        (secs_of_day % 3_600) / 60,
        secs_of_day % 60
    )
}

fn target_name(dest_dir: &Path) -> String {
    dest_dir
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| ".claude".to_string())
}

fn build(
    dest_dir: &Path,
    component_ids: Vec<String>,
    other_sources: Vec<String>,
    epoch_secs: u64,
) -> InstallManifest {
    InstallManifest {
        source: SOURCE_REPO.to_string(),
        version: super::VERSION.to_string(),
        target: target_name(dest_dir),
        updated_at: format_utc(epoch_secs),
        components: component_ids,
        other_sources,
        hashes: BTreeMap::new(),
    }
}

/// Write the manifest, replacing any previous one.
///
/// The component list is a snapshot of what is installed now rather than a
/// union across runs, so removals are reflected instead of accumulating.
pub fn write(dest_dir: &Path, components: &[Component]) -> Result<()> {
    write_to(&manifest_path()?, dest_dir, components)
}

/// `write` with the destination path injected, so the file-writing path itself
/// is testable without touching the real home directory.
fn write_to(path: &Path, dest_dir: &Path, components: &[Component]) -> Result<()> {
    let mut manifest = build(
        dest_dir,
        installed_component_ids(components),
        other_source_labels(components),
        now_epoch_secs()?,
    );
    let previous = previous_hashes(path, &manifest.target);
    manifest.hashes = hashes_for(components, &previous);
    publish(path, &manifest)
}

/// Drop `ids` from the record for `dest_dir`, so a later run does not judge
/// them again. A missing or unreadable record, or one another hibi has
/// since rewritten for a different target, is left as it is.
pub fn forget(dest_dir: &Path, ids: &[String]) -> Result<()> {
    forget_in(&manifest_path()?, dest_dir, ids)
}

fn forget_in(path: &Path, dest_dir: &Path, ids: &[String]) -> Result<()> {
    let Some(mut manifest) = read_manifest(path) else {
        return Ok(());
    };
    if manifest.target != target_name(dest_dir) {
        return Ok(());
    }
    let before = manifest.components.len();
    manifest.components.retain(|id| !ids.contains(id));
    manifest.hashes.retain(|id, _| !ids.contains(id));
    if manifest.components.len() == before {
        return Ok(());
    }
    manifest.updated_at = format_utc(now_epoch_secs()?);
    publish(path, &manifest)
}

fn read_manifest(path: &Path) -> Option<InstallManifest> {
    serde_json::from_str(&fs::read_to_string(path).ok()?).ok()
}

/// Hashes a record for the same target vouched for; a record for another
/// target describes other files.
fn previous_hashes(path: &Path, target: &str) -> BTreeMap<String, String> {
    read_manifest(path)
        .filter(|previous| previous.target == target)
        .map(|previous| previous.hashes)
        .unwrap_or_default()
}

fn now_epoch_secs() -> Result<u64> {
    // Refuse rather than stamp 1970: a wrong timestamp in a provenance file is
    // worse than an absent one, and the caller surfaces the refusal as a
    // warning without failing the install.
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("System clock reads before the Unix epoch; refusing to write a false timestamp")?
        .as_secs())
}

/// Writes to a temp file and renames, so an interrupted write cannot leave a
/// half-parsed manifest behind.
fn publish(path: &Path, manifest: &InstallManifest) -> Result<()> {
    let parent = path
        .parent()
        .context("Manifest path has no parent directory")?;
    fs::create_dir_all(parent).with_context(|| format!("Failed to create {}", parent.display()))?;

    let json = serde_json::to_string_pretty(manifest)?;
    // Per-process temp name: two hibi instances installing at once would
    // otherwise share one temp path, and the second rename would fail with
    // NotFound after the first moved the file out from under it.
    let tmp = path.with_extension(format!("json.{}.tmp", std::process::id()));
    fs::write(&tmp, &json).with_context(|| format!("Failed to write {}", tmp.display()))?;
    if let Err(e) = fs::rename(&tmp, path) {
        // Leave no stray temp file behind on a failed publish.
        let _ = fs::remove_file(&tmp);
        return Err(anyhow::Error::new(e).context(format!("Failed to replace {}", path.display())));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::ComponentType;
    use std::path::PathBuf;

    fn component(name: &str, ct: ComponentType, status: InstallStatus) -> Component {
        let mut c = Component::new(
            ct,
            name.to_string(),
            PathBuf::from("src"),
            PathBuf::from("dest"),
            status,
        );
        c.selected = false;
        c
    }

    fn from_source(name: &str, source: &str) -> Component {
        let mut c = component(name, ComponentType::Skills, InstallStatus::Unchanged);
        c.source_name = source.to_string();
        c
    }

    fn unique_dir(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("hibi_manifest_{label}_{nanos}"))
    }

    #[test]
    fn recorded_ids_come_back_only_for_the_matching_target() {
        let record = r#"{"source":"s","version":"v1","target":".claude",
            "updated_at":"t","components":["skills/iced_rs/SKILL.md"]}"#;

        let home = Path::new("/home/u");
        assert_eq!(
            record_for_target(record, &home.join(".claude"), home).ids,
            vec![String::from("skills/iced_rs/SKILL.md")]
        );
        assert_eq!(
            record_for_target(record, &home.join(".codex"), home),
            InstallRecord::default()
        );
        assert_eq!(
            record_for_target(record, Path::new("/work/project/.claude"), home),
            InstallRecord::default(),
            "a project .claude is not the directory the record describes"
        );
        assert_eq!(
            record_for_target("not json", &home.join(".claude"), home),
            InstallRecord::default()
        );
    }

    fn installed_at(
        dir: &Path,
        name: &str,
        text: &str,
        ct: ComponentType,
        status: InstallStatus,
    ) -> Component {
        let (source, dest) = (dir.join("source").join(name), dir.join(name));
        for path in [&source, &dest] {
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, text).unwrap();
        }
        Component::new(ct, name.to_string(), source, dest, status)
    }

    fn parse(path: &Path) -> InstallManifest {
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
    }

    fn claude() -> PathBuf {
        PathBuf::from("/Users/x/.claude")
    }

    #[test]
    fn an_unchanged_file_is_hashed_from_its_source() {
        // The user may save an edit between the scan and this write; the
        // source is what hibi wrote, so the edit is never vouched for.
        let dir = unique_dir("hash_source");
        let file = installed_at(
            &dir,
            "a.md",
            "as shipped",
            ComponentType::Skills,
            InstallStatus::Unchanged,
        );
        std::fs::write(&file.dest_path, "saved after the scan").unwrap();

        let hashes = hashes_for(&[file], &BTreeMap::new());

        assert_eq!(
            hashes.get("skills/a.md"),
            Some(&content_hash(b"as shipped"))
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn other_statuses_and_an_unrecorded_modified_file_get_no_hash() {
        let dir = unique_dir("hash_none");
        let components = [
            installed_at(
                &dir,
                "ext.md",
                "e",
                ComponentType::Skills,
                InstallStatus::External,
            ),
            installed_at(
                &dir,
                "new.md",
                "n",
                ComponentType::Skills,
                InstallStatus::New,
            ),
            installed_at(
                &dir,
                "mod.md",
                "m",
                ComponentType::Skills,
                InstallStatus::Modified,
            ),
        ];

        assert!(hashes_for(&components, &BTreeMap::new()).is_empty());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn forget_rewrites_nothing_when_no_id_matches_or_the_target_differs() {
        let dir = unique_dir("forget_noop");
        let path = dir.join("install.json");
        std::fs::create_dir_all(&dir).unwrap();
        let record = r#"{"source":"s","version":"v1","target":".claude","updated_at":"t","components":["a"]}"#;
        std::fs::write(&path, record).unwrap();

        forget_in(&path, &claude(), &[String::from("z")]).unwrap();
        forget_in(
            &path,
            &PathBuf::from("/Users/x/.codex"),
            &[String::from("a")],
        )
        .unwrap();

        assert_eq!(std::fs::read_to_string(&path).unwrap(), record);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn content_hash_is_the_sha256_of_lf_content() {
        assert_eq!(
            content_hash(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(content_hash(b"a\r\nb\r\n"), content_hash(b"a\nb\n"));
    }

    #[test]
    fn hashes_vouch_only_for_files_as_hibi_wrote_them() {
        let dir = unique_dir("hashes");
        let mut theirs = installed_at(
            &dir,
            "t.md",
            "t",
            ComponentType::Skills,
            InstallStatus::Unchanged,
        );
        theirs.source_name = String::from("team-configs");
        let components = [
            installed_at(
                &dir,
                "u.md",
                "u",
                ComponentType::Skills,
                InstallStatus::Unchanged,
            ),
            installed_at(
                &dir,
                "older.md",
                "older",
                ComponentType::Skills,
                InstallStatus::Modified,
            ),
            installed_at(
                &dir,
                "edited.md",
                "edited",
                ComponentType::Skills,
                InstallStatus::Modified,
            ),
            installed_at(
                &dir,
                "settings.json",
                "{}",
                ComponentType::ConfigFile,
                InstallStatus::Managed,
            ),
            theirs,
        ];
        let previous = BTreeMap::from([
            (String::from("skills/older.md"), content_hash(b"older")),
            (
                String::from("skills/edited.md"),
                content_hash(b"as hibi wrote it"),
            ),
        ]);

        let hashes = hashes_for(&components, &previous);

        assert_eq!(
            hashes,
            BTreeMap::from([
                (String::from("skills/older.md"), content_hash(b"older")),
                (String::from("skills/u.md"), content_hash(b"u")),
            ]),
            "an edited file, a merged config, and another source's file get no hash"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_record_carries_its_hashes_and_an_older_record_has_none() {
        let home = Path::new("/home/u");
        let with_hashes = r#"{"source":"s","version":"v1","target":".claude",
            "updated_at":"t","components":["skills/a.md"],"hashes":{"skills/a.md":"h"}}"#;
        let older = r#"{"source":"s","version":"v1","target":".claude",
            "updated_at":"t","components":["skills/a.md"]}"#;

        assert_eq!(
            record_for_target(with_hashes, &home.join(".claude"), home),
            InstallRecord {
                ids: vec![String::from("skills/a.md")],
                hashes: BTreeMap::from([(String::from("skills/a.md"), String::from("h"))]),
            }
        );
        assert!(
            record_for_target(older, &home.join(".claude"), home)
                .hashes
                .is_empty()
        );
    }

    #[test]
    fn forget_drops_the_ids_and_their_hashes_only() {
        let dir = unique_dir("forget");
        let path = dir.join("install.json");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            &path,
            r#"{"source":"s","version":"v1","target":".claude","updated_at":"t",
            "components":["a","b","c"],"hashes":{"a":"ha","b":"hb"}}"#,
        )
        .unwrap();

        forget_in(&path, &claude(), &[String::from("a"), String::from("c")]).unwrap();

        let parsed = parse(&path);
        assert_eq!(parsed.components, vec![String::from("b")]);
        assert_eq!(
            parsed.hashes,
            BTreeMap::from([(String::from("b"), String::from("hb"))])
        );
        assert_eq!(
            (parsed.source.as_str(), parsed.target.as_str()),
            ("s", ".claude")
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn forget_leaves_an_unreadable_record_alone() {
        let dir = unique_dir("forget_bad");
        let path = dir.join("install.json");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(&path, "not json").unwrap();

        forget_in(&path, &claude(), &[String::from("a")]).unwrap();

        assert_eq!(std::fs::read_to_string(&path).unwrap(), "not json");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn write_to_keeps_vouching_for_a_file_an_older_hibi_wrote() {
        let dir = unique_dir("vouch");
        let path = dir.join("install.json");
        let claude = PathBuf::from("/Users/x/.claude");
        let file = |text, status| installed_at(&dir, "a.md", text, ComponentType::Skills, status);

        write_to(&path, &claude, &[file("v1", InstallStatus::Unchanged)]).unwrap();
        assert!(parse(&path).hashes.contains_key("skills/a.md"));

        // The source moved on and the file still holds what hibi wrote.
        write_to(&path, &claude, &[file("v1", InstallStatus::Modified)]).unwrap();
        assert!(parse(&path).hashes.contains_key("skills/a.md"));

        write_to(&path, &claude, &[file("edited", InstallStatus::Modified)]).unwrap();
        assert!(!parse(&path).hashes.contains_key("skills/a.md"));

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_record_for_another_target_vouches_for_nothing() {
        let dir = unique_dir("vouch_target");
        let path = dir.join("install.json");
        let file = |status| installed_at(&dir, "a.md", "v1", ComponentType::Skills, status);

        write_to(
            &path,
            &PathBuf::from("/Users/x/.claude"),
            &[file(InstallStatus::Unchanged)],
        )
        .unwrap();
        write_to(
            &path,
            &PathBuf::from("/Users/x/.codex"),
            &[file(InstallStatus::Modified)],
        )
        .unwrap();

        assert!(parse(&path).hashes.is_empty());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn format_utc_matches_known_timestamps() {
        assert_eq!(format_utc(0), "1970-01-01T00:00:00Z");
        assert_eq!(format_utc(1_000_000_000), "2001-09-09T01:46:40Z");
        // Leap-year day, to catch an off-by-one in the civil-date conversion.
        assert_eq!(format_utc(1_709_164_800), "2024-02-29T00:00:00Z");
        // Year-end wrap in both directions.
        assert_eq!(format_utc(1_735_689_599), "2024-12-31T23:59:59Z");
        assert_eq!(format_utc(1_735_689_600), "2025-01-01T00:00:00Z");
        // 2100 is not a leap year — a century rule the naive version gets wrong.
        assert_eq!(format_utc(4_107_542_400), "2100-03-01T00:00:00Z");
    }

    #[test]
    fn installed_ids_skip_new_and_external() {
        let components = vec![
            component(
                "qa-handoff",
                ComponentType::Skills,
                InstallStatus::Unchanged,
            ),
            component("commit", ComponentType::Commands, InstallStatus::Modified),
            component(
                "statusline",
                ComponentType::Statusline,
                InstallStatus::Managed,
            ),
            component(
                "settings.json",
                ComponentType::ConfigFile,
                InstallStatus::Managed,
            ),
            component("not-yet", ComponentType::Skills, InstallStatus::New),
            component("mine", ComponentType::Skills, InstallStatus::External),
        ];

        let ids = installed_component_ids(&components);

        assert_eq!(
            ids,
            vec![
                "commands/commit".to_string(),
                "config/settings.json".to_string(),
                "skills/qa-handoff".to_string(),
                "statusline/statusline".to_string(),
            ]
        );
    }

    #[test]
    fn components_from_other_sources_are_listed_separately_not_as_ours() {
        // `source` names this repository, so a component installed from a
        // user's own source must not appear under it.
        let components = vec![
            from_source("ours", BUNDLED_SOURCE),
            from_source("theirs", "team-configs"),
            from_source("also-theirs", "team-configs"),
            from_source("third-party", "dotfiles"),
        ];

        assert_eq!(
            installed_component_ids(&components),
            vec!["skills/ours".to_string()]
        );
        assert_eq!(
            other_source_labels(&components),
            vec!["dotfiles".to_string(), "team-configs".to_string()]
        );
    }

    #[test]
    fn installed_ids_are_sorted_and_deduped() {
        let components = vec![
            component("b", ComponentType::Skills, InstallStatus::Unchanged),
            component("a", ComponentType::Skills, InstallStatus::Unchanged),
            component("a", ComponentType::Skills, InstallStatus::Modified),
        ];

        assert_eq!(
            installed_component_ids(&components),
            vec!["skills/a".to_string(), "skills/b".to_string()]
        );
    }

    #[test]
    fn build_records_source_version_and_target() {
        let manifest = build(
            &PathBuf::from("/Users/x/.codex"),
            vec!["skills/qa-handoff".to_string()],
            Vec::new(),
            1_709_164_800,
        );

        assert_eq!(manifest.source, SOURCE_REPO);
        assert_eq!(manifest.version, super::super::VERSION);
        assert_eq!(manifest.target, ".codex");
        assert_eq!(manifest.updated_at, "2024-02-29T00:00:00Z");
        assert_eq!(manifest.components, vec!["skills/qa-handoff".to_string()]);
    }

    #[test]
    fn build_falls_back_to_claude_for_a_rootless_dest() {
        let manifest = build(&PathBuf::from("/"), Vec::new(), Vec::new(), 0);
        assert_eq!(manifest.target, ".claude");
    }

    #[test]
    fn write_to_creates_a_parsable_manifest_and_leaves_no_temp_file() {
        let dir = unique_dir("write");
        let path = dir.join("install.json");
        let components = vec![component(
            "qa-handoff",
            ComponentType::Skills,
            InstallStatus::Unchanged,
        )];

        write_to(&path, &PathBuf::from("/Users/x/.claude"), &components).unwrap();

        let parsed: InstallManifest =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(parsed.source, SOURCE_REPO);
        assert_eq!(parsed.target, ".claude");
        assert_eq!(parsed.components, vec!["skills/qa-handoff".to_string()]);
        assert!(parsed.updated_at.ends_with('Z'));
        assert!(
            !path
                .with_extension(format!("json.{}.tmp", std::process::id()))
                .exists()
        );

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn write_to_replaces_a_previous_manifest_rather_than_appending() {
        let dir = unique_dir("replace");
        let path = dir.join("install.json");
        let dest = PathBuf::from("/Users/x/.claude");

        write_to(
            &path,
            &dest,
            &[component(
                "a",
                ComponentType::Skills,
                InstallStatus::Unchanged,
            )],
        )
        .unwrap();
        write_to(
            &path,
            &dest,
            &[component(
                "b",
                ComponentType::Skills,
                InstallStatus::Unchanged,
            )],
        )
        .unwrap();

        let parsed: InstallManifest =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(parsed.components, vec!["skills/b".to_string()]);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn write_to_reports_why_it_failed_instead_of_failing_silently() {
        // The caller turns this error into a warning line, so the message is
        // the only trace a user gets — it has to name the path it could not
        // create. A regular file where the parent directory belongs is the
        // cheapest way to force the failure.
        let dir = unique_dir("blocked");
        std::fs::create_dir_all(&dir).unwrap();
        let blocker = dir.join("install-dir");
        std::fs::write(&blocker, "not a directory").unwrap();
        let path = blocker.join("install.json");

        let err = write_to(&path, &PathBuf::from("/Users/x/.claude"), &[]).unwrap_err();

        let message = format!("{:#}", err);
        assert!(
            message.contains("install-dir"),
            "error should name the path: {message}"
        );
        assert!(!path.exists());

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn manifest_round_trips_through_json() {
        let manifest = build(
            &PathBuf::from("/Users/x/.claude"),
            vec!["skills/a".to_string()],
            vec!["team-configs".to_string()],
            0,
        );
        let json = serde_json::to_string_pretty(&manifest).unwrap();
        let parsed: InstallManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, manifest);
    }
}
