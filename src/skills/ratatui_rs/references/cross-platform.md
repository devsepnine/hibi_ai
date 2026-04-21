# Cross-Platform TUI (Windows / macOS / Linux)

Most ratatui tutorials assume POSIX. Production TUIs ship to Windows users running Git Bash / MSYS / native cmd, and the assumptions break in subtle ways.

## The Three Things That Always Break on Windows

### 1. Backslashes Are NOT Escape Characters

```rust
// shlex::split(r"C:\Users\me") on Windows returns ["C:Usersme"] — backslashes eaten as escapes.
// You MUST split with Windows semantics on Windows.

#[cfg(windows)]
pub fn split_command(cmd: &str) -> Option<Vec<String>> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;

    for ch in cmd.chars() {
        match ch {
            '"' | '\'' if quote == Some(ch) => quote = None,
            '"' | '\'' if quote.is_none() => quote = Some(ch),
            ' ' | '\t' if quote.is_none() => {
                if !current.is_empty() {
                    args.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(ch),
        }
    }
    if !current.is_empty() { args.push(current); }
    Some(args)
}

#[cfg(not(windows))]
pub fn split_command(cmd: &str) -> Option<Vec<String>> {
    shlex::split(cmd)
}
```

### 2. MSYS Paths (`/c/Users/...`) Are Not Real Windows Paths

Git for Windows / MSYS represents `C:\Users\me` as `/c/Users/me`. Native Windows APIs (and `std::fs::canonicalize` on Windows) don't understand this form.

```rust
/// Convert `/c/Users/me` → `C:\Users\me` on Windows. On other platforms returns input unchanged.
#[cfg(windows)]
pub fn normalize_git_path(p: &str) -> String {
    // Match `/<single-letter>/...`
    let bytes = p.as_bytes();
    if bytes.len() >= 3 && bytes[0] == b'/' && bytes[2] == b'/' && bytes[1].is_ascii_alphabetic() {
        let drive = (bytes[1] as char).to_ascii_uppercase();
        return format!("{drive}:{}", &p[2..].replace('/', "\\"));
    }
    p.to_string()
}

#[cfg(not(windows))]
pub fn normalize_git_path(p: &str) -> String { p.to_string() }
```

### 3. Shell Metacharacters Differ

Unix-only blocklists miss `%`, `^`, `!` (cmd.exe), and Windows-only blocklists miss `;`, `&`, `|`, `<`, `>`, `` ` ``.

```rust
pub fn is_safe_command(cmd: &str) -> bool {
    // Block both Unix and Windows shell metacharacters so a single function
    // works regardless of which shell the user has configured.
    let unsafe_chars = [
        // POSIX shells
        '&', '|', ';', '<', '>', '$', '`', '\n', '\r',
        // cmd.exe
        '%', '^', '!',
    ];
    !cmd.chars().any(|c| unsafe_chars.contains(&c))
}
```

## The Anti-Pattern That Burns Everyone: `cmd /c`

```rust
// ❌ NEVER DO THIS on Windows. Enables shell injection.
Command::new("cmd").args(["/c", &user_input]).spawn();

// ✅ Invoke the executable directly. Windows resolves .cmd / .exe via PATHEXT.
Command::new("npm.cmd").arg("install").spawn();   // Windows
Command::new("npm").arg("install").spawn();        // POSIX
```

If you need cross-platform tool invocation, abstract it:

```rust
fn npm_command() -> Command {
    if cfg!(windows) { Command::new("npm.cmd") } else { Command::new("npm") }
}
```

## Path Display in the TUI

Showing `C:\Users\me\file.txt` in a UI built around forward-slashes can look out of place. Decide once: do you display POSIX-style or native? Then convert at the display boundary, not throughout the code.

```rust
fn to_display_path(p: &Path) -> String {
    let s = p.display().to_string();
    if cfg!(windows) {
        s.replace('\\', "/")  // display POSIX-style
    } else {
        s
    }
}
```

## Key Codes That Differ

- **Ctrl+C**: on Windows, crossterm reports it as a `KeyCode::Char('c')` with `KeyModifiers::CONTROL`. Do not also try to install a SIGINT handler — the OS doesn't deliver SIGINT to console apps the same way.
- **Function keys / Alt combos**: terminal capabilities vary widely (Windows Terminal vs ConHost vs MinTTY). Avoid relying on F-keys for primary actions; offer a Ctrl-letter fallback.

## Testing Cross-Platform Code

Use `#[cfg(target_os = "...")]` tests for OS-specific paths:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(windows)]
    #[test]
    fn windows_split_keeps_backslashes() {
        let args = split_command(r#"foo "C:\path\to\bin" --flag"#).unwrap();
        assert_eq!(args, vec!["foo", r"C:\path\to\bin", "--flag"]);
    }

    #[cfg(windows)]
    #[test]
    fn normalizes_msys_path() {
        assert_eq!(normalize_git_path("/c/Users/me"), r"C:\Users\me");
    }

    #[test]
    fn rejects_shell_metachars() {
        assert!(!is_safe_command("ls; rm -rf /"));
        assert!(!is_safe_command("echo %SECRET%"));
        assert!(is_safe_command("cargo build --release"));
    }
}
```

## Summary Checklist

- [ ] No `cmd /c` anywhere
- [ ] `split_command` cfg-gated for Windows
- [ ] MSYS path normalization at every external-input boundary (git remotes, env vars)
- [ ] `is_safe_command` blocks both `&|;<>$` and `%^!`
- [ ] Native executable resolution (`npm.cmd` on Windows, `npm` on POSIX)
- [ ] At least one Windows test in CI (GitHub Actions: `runs-on: windows-latest`)
