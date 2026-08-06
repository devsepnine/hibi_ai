//! Records where an installation came from, so an install-only user can find
//! the upstream without cloning it.
//!
//! Nothing in `~/.claude` names this project, so someone who installed via
//! Homebrew or Scoop has no way to tell which repo produced their config, which
//! version they are on, or where to send an improvement. This module writes that
//! provenance to `~/.hibi/install.json` — hibi's own directory, so the
//! agent-owned `~/.claude` tree stays untouched.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::component::{Component, InstallStatus};

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
}

/// `~/.hibi/install.json`.
pub fn manifest_path() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not determine home directory")?;
    Ok(home.join(".hibi").join("install.json"))
}

fn is_installed(component: &Component) -> bool {
    // `New` never landed, and `External` is a user's own file that no source
    // produces — claiming either as installed would misreport provenance.
    matches!(
        component.status,
        InstallStatus::Unchanged | InstallStatus::Modified | InstallStatus::Managed
    )
}

/// Components present in the destination that came from the bundled source.
pub fn installed_component_ids(components: &[Component]) -> Vec<String> {
    let mut ids: Vec<String> = components
        .iter()
        .filter(|c| is_installed(c) && c.source_name == BUNDLED_SOURCE)
        .map(|c| format!("{}/{}", c.component_type.display_name(), c.name))
        .collect();
    ids.sort();
    ids.dedup();
    ids
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

fn build(
    dest_dir: &Path,
    component_ids: Vec<String>,
    other_sources: Vec<String>,
    epoch_secs: u64,
) -> InstallManifest {
    let target = dest_dir
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| ".claude".to_string());

    InstallManifest {
        source: SOURCE_REPO.to_string(),
        version: super::VERSION.to_string(),
        target,
        updated_at: format_utc(epoch_secs),
        components: component_ids,
        other_sources,
    }
}

/// Write the manifest, replacing any previous one.
///
/// The component list is a snapshot of what is installed now rather than a
/// union across runs, so removals are reflected instead of accumulating.
/// Writes to a temp file and renames, so an interrupted write cannot leave a
/// half-parsed manifest behind.
pub fn write(dest_dir: &Path, components: &[Component]) -> Result<()> {
    write_to(&manifest_path()?, dest_dir, components)
}

/// `write` with the destination path injected, so the file-writing path itself
/// is testable without touching the real home directory.
fn write_to(path: &Path, dest_dir: &Path, components: &[Component]) -> Result<()> {
    // Refuse rather than stamp 1970: a wrong timestamp in a provenance file is
    // worse than an absent one, and the caller surfaces the refusal as a
    // warning without failing the install.
    let epoch_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("System clock reads before the Unix epoch; refusing to write a false timestamp")?
        .as_secs();
    let manifest = build(
        dest_dir,
        installed_component_ids(components),
        other_source_labels(components),
        epoch_secs,
    );

    let parent = path
        .parent()
        .context("Manifest path has no parent directory")?;
    fs::create_dir_all(parent)
        .with_context(|| format!("Failed to create {}", parent.display()))?;

    let json = serde_json::to_string_pretty(&manifest)?;
    // Per-process temp name: two hibi instances installing at once would
    // otherwise share one temp path, and the second rename would fail with
    // NotFound after the first moved the file out from under it.
    let tmp = path.with_extension(format!("json.{}.tmp", std::process::id()));
    fs::write(&tmp, &json)
        .with_context(|| format!("Failed to write {}", tmp.display()))?;
    if let Err(e) = fs::rename(&tmp, path) {
        // Leave no stray temp file behind on a failed publish.
        let _ = fs::remove_file(&tmp);
        return Err(anyhow::Error::new(e)
            .context(format!("Failed to replace {}", path.display())));
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
            component("qa-handoff", ComponentType::Skills, InstallStatus::Unchanged),
            component("commit", ComponentType::Commands, InstallStatus::Modified),
            component("statusline", ComponentType::Statusline, InstallStatus::Managed),
            component("settings.json", ComponentType::ConfigFile, InstallStatus::Managed),
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
            &[component("a", ComponentType::Skills, InstallStatus::Unchanged)],
        )
        .unwrap();
        write_to(
            &path,
            &dest,
            &[component("b", ComponentType::Skills, InstallStatus::Unchanged)],
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
