use std::collections::BTreeSet;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};

pub(crate) fn load(path: &Path) -> Result<BTreeSet<PathBuf>> {
    match std::fs::read(path) {
        Ok(content) => serde_json::from_slice(&content)
            .with_context(|| format!("Invalid visibility settings at {}", path.display())),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(BTreeSet::new()),
        Err(error) => Err(error).with_context(|| format!("Cannot read {}", path.display())),
    }
}

pub(crate) fn save(
    path: &Path,
    previous: &BTreeSet<PathBuf>,
    folders: &BTreeSet<PathBuf>,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Cannot create {}", parent.display()))?;
    }
    let lock_path = path.with_extension("lock");
    let lock = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(&lock_path)
        .with_context(|| format!("Cannot open {}", lock_path.display()))?;
    match lock.try_lock() {
        Ok(()) => {}
        Err(std::fs::TryLockError::WouldBlock) => {
            anyhow::bail!("Another installer is saving folder visibility. Try again.")
        }
        Err(std::fs::TryLockError::Error(error)) => {
            return Err(error).with_context(|| format!("Cannot lock {}", lock_path.display()));
        }
    }
    let _lock = VisibilityLock(lock);
    if load(path)? != *previous {
        anyhow::bail!(
            "Visibility settings changed in another session. Re-select the CLI to reload them."
        );
    }
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let temporary_path = path.with_extension(format!("{}.{stamp}.tmp", std::process::id()));
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary_path)
        .with_context(|| format!("Cannot create {}", temporary_path.display()))?;
    let pending = PendingFile(temporary_path);
    file.write_all(&serde_json::to_vec_pretty(folders)?)
        .with_context(|| format!("Cannot write {}", pending.0.display()))?;
    file.sync_all()?;
    drop(file);
    std::fs::rename(&pending.0, path).with_context(|| format!("Cannot replace {}", path.display()))
}

struct PendingFile(PathBuf);

struct VisibilityLock(std::fs::File);

impl Drop for VisibilityLock {
    fn drop(&mut self) {
        // Forked subprocesses can retain a descriptor after the writer closes its copy.
        let _ = self.0.unlock();
    }
}

impl Drop for PendingFile {
    fn drop(&mut self) {
        // A successful rename already consumed the temporary file.
        let _ = std::fs::remove_file(&self.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::OpenOptions;
    use std::sync::{Arc, Barrier};

    struct Fixture(PathBuf);

    impl Fixture {
        fn new() -> Self {
            static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
            let serial = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let stamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "hibi_visibility_save_{}_{stamp}_{serial}",
                std::process::id()
            ));
            std::fs::create_dir_all(&path).unwrap();
            Self(path)
        }

        fn path(&self) -> PathBuf {
            self.0.join("visibility.json")
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            std::fs::remove_dir_all(&self.0).unwrap();
        }
    }

    #[test]
    fn another_writer_lock_preserves_preferences_until_released() {
        let fixture = Fixture::new();
        let path = fixture.path();
        let previous = BTreeSet::new();
        let updated = BTreeSet::from([PathBuf::from("skills/.system")]);
        let lock = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(path.with_extension("lock"))
            .unwrap();
        lock.try_lock().unwrap();
        let lock = VisibilityLock(lock);
        let error = save(&path, &previous, &updated).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("Another installer is saving folder visibility")
        );
        assert!(!path.exists());
        drop(lock);
        save(&path, &previous, &updated).unwrap();
        assert_eq!(
            load(&path).unwrap(),
            BTreeSet::from([PathBuf::from("skills/.system")])
        );
    }

    #[test]
    fn guard_releases_the_lock_even_while_a_cloned_handle_remains_open() {
        let fixture = Fixture::new();
        let path = fixture.path();
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(path.with_extension("lock"))
            .unwrap();
        file.try_lock().unwrap();
        let inherited = file.try_clone().unwrap();
        drop(VisibilityLock(file));
        save(
            &path,
            &BTreeSet::new(),
            &BTreeSet::from([PathBuf::from("skills/.system")]),
        )
        .unwrap();
        assert_eq!(
            load(&path).unwrap(),
            BTreeSet::from([PathBuf::from("skills/.system")])
        );
        drop(inherited);
    }

    #[test]
    fn concurrent_writers_cannot_both_overwrite_the_same_snapshot() {
        let fixture = Fixture::new();
        let barrier = Arc::new(Barrier::new(2));
        let threads: Vec<_> = ["skills/.system", "skills/synced"]
            .into_iter()
            .map(|folder| {
                let path = fixture.path();
                let barrier = Arc::clone(&barrier);
                std::thread::spawn(move || {
                    let updated = BTreeSet::from([PathBuf::from(folder)]);
                    barrier.wait();
                    (folder, save(&path, &BTreeSet::new(), &updated))
                })
            })
            .collect();
        let results: Vec<_> = threads
            .into_iter()
            .map(|thread| thread.join().unwrap())
            .collect();
        assert_eq!(
            results.iter().filter(|(_, result)| result.is_ok()).count(),
            1
        );
        let saved = load(&fixture.path()).unwrap();
        for (folder, result) in results {
            assert_eq!(saved.contains(&PathBuf::from(folder)), result.is_ok());
        }
    }
}
