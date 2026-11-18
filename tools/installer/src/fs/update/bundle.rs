use super::{PUBLIC_KEY, REPO, archive, executable, fetch, origin, platform};
use anyhow::{Context, Result, ensure};
use reqwest::blocking::Client;
use semver::Version;
use sha2::{Digest, Sha256};
use std::{fs, path::Path, process::Command};
use tempfile::TempDir;

fn digest(key: &str, sums: &[u8], signature: &[u8], asset: &str) -> Result<String> {
    let key = minisign_verify::PublicKey::decode(key)?;
    let sig = minisign_verify::Signature::decode(std::str::from_utf8(signature)?)?;
    key.verify(sums, &sig, false)
        .context("Invalid release signature")?;
    let mut matches = std::str::from_utf8(sums)?
        .lines()
        .filter_map(|line| line.split_once("  "))
        .filter_map(|(hash, name)| (name == asset).then_some(hash));
    let hash = matches
        .next()
        .context("Asset missing from signed checksums")?;
    ensure!(
        matches.next().is_none() && hash.len() == 64 && hash.bytes().all(|c| c.is_ascii_hexdigit()),
        "Invalid checksum record"
    );
    Ok(hash.to_owned())
}

pub(super) fn probe(exe: &Path, version: &Version) -> Result<()> {
    let actual = origin::run(Command::new(exe).arg("--version"), 15)?;
    ensure!(
        actual.trim() == format!("hibi v{version} (Config Installer)"),
        "Unexpected installed executable version"
    );
    Ok(())
}

pub(super) fn install(client: &Client, root: &Path, version: &Version) -> Result<()> {
    let base = format!("https://github.com/{REPO}/releases/download/v{version}");
    let stage = download(client, root, version, &base, PUBLIC_KEY)?;
    #[cfg(windows)]
    {
        super::windows::replace(root, stage, version)
    }
    #[cfg(unix)]
    {
        activate(root, stage, version, |exe| {
            crate::exec::launch(&mut Command::new(exe))
        })
    }
}

fn download(
    client: &Client,
    root: &Path,
    version: &Version,
    base: &str,
    key: &str,
) -> Result<TempDir> {
    let prefix = format!("hibi-ai-{version}-{}", platform());
    let ext = if cfg!(windows) { "zip" } else { "tar.gz" };
    let name = format!("{prefix}.{ext}");
    let sums = fetch(client, &format!("{base}/checksums.txt"), 1 << 20)?;
    let sig = fetch(client, &format!("{base}/checksums.txt.minisig"), 8192)?;
    let hash = digest(key, &sums, &sig, &name)?;
    let bytes = fetch(client, &format!("{base}/{name}"), 256 << 20)?;
    stage(root, version, &prefix, &hash, &bytes)
}

fn stage(
    root: &Path,
    version: &Version,
    prefix: &str,
    hash: &str,
    bytes: &[u8],
) -> Result<TempDir> {
    ensure!(
        format!("{:x}", Sha256::digest(bytes)).eq_ignore_ascii_case(hash),
        "Archive checksum mismatch"
    );
    let mut staging = tempfile::Builder::new();
    staging.prefix(".hibi-update-");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        staging.permissions(fs::Permissions::from_mode(0o700));
    }
    let stage = staging.tempdir_in(root.parent().context("Missing bundle parent")?)?;
    let next = stage.path().join("bundle");
    fs::create_dir(&next)?;
    archive::extract(bytes, prefix, &next, cfg!(windows))?;
    ensure!(
        origin::marker_matches(&next, &version.to_string())
            && next.join("skills").is_dir()
            && next.join("settings.json").is_file(),
        "Incomplete release bundle"
    );
    probe(&next.join(executable()), version)?;
    Ok(stage)
}

