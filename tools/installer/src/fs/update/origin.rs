use super::{executable, fetch, platform};
use anyhow::{Context, Result, ensure};
use reqwest::blocking::Client;
use semver::Version;
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Route {
    Direct(PathBuf),
    Brew {
        program: PathBuf,
    },
    #[cfg(windows)]
    Scoop,
    Pending,
    Manual,
}

fn json_file(path: &Path) -> Option<Value> {
    serde_json::from_slice(&fs::read(path).ok()?).ok()
}

pub(super) fn resolve(exe: &Path, version: &Version, client: &Client) -> Result<Route> {
    let root = exe.parent().context("Missing installation directory")?;
    if let Some(keg) = root.parent()
        && let Some(receipt) = json_file(&keg.join("INSTALL_RECEIPT.json"))
    {
        if receipt.pointer("/source/tap").and_then(Value::as_str) != Some("devsepnine/brew") {
            return Ok(Route::Manual);
        }
        let Some(prefix_dir) = keg.ancestors().nth(3) else {
            return Ok(Route::Manual);
        };
        let program = prefix_dir.join("bin/brew");
        if !program.is_file() {
            return Ok(Route::Manual);
        }
        let prefix = run(
            Command::new(&program).args(["--prefix", "devsepnine/brew/hibi"]),
            20,
        )?;
        if Path::new(prefix.trim()).canonicalize()? != keg {
            return Ok(Route::Manual);
        }
        let formula = fetch(
            client,
            "https://raw.githubusercontent.com/devsepnine/homebrew-brew/HEAD/Formula/hibi.rb",
            65536,
        )?;
        let line = format!("version \"{version}\"");
        return Ok(
            if std::str::from_utf8(&formula)?
                .lines()
                .any(|s| s.trim() == line)
            {
                Route::Brew { program }
            } else {
                Route::Pending
            },
        );
    }
    #[cfg(windows)]
    if root
        .parent()
        .and_then(Path::file_name)
        .is_some_and(|name| name.eq_ignore_ascii_case("hibi-ai"))
        && (root.join("scoop-install.json").is_file() || root.join("install.json").is_file())
    {
        let installed = json_file(&root.join("scoop-manifest.json"))
            .or_else(|| json_file(&root.join("manifest.json")))
            .unwrap_or_default();
        if installed["homepage"].as_str() != Some("https://github.com/devsepnine/hibi_ai")
            || !super::windows::owns_scoop_install(root)?
        {
            return Ok(Route::Manual);
        }
        let bytes = fetch(
            client,
            "https://raw.githubusercontent.com/devsepnine/scoop-bucket/HEAD/hibi-ai.json",
            65536,
        )?;
        let manifest: Value = serde_json::from_slice(&bytes)?;
        return Ok(
            if manifest["version"].as_str() == Some(&version.to_string()) {
                Route::Scoop
            } else {
                Route::Pending
            },
        );
    }
    Ok(if direct_root(root, env!("CARGO_PKG_VERSION")) {
        Route::Direct(root.into())
    } else {
        Route::Manual
    })
}

pub(super) fn direct_root(root: &Path, current: &str) -> bool {
    let portable = root
        .file_name()
        .and_then(|n| n.to_str())
        .and_then(|s| s.strip_prefix("hibi-ai-"))
        .and_then(|s| s.strip_suffix(&format!("-{}", platform())))
        .is_some_and(|v| Version::parse(v).is_ok());
    let prefix_install = root.ends_with("lib/hibi-ai")
        && crate::paths::home_dir()
            .and_then(|p| p.canonicalize().ok())
            .is_some_and(|home| root.starts_with(home))
        && root
            .parent()
            .and_then(Path::parent)
            .and_then(|p| p.join("bin/hibi").canonicalize().ok())
            .is_some_and(|bin| bin == root.join(executable()));
    (portable || prefix_install) && marker_matches(root, current)
}

pub(super) fn marker_matches(root: &Path, version: &str) -> bool {
    fs::read_to_string(root.join(".hibi-bundle")).is_ok_and(|v| v.trim() == version)
}

