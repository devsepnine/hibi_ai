use std::path::Path;
use anyhow::{Result, Context};
use similar::{ChangeTag, TextDiff};

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
    // Check if source is binary
    if is_binary_file(source).unwrap_or(false) {
        return Ok(format!(
            "Binary file: {}\n\nCannot display diff for binary files.\n\nFile size: {} bytes",
            source.display(),
            std::fs::metadata(source)?.len()
        ));
    }

    let source_content = std::fs::read_to_string(source)
        .with_context(|| format!("Failed to read source file as UTF-8: {}", source.display()))?;

    if !dest.exists() {
        // New file - show all as additions
        let mut output = String::new();
        output.push_str("--- (new file)\n");
        output.push_str(&format!("+++ {}\n", source.display()));
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
        .with_context(|| format!("Failed to read destination file as UTF-8: {}", dest.display()))?;

    if source_content == dest_content {
        // Files are identical - show the content
        let mut output = String::new();
        output.push_str(&format!("=== {} (identical) ===\n\n", source.display()));
        output.push_str(&source_content);
        return Ok(output);
    }

    let diff = TextDiff::from_lines(&dest_content, &source_content);

    let mut output = String::new();
    output.push_str(&format!("--- {}\n", dest.display()));
    output.push_str(&format!("+++ {}\n", source.display()));

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