#[cfg(unix)]
fn exchange(left: &Path, right: &Path) -> Result<()> {
    use std::{ffi::CString, os::unix::ffi::OsStrExt};
    let a = CString::new(left.as_os_str().as_bytes())?;
    let b = CString::new(right.as_os_str().as_bytes())?;
    // Both paths live on the same volume. The kernel exchanges the directory entries atomically.
    #[cfg(target_os = "macos")]
    let result = unsafe {
        libc::renameatx_np(
            libc::AT_FDCWD,
            a.as_ptr(),
            libc::AT_FDCWD,
            b.as_ptr(),
            libc::RENAME_SWAP,
        )
    };
    #[cfg(target_os = "linux")]
    let result = unsafe {
        libc::syscall(
            libc::SYS_renameat2,
            libc::AT_FDCWD,
            a.as_ptr(),
            libc::AT_FDCWD,
            b.as_ptr(),
            libc::RENAME_EXCHANGE,
        )
    };
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    anyhow::bail!("Directory exchange unsupported on this platform");
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    if result != 0 {
        return Err(std::io::Error::last_os_error())
            .context("Cannot exchange installation directories");
    }
    Ok(())
}

#[cfg(unix)]
fn activate(
    root: &Path,
    stage: TempDir,
    version: &Version,
    launch: impl FnOnce(&Path) -> Result<()>,
) -> Result<()> {
    let lock = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .open(
            root.parent()
                .context("Missing bundle parent")?
                .join(".hibi-update.lock"),
        )?;
    lock.try_lock().context("Another update is running")?;
    ensure!(
        origin::direct_root(root, env!("CARGO_PKG_VERSION")),
        "Installed bundle changed"
    );
    let next = stage.path().join("bundle");
    exchange(root, &next)?;
    // Keep the old bundle, including locally added files, even if recovery fails.
    let backup = stage.keep();
    let exe = root.join(executable());
    if let Err(error) = probe(&exe, version).and_then(|()| launch(&exe)) {
        exchange(root, &next).with_context(|| {
            format!(
                "Restore the previous installation from {}",
                backup.join("bundle").display()
            )
        })?;
        return Err(error).context("Update failed; previous installation restored");
    }
    Ok(())
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    #[test]
    #[ignore = "run with tests/update_smoke.py and two isolated native builds"]
    fn signed_native_update() -> Result<()> {
        let work = std::path::PathBuf::from(std::env::var("HIBI_TEST_UPDATE_DIR")?);
        let version = Version::parse(&std::env::var("HIBI_TEST_NEXT_VERSION")?)?;
        let current = Version::parse(env!("CARGO_PKG_VERSION"))?;
        let root = work.join(format!("hibi-ai-{current}-{}", platform()));
        let prefix = format!("hibi-ai-{version}-{}", platform());
        let asset = format!("{prefix}.tar.gz");
        let sums = fs::read(work.join("checksums.txt"))?;
        let sig = fs::read(work.join("checksums.txt.minisig"))?;
        let key = fs::read_to_string(work.join("fixture.pub"))?;
        let base = std::env::var("HIBI_TEST_UPDATE_URL")?;
        let client = Client::builder().no_proxy().build()?;
        let hash = digest(&key, &sums, &sig, &asset)?;
        assert!(digest(PUBLIC_KEY, &sums, &sig, &asset).is_err());
        assert!(digest(&key, b"changed", &sig, &asset).is_err());
        assert!(digest(&key, &sums, &sig, "other.tar.gz").is_err());
        assert!(stage(&root, &version, &prefix, &hash, b"damaged").is_err());
        let staged = || download(&client, &root, &version, &base, &key);
        assert!(
            activate(&root, staged()?, &version, |_| anyhow::bail!(
                "relaunch failed"
            ))
            .is_err()
        );
        probe(&root.join(executable()), &current)?;
        assert_eq!(fs::read_to_string(root.join("user-added-file"))?, "keep");
        activate(&root, staged()?, &version, |exe| {
            origin::run(
                Command::new("python3")
                    .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/update_smoke.py"))
                    .arg("--launch")
                    .arg(exe),
                45,
            )?;
            Ok(())
        })?;
        assert_eq!(
            fs::read_to_string(root.join("skills/probe"))?,
            version.to_string()
        );
        assert!(origin::direct_root(&root, &version.to_string()));
        assert!(activate(&root, staged()?, &version, |_| Ok(())).is_err());
        Ok(())
    }
}
