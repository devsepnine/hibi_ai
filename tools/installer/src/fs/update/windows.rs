use super::origin::run;
use anyhow::{Context, Result};
use base64::Engine;
use semver::Version;
use std::{
    path::{Path, PathBuf},
    process::Command,
};
use tempfile::TempDir;

fn powershell(script: &str) -> Result<Command> {
    let system = std::env::var_os("SystemRoot").context("Cannot locate Windows PowerShell")?;
    let mut cmd =
        Command::new(PathBuf::from(system).join("System32/WindowsPowerShell/v1.0/powershell.exe"));
    let utf16: Vec<u8> = script.encode_utf16().flat_map(u16::to_le_bytes).collect();
    cmd.args(["-NoProfile", "-NonInteractive", "-EncodedCommand"])
        .arg(base64::engine::general_purpose::STANDARD.encode(utf16));
    Ok(cmd)
}

pub(super) fn owns_scoop_install(path: &Path) -> Result<bool> {
    let script = "$ErrorActionPreference='Stop'; & scoop prefix hibi-ai; exit $LASTEXITCODE";
    let prefix = run(&mut powershell(script)?, 30)?;
    Ok(Path::new(prefix.trim()).canonicalize().ok().as_deref() == Some(path))
}

fn worker(root: &Path, version: &Version) -> Result<Command> {
    let mut cmd = powershell(include_str!("windows-update.ps1"))?;
    cmd.current_dir(root.parent().context("Missing installation parent")?)
        .env("HIBI_UPDATE_PARENT", std::process::id().to_string())
        .env("HIBI_UPDATE_ROOT", dunce::simplified(root))
        .env("HIBI_UPDATE_FROM", env!("CARGO_PKG_VERSION"))
        .env("HIBI_UPDATE_TO", version.to_string());
    Ok(cmd)
}

pub(super) fn replace(root: &Path, stage: TempDir, version: &Version) -> Result<()> {
    let mut cmd = worker(root, version)?;
    cmd.env("HIBI_UPDATE_KIND", "direct")
        .env("HIBI_UPDATE_STAGE", dunce::simplified(stage.path()));
    crate::exec::launch(&mut cmd)?;
    let _ = stage.keep();
    Ok(())
}

pub(super) fn upgrade_scoop(exe: &Path, version: &Version) -> Result<()> {
    let root = exe.parent().context("Missing Scoop installation")?;
    crate::exec::launch(worker(root, version)?.env("HIBI_UPDATE_KIND", "scoop"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windows_worker_restarts_new_binary_and_restores_old_bundle_after_failed_probe() -> Result<()>
    {
        use std::{
            fs, thread,
            time::{Duration, Instant},
        };
        let temp = tempfile::tempdir()?;
        let stub = temp.path().join("probe.rs");
        fs::write(
            &stub,
            r#"
            fn main() {
                let dir=std::env::current_exe().unwrap().parent().unwrap().to_owned();
                let version=std::fs::read_to_string(dir.join(".hibi-bundle")).unwrap();
                if std::env::args().any(|arg|arg=="--version") {
                    println!("hibi v{} (Config Installer)",version);
                } else {
                    std::fs::write(dir.join("launched"),version).unwrap();
                }
            }"#,
        )?;
        let exe = temp.path().join("probe.exe");
        run(Command::new("rustc").arg(stub).arg("-o").arg(&exe), 60)?;
        for succeeds in [true, false] {
            let directory = tempfile::tempdir_in(temp.path())?;
            let root = directory.path().join("installed bundle");
            let stage = directory.path().join("stage");
            let bundle = stage.join("bundle");
            let version = "2.0.0-beta.1";
            for (path, marker) in [
                (&root, "2.0.0-beta"),
                (&bundle, if succeeds { version } else { "wrong" }),
            ] {
                fs::create_dir_all(path)?;
                fs::copy(&exe, path.join("hibi.exe"))?;
                fs::write(path.join(".hibi-bundle"), marker)?;
            }
            let script = format!(
                "& {{\n{}\n}} -ReportFailure {{ param($reason) [Console]::Error.WriteLine($reason) }}; exit $LASTEXITCODE",
                include_str!("windows-update.ps1")
            );
            let mut cmd = powershell(&script)?;
            cmd.env("HIBI_UPDATE_PARENT", "2147483647")
                .env("HIBI_UPDATE_ROOT", &root)
                .env("HIBI_UPDATE_STAGE", &stage)
                .env("HIBI_UPDATE_KIND", "direct")
                .env("HIBI_UPDATE_FROM", "2.0.0-beta")
                .env("HIBI_UPDATE_TO", version);
            assert_eq!(run(&mut cmd, 60).is_ok(), succeeds);
            let until = Instant::now() + Duration::from_secs(10);
            while !root.join("launched").is_file() && Instant::now() < until {
                thread::sleep(Duration::from_millis(50));
            }
            assert_eq!(
                fs::read_to_string(root.join("launched"))?,
                if succeeds { version } else { "2.0.0-beta" }
            );
        }
        Ok(())
    }
}
