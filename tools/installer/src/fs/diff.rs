use std::path::Path;
use anyhow::{Result, Context};
use similar::{ChangeTag, TextDiff};

/// Normalize path display to remove Windows extended-length prefix
fn normalize_path_display(path: &Path) -> String {
    let path_str = path.display().to_string();

    // Windows: Remove \\?\ prefix for cleaner display
    #[cfg(windows)]
    {
        if path_str.starts_with(r"\\?\") {
            return path_str[4..].to_string();
        }
    }

    path_str
}

/// Check if a file is likely binary by reading first few bytes
fn is_binary_file(path: &Path) -> Result<bool> {
    use std::io::Read;
    let mut file = std::fs::File::open(path)?;
    let mut buffer = [0u8; 512];
    let bytes_read = file.read(&mut buffer)?;

    // Check for null bytes or high ratio of non-printable characters
    let null_count = buffer[..bytes_read].iter().filter(|&&b| b == 0).count();
    let non_printable = buffer[..bytes_read].iter()
        .filter(|&&b| b < 32 && b != 9 && b != 10 && b != 13)
        .count();

    Ok(null_count > 0 || non_printable > bytes_read / 4)
}

pub fn compare_files(source: &Path, dest: &Path) -> Result<String> {
    // External-file case: scanner sets source_path == dest_path for files
    // present in dest_dir without a matching source. There's no diff to
    // compute -- show the file as-is so the user can inspect before
    // deciding to remove.
    if source == dest {
        if is_binary_file(source).unwrap_or(false) {
            return Ok(format!(
                "Binary file: {}\n\nCannot display diff for binary files.\n\nFile size: {} bytes",
                source.display(),
                std::fs::metadata(source)?.len()
            ));
        }
        let content = std::fs::read_to_string(source)
            .with_context(|| format!("Failed to read external file as UTF-8: {}", source.display()))?
            .replace("\r\n", "\n");
        let mut output = String::new();
        output.push_str(&format!("=== {} (external file -- no source) ===\n\n", normalize_path_display(source)));
        output.push_str(&content);
        return Ok(output);
    }

    // Check if source is binary
    if is_binary_file(source).unwrap_or(false) {
        return Ok(format!(
            "Binary file: {}\n\nCannot display diff for binary files.\n\nFile size: {} bytes",
            source.display(),
            std::fs::metadata(source)?.len()
        ));
    }

    let source_content = std::fs::read_to_string(source)
        .with_context(|| format!("Failed to read source file as UTF-8: {}", source.display()))?
        .replace("\r\n", "\n");

    if !dest.exists() {
        // New file - show all as additions
        let mut output = String::new();
        output.push_str("--- (new file)\n");
        output.push_str(&format!("+++ {}\n", normalize_path_display(source)));
        output.push_str("@@ new file @@\n");
        for line in source_content.lines() {
            output.push_str(&format!("+{}\n", line));
        }
        return Ok(output);
    }

    // Check if destination is binary
    if is_binary_file(dest).unwrap_or(false) {
        return Ok(format!(
            "Binary file: {}\n\nCannot display diff for binary files.",
            dest.display()
        ));
    }

    let dest_content = std::fs::read_to_string(dest)
        .with_context(|| format!("Failed to read destination file as UTF-8: {}", dest.display()))?
        .replace("\r\n", "\n");

    if source_content == dest_content {
        // Files are identical - show the content
        let mut output = String::new();
        output.push_str(&format!("=== {} (identical) ===\n\n", source.display()));
        output.push_str(&source_content);
        return Ok(output);
    }

    let diff = TextDiff::from_lines(&dest_content, &source_content);

    let mut output = String::new();
    output.push_str(&format!("--- {}\n", normalize_path_display(dest)));
    output.push_str(&format!("+++ {}\n", normalize_path_display(source)));

    for change in diff.iter_all_changes() {
        let sign = match change.tag() {
            ChangeTag::Delete => "-",
            ChangeTag::Insert => "+",
            ChangeTag::Equal => " ",
        };
        output.push_str(sign);
        output.push_str(change.value());
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_test_file(label: &str, contents: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let dir = std::env::temp_dir().join(format!("hibi_diff_{label}_{nanos}"));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("file.md");
        std::fs::write(&path, contents).unwrap();
        path
    }

    #[test]
    fn external_file_uses_single_view_when_source_equals_dest() {
        // When the scanner reports an External component, source_path and
        // dest_path are the same -- there's no diff to compute. The view
        // should show a header marking it external and the file contents
        // verbatim, not a self-diff and not the "(identical)" marker used
        // for source/dest pairs that happen to match byte-for-byte.
        let path = unique_test_file("external", "# external content\nline 2\n");

        let out = compare_files(&path, &path).unwrap();

        assert!(
            out.contains("(external file -- no source)"),
            "expected external marker, got:\n{out}"
        );
        assert!(out.contains("# external content"), "file body must appear:\n{out}");
        assert!(
            !out.contains("(identical)"),
            "must not use the source==dest identical marker for externals:\n{out}"
        );
        // No unified-diff prefixes on the body lines.
        assert!(!out.lines().any(|l| l.starts_with("+++ ") || l.starts_with("--- ")));

        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }
}
