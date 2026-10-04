//! Filesystem fixtures shared by the cleanup tests.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub(super) fn unique_dir(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("hibi_cleanup_{label}_{nanos}"));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

pub(super) fn put(root: &Path, rel: &str) {
    put_text(root, rel, "x");
}

pub(super) fn put_text(root: &Path, rel: &str, text: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, text).unwrap();
}

#[cfg(unix)]
pub(super) fn link_dir(target: &Path, link: &Path) -> bool {
    std::os::unix::fs::symlink(target, link).is_ok()
}

#[cfg(windows)]
pub(super) fn link_dir(target: &Path, link: &Path) -> bool {
    // Symlinks need a privilege on Windows; a junction does not, and it is
    // the link a user is most likely to have made. mklink rejects `/`, which
    // `Path::join` leaves in place.
    let native = |p: &Path| p.to_string_lossy().replace('/', "\\");
    std::process::Command::new("cmd")
        .args(["/C", "mklink", "/J"])
        .arg(native(link))
        .arg(native(target))
        .output()
        .is_ok_and(|out| out.status.success())
}

/// Make `file` impossible to delete while the returned guard lives:
/// Windows refuses to delete a file another handle holds without
/// delete sharing, and Unix refuses to unlink from a read-only directory.
#[cfg(windows)]
pub(super) fn make_undeletable(file: &Path) -> std::fs::File {
    use std::os::windows::fs::OpenOptionsExt;
    std::fs::OpenOptions::new()
        .read(true)
        .share_mode(0)
        .open(file)
        .unwrap()
}

#[cfg(unix)]
pub(super) struct RestorePermissions {
    dir: PathBuf,
    before: std::fs::Permissions,
}

#[cfg(unix)]
impl Drop for RestorePermissions {
    fn drop(&mut self) {
        let _ = std::fs::set_permissions(&self.dir, self.before.clone());
    }
}

#[cfg(unix)]
pub(super) fn make_undeletable(file: &Path) -> RestorePermissions {
    set_mode(file.parent().unwrap(), 0o555)
}

/// Deny every access to `dir`, so looking up anything inside it fails with
/// a permission error rather than NotFound.
#[cfg(unix)]
pub(super) fn make_unreadable(dir: &Path) -> RestorePermissions {
    set_mode(dir, 0o000)
}

#[cfg(unix)]
fn set_mode(dir: &Path, mode: u32) -> RestorePermissions {
    use std::os::unix::fs::PermissionsExt;
    let before = std::fs::metadata(dir).unwrap().permissions();
    std::fs::set_permissions(dir, std::fs::Permissions::from_mode(mode)).unwrap();
    RestorePermissions {
        dir: dir.to_path_buf(),
        before,
    }
}
