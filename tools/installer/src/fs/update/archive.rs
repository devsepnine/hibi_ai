use anyhow::{Context, Result, ensure};
use std::{
    fs,
    io::{self, Cursor, Read},
    path::{Component, Path, PathBuf},
};

const LIMIT: u64 = 512 << 20;

fn relative(name: &Path, prefix: &str) -> Result<PathBuf> {
    ensure!(
        name.components()
            .all(|part| matches!(part, Component::Normal(_))),
        "Invalid archive path"
    );
    let rel = name
        .strip_prefix(prefix)
        .context("Unexpected archive root")?;
    // Windows also interprets alternate stream names and trailing dots specially.
    for part in rel.iter() {
        let name = part.to_str().context("Invalid archive filename")?;
        let stem = name
            .split('.')
            .next()
            .unwrap_or("")
            .trim_end()
            .to_ascii_uppercase();
        let reserved = matches!(
            stem.as_str(),
            "CON" | "PRN" | "AUX" | "NUL" | "CONIN$" | "CONOUT$"
        ) || ["COM", "LPT"].iter().any(|prefix| {
            stem.strip_prefix(prefix).is_some_and(|suffix| {
                matches!(
                    suffix,
                    "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "¹" | "²" | "³"
                )
            })
        });
        ensure!(
            !reserved && !name.contains([':', '\\']) && !name.ends_with([' ', '.']),
            "Invalid archive filename"
        );
    }
    Ok(rel.into())
}

fn write_entry(
    root: &Path,
    relative: PathBuf,
    directory: bool,
    size: u64,
    reader: impl Read,
    left: &mut u64,
    mode: u32,
) -> Result<()> {
    let path = root.join(relative);
    if directory {
        ensure!(size == 0, "Unexpected directory payload");
        fs::create_dir_all(path)?;
        return Ok(());
    }
    *left = left
        .checked_sub(size)
        .context("Expanded archive exceeds size limit")?;
    fs::create_dir_all(path.parent().context("Invalid archive entry")?)?;
    let mut file = fs::File::create_new(&path)?;
    let count = io::copy(&mut reader.take(size + 1), &mut file)?;
    ensure!(count == size, "Invalid archive entry size");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        file.set_permissions(fs::Permissions::from_mode(0o644 | (mode & 0o111)))?;
    }
    #[cfg(not(unix))]
    let _ = mode;
    Ok(())
}

pub(super) fn extract(bytes: &[u8], prefix: &str, out: &Path, zip: bool) -> Result<()> {
    let mut left = LIMIT;
    if zip {
        let mut archive = zip::ZipArchive::new(Cursor::new(bytes))?;
        ensure!(archive.len() <= 16000, "Too many archive entries");
        for index in 0..archive.len() {
            let entry = archive.by_index(index)?;
            let path = relative(Path::new(entry.name()), prefix)?;
            let mode = entry.unix_mode().unwrap_or(0);
            ensure!(
                matches!(mode & 0o170000, 0 | 0o040000 | 0o100000),
                "Unsupported ZIP entry"
            );
            write_entry(
                out,
                path,
                entry.is_dir(),
                entry.size(),
                entry,
                &mut left,
                mode,
            )?;
        }
    } else {
        let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(bytes));
        for (index, entry) in archive.entries()?.enumerate() {
            ensure!(index < 16000, "Too many archive entries");
            let entry = entry?;
            let kind = entry.header().entry_type();
            ensure!(kind.is_dir() || kind.is_file(), "Unsupported tar entry");
            let path = relative(&entry.path()?, prefix)?;
            let mode = entry.header().mode()?;
            write_entry(
                out,
                path,
                kind.is_dir(),
                entry.size(),
                entry,
                &mut left,
                mode,
            )?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_traversal_links_and_nonportable_names() {
        for name in [
            "elsewhere/file",
            "../outside",
            "bundle/../target",
            "bundle/alt:stream",
            "bundle/NUL.txt",
            "bundle/trailing.",
            "/bundle/file",
        ] {
            assert!(relative(Path::new(name), "bundle").is_err(), "{name}");
        }
        let mut builder = tar::Builder::new(flate2::write::GzEncoder::new(
            Vec::new(),
            flate2::Compression::fast(),
        ));
        let mut header = tar::Header::new_gnu();
        header.set_entry_type(tar::EntryType::Symlink);
        header.set_size(0);
        builder
            .append_link(&mut header, "bundle/link", "elsewhere")
            .unwrap();
        let bytes = builder.into_inner().unwrap().finish().unwrap();
        let out = tempfile::tempdir().unwrap();
        assert!(extract(&bytes, "bundle", out.path(), false).is_err());
        assert_eq!(out.path().read_dir().unwrap().count(), 0);
    }

    #[test]
    fn extracts_windows_zip_and_rejects_duplicates() {
        use std::io::Write;
        for duplicate in [false, true] {
            let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
            zip.start_file("bundle/hibi.exe", zip::write::SimpleFileOptions::default())
                .unwrap();
            zip.write_all(b"fixture").unwrap();
            if duplicate {
                zip.start_file(
                    "bundle/dir/../hibi.exe",
                    zip::write::SimpleFileOptions::default(),
                )
                .unwrap();
            }
            let bytes = zip.finish().unwrap().into_inner();
            let out = tempfile::tempdir().unwrap();
            assert_eq!(
                extract(&bytes, "bundle", out.path(), true).is_err(),
                duplicate
            );
            assert_eq!(fs::read(out.path().join("hibi.exe")).unwrap(), b"fixture");
        }
    }
}
