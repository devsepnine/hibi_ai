use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Appearance {
    Dark,
    Light,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    #[default]
    #[serde(rename = "en")]
    English,
    #[serde(rename = "ko")]
    Korean,
    #[serde(rename = "ja")]
    Japanese,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct Preferences {
    pub appearance: Option<Appearance>,
    pub language: Option<Language>,
}

fn path() -> Result<PathBuf> {
    Ok(crate::paths::require_home_dir()?.join(".hibi/preferences.json"))
}

pub fn load() -> Result<Preferences> {
    load_from(&path()?)
}

pub fn save(settings: Preferences) -> Result<()> {
    save_to(&path()?, settings)
}

fn load_from(path: &Path) -> Result<Preferences> {
    serde_json::from_value(Value::Object(read_object(path)?)).context("Invalid display preferences")
}

fn read_object(path: &Path) -> Result<Map<String, Value>> {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Map::new()),
        Err(error) => return Err(error).with_context(|| format!("Cannot read {}", path.display())),
    };
    serde_json::from_slice::<Map<String, Value>>(&bytes)
        .with_context(|| format!("Invalid preferences at {}", path.display()))
}

fn save_to(path: &Path, settings: Preferences) -> Result<()> {
    let path = match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => fs::canonicalize(path)?,
        Ok(_) => path.to_path_buf(),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => path.to_path_buf(),
        Err(error) => return Err(error).context("Cannot inspect preferences"),
    };
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    // Theme and language can be edited by different installer windows.
    // Lock a stable sibling because saving replaces the preferences file.
    let lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(path.with_extension("lock"))
        .context("Cannot open display preferences lock")?;
    lock.lock().context("Cannot lock display preferences")?;
    let result = write_settings(&path, settings);
    lock.unlock().context("Cannot unlock display preferences")?;
    result
}