pub(super) fn run(command: &mut Command, timeout: u64) -> Result<String> {
    let output = crate::fs::run_with_timeout(command, timeout)?;
    ensure!(
        output.status.success(),
        "{} failed ({}): {}",
        command.get_program().to_string_lossy(),
        output.status,
        String::from_utf8_lossy(&output.stderr)
            .trim()
            .chars()
            .take(500)
            .collect::<String>()
    );
    Ok(String::from_utf8(output.stdout)?)
}

pub(super) fn upgrade_brew(program: &Path, version: &Version) -> Result<()> {
    const FORMULA: &str = "devsepnine/brew/hibi";
    run(Command::new(program).arg("update"), 300)?;
    let info = run(
        Command::new(program).args(["info", "--json=v2", FORMULA]),
        30,
    )?;
    let info: Value = serde_json::from_str(&info)?;
    ensure!(
        info.pointer("/formulae/0/versions/stable")
            .and_then(Value::as_str)
            == Some(&version.to_string()),
        "Formula changed; check again"
    );
    run(
        Command::new(program)
            .env("HOMEBREW_NO_AUTO_UPDATE", "1")
            .args(["upgrade", FORMULA]),
        600,
    )?;
    let prefix = run(Command::new(program).args(["--prefix", FORMULA]), 30)?;
    let replacement = Path::new(prefix.trim()).join("bin/hibi");
    super::bundle::probe(&replacement, version)?;
    crate::exec::launch(&mut Command::new(replacement))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn direct_update_requires_a_dedicated_installation_and_matching_marker() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp
            .path()
            .join(format!("hibi-ai-2.0.0-beta-{}", platform()));
        fs::create_dir(&root).unwrap();
        let version = env!("CARGO_PKG_VERSION");
        assert!(!direct_root(&root, version));
        fs::write(root.join(".hibi-bundle"), version).unwrap();
        assert!(direct_root(&root, version));
        assert!(!direct_root(&root, "0.0.0"));
        fs::write(temp.path().join(".hibi-bundle"), version).unwrap();
        assert!(!direct_root(temp.path(), version));
    }

    #[cfg(unix)]
    #[test]
    fn brew_upgrade_checks_the_formula_version_before_install_and_relaunch() -> Result<()> {
        use std::os::unix::fs::PermissionsExt;
        let temp = tempfile::tempdir()?;
        let bin = temp.path().join("bin");
        fs::create_dir(&bin)?;
        let brew = temp.path().join("brew");
        fs::write(
            &brew,
            r#"#!/bin/sh
set -eu
prefix=$(dirname "$0")
case "$*" in
    update) ;;
    'info --json=v2 devsepnine/brew/hibi') cat "$prefix/info" ;;
    'upgrade devsepnine/brew/hibi') test "$HOMEBREW_NO_AUTO_UPDATE" = 1; touch "$prefix/upgraded" ;;
    '--prefix devsepnine/brew/hibi') echo "$prefix" ;;
    *) exit 1 ;;
esac
"#,
        )?;
        let exe = bin.join("hibi");
        fs::write(
            &exe,
            r#"#!/bin/sh
if [ "${1:-}" = --version ]; then echo 'hibi v2.0.0 (Config Installer)'; else touch "$(dirname "$0")/started"; fi
"#,
        )?;
        for path in [&brew, &exe] {
            fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
        }
        let info = temp.path().join("info");
        let version = Version::new(2, 0, 0);
        fs::write(&info, r#"{"formulae":[{"versions":{"stable":"1.25.0"}}]}"#)?;
        assert!(upgrade_brew(&brew, &version).is_err());
        assert!(!temp.path().join("upgraded").exists());
        fs::write(info, r#"{"formulae":[{"versions":{"stable":"2.0.0"}}]}"#)?;
        upgrade_brew(&brew, &version)?;
        let until = std::time::Instant::now() + std::time::Duration::from_secs(3);
        while !bin.join("started").exists() && std::time::Instant::now() < until {
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(bin.join("started").is_file());
        assert!(temp.path().join("upgraded").is_file());
        Ok(())
    }
}