fn write_settings(path: &Path, settings: Preferences) -> Result<()> {
    let mut object = read_object(path)?;
    serde_json::from_value::<Preferences>(Value::Object(object.clone()))
        .context("Invalid display preferences")?;
    if let Some(appearance) = settings.appearance {
        object.insert("appearance".into(), serde_json::to_value(appearance)?);
    }
    if let Some(language) = settings.language {
        object.insert("language".into(), serde_json::to_value(language)?);
    }
    let bytes = serde_json::to_vec_pretty(&object)?;
    let permissions = match fs::metadata(path) {
        Ok(metadata) => {
            if metadata.permissions().readonly() {
                anyhow::bail!("Preferences are read only: {}", path.display());
            }
            Some(metadata.permissions())
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => return Err(error).context("Cannot inspect preference permissions"),
    };
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let serial = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_nanos();
    let temporary = path.with_extension(format!("{}.{stamp}.{serial}.tmp", std::process::id()));
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let file = options
        .open(&temporary)
        .with_context(|| format!("Cannot create {}", temporary.display()))?;
    let pending = PendingPreference(temporary);
    write_staged(file, &bytes, permissions)?;
    fs::rename(&pending.0, path).with_context(|| format!("Cannot save {}", path.display()))
}

fn write_staged(mut file: File, bytes: &[u8], permissions: Option<fs::Permissions>) -> Result<()> {
    file.write_all(bytes)?;
    if let Some(permissions) = permissions {
        file.set_permissions(permissions)?;
    }
    file.sync_all()?;
    Ok(())
}

struct PendingPreference(PathBuf);

impl Drop for PendingPreference {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn appearance(value: Appearance) -> Preferences {
        Preferences {
            appearance: Some(value),
            language: None,
        }
    }

    struct Fixture(PathBuf);

    impl Fixture {
        fn new() -> Self {
            static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
            let serial = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let stamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "hibi_preferences_{}_{stamp}_{serial}",
                std::process::id()
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }

        fn path(&self) -> PathBuf {
            self.0.join("preferences.json")
        }

        fn assert_no_temporary_file(&self) {
            assert!(fs::read_dir(&self.0).unwrap().all(|entry| {
                entry
                    .unwrap()
                    .path()
                    .extension()
                    .is_none_or(|extension| extension != "tmp")
            }));
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            #[cfg(windows)]
            if let Ok(metadata) = fs::metadata(self.path()) {
                let mut permissions = metadata.permissions();
                permissions.set_readonly(false);
                let _ = fs::set_permissions(self.path(), permissions);
            }
            fs::remove_dir_all(&self.0).unwrap();
        }
    }

    #[test]
    fn missing_preferences_have_no_override_and_create_no_file() {
        let fixture = Fixture::new();
        assert_eq!(load_from(&fixture.path()).unwrap().appearance, None);
        assert!(!fixture.path().exists());
        assert_eq!(fs::read_dir(&fixture.0).unwrap().count(), 0);
    }

    #[test]
    fn each_appearance_roundtrips_and_unknown_fields_survive() {
        let fixture = Fixture::new();
        fs::write(fixture.path(), br#"{"future":{"enabled":true},"count":7}"#).unwrap();
        for mode in [Appearance::Dark, Appearance::Light, Appearance::Dark] {
            save_to(&fixture.path(), appearance(mode)).unwrap();
            assert_eq!(load_from(&fixture.path()).unwrap().appearance, Some(mode));
            let value: Value = serde_json::from_slice(&fs::read(fixture.path()).unwrap()).unwrap();
            assert_eq!(value["future"]["enabled"], true);
            assert_eq!(value["count"], 7);
            fixture.assert_no_temporary_file();
        }
    }

    #[test]
    fn language_roundtrips_without_changing_appearance_or_future_fields() {
        let fixture = Fixture::new();
        fs::write(fixture.path(), br#"{"appearance":"dark","future":true}"#).unwrap();
        assert_eq!(load_from(&fixture.path()).unwrap().language, None);
        for language in [Language::Korean, Language::Japanese, Language::English] {
            save_to(
                &fixture.path(),
                Preferences {
                    language: Some(language),
                    appearance: None,
                },
            )
            .unwrap();
            save_to(&fixture.path(), appearance(Appearance::Light)).unwrap();
            assert_eq!(
                load_from(&fixture.path()).unwrap(),
                Preferences {
                    language: Some(language),
                    appearance: Some(Appearance::Light)
                }
            );
            let stored: Value = serde_json::from_slice(&fs::read(fixture.path()).unwrap()).unwrap();
            assert_eq!(stored["future"], true);
        }
    }

    #[test]
    fn malformed_data_is_reported_and_never_overwritten() {
        let fixture = Fixture::new();
        for bytes in [
            b"{invalid".as_slice(),
            b"[]",
            br#"{"appearance":"unknown","other":1}"#,
            br#"{"appearance":false}"#,
            br#"{"language":"unsupported"}"#,
        ] {
            fs::write(fixture.path(), bytes).unwrap();
            assert!(load_from(&fixture.path()).is_err());
            assert!(save_to(&fixture.path(), appearance(Appearance::Dark)).is_err());
            assert_eq!(fs::read(fixture.path()).unwrap(), bytes);
            fixture.assert_no_temporary_file();
        }
    }

    #[test]
    fn read_only_save_preserves_existing_preference_and_unknown_fields() {
        let fixture = Fixture::new();
        let bytes = br#"{"appearance":"light","other":42}"#;
        fs::write(fixture.path(), bytes).unwrap();
        let mut permissions = fs::metadata(fixture.path()).unwrap().permissions();
        permissions.set_readonly(true);
        fs::set_permissions(fixture.path(), permissions).unwrap();
        assert!(save_to(&fixture.path(), appearance(Appearance::Dark)).is_err());
        assert_eq!(fs::read(fixture.path()).unwrap(), bytes);
        assert_eq!(
            load_from(&fixture.path()).unwrap().appearance,
            Some(Appearance::Light)
        );
        fixture.assert_no_temporary_file();
    }

    #[cfg(unix)]
    #[test]
    fn saves_preserve_permissions_and_follow_existing_symlink() {
        use std::os::unix::fs::PermissionsExt;
        let fixture = Fixture::new();
        let target = fixture.0.join("actual.json");
        save_to(&target, appearance(Appearance::Dark)).unwrap();
        assert_eq!(
            fs::metadata(&target).unwrap().permissions().mode() & 0o777,
            0o600
        );
        fs::set_permissions(&target, fs::Permissions::from_mode(0o640)).unwrap();
        std::os::unix::fs::symlink(&target, fixture.path()).unwrap();
        save_to(&fixture.path(), appearance(Appearance::Light)).unwrap();
        assert!(
            fs::symlink_metadata(fixture.path())
                .unwrap()
                .file_type()
                .is_symlink()
        );
        assert_eq!(
            load_from(&target).unwrap().appearance,
            Some(Appearance::Light)
        );
        assert_eq!(
            fs::metadata(target).unwrap().permissions().mode() & 0o777,
            0o640
        );
        fixture.assert_no_temporary_file();
    }
}
