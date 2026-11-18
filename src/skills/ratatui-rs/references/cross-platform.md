# Cross-Platform TUI, Windows / macOS / Linux

Most ratatui tutorials assume POSIX. Production TUIs ship to Windows users running Git Bash / MSYS / native cmd, and the assumptions break in subtle ways.

## Arguments and paths on Windows

### Preserve argument boundaries

Accept the executable and its arguments as separate fields. This preserves empty arguments, spaces, apostrophes, and backslashes. Read existing CLI arguments with `std::env::args_os` and use an argument array in new configuration. For existing command strings, use the parser specified by the field’s contract, such as `CommandLineToArgvW` for Windows command line syntax or `shlex` for POSIX quoting.

```rust
use std::ffi::{OsStr, OsString};
use std::process::Command;

pub fn build_command(program: &OsStr, args: &[OsString]) -> Command {
    let mut command = Command::new(program);
    command.args(args);
    command
}
```

### MSYS Paths, `/c/Users/...`, Are Not Real Windows Paths

Git for Windows / MSYS represents `C:\Users\me` as `/c/Users/me`. Native Windows APIs, and `std::fs::canonicalize` on Windows, don't understand this form.

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

### Native executables and shell scripts

Pass the executable path to `Command::new` and each data argument to `Command::arg` or `Command::args`. A native tool that follows normal argument parsing receives filenames such as `Budget 100%!;final.txt` as one value. Apply the tool’s own rules for options and paths.

Shells and Windows batch files interpret command text. If a tool ships a batch wrapper and accepts arbitrary filenames, choose its native entry point or invoke its interpreter with the trusted script path followed by separate arguments. For npm on Windows, this can be the configured Node executable with the installed `npm-cli.js` path as its first argument.

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

- **Ctrl+C**: on Windows, crossterm reports it as a `KeyCode::Char('c')` with `KeyModifiers::CONTROL`. Do not also try to install a SIGINT handler, the OS doesn't deliver SIGINT to console apps the same way.
- **Function keys / Alt combos**: terminal capabilities vary widely, Windows Terminal vs ConHost vs MinTTY. Avoid relying on F-keys for primary actions; offer a Ctrl-letter fallback.

## Testing Cross-Platform Code

Use `#[cfg(target_os = "...")]` tests for OS-specific paths:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_argument_boundaries() {
        let args = ["", "O'Brien", "Budget 100%!;final.txt", r"C:\Program Files\Demo"].map(OsString::from);
        let cmd = build_command(OsStr::new("tool"), &args);
        assert!(cmd.get_args().eq(args.iter().map(OsString::as_os_str)));
    }

    #[cfg(windows)]
    #[test]
    fn normalizes_msys_path() {
        assert_eq!(normalize_git_path("/c/Users/me"), r"C:\Users\me");
    }
}
```

## Summary Checklist

- [ ] Native process calls preserve empty arguments and filename punctuation.
- [ ] The command identifies any shell or batch interpreter that will parse its arguments.
- [ ] Convert MSYS notation only for inputs whose contract specifies an MSYS path.
- [ ] When Windows is supported, run the argument round trip with the actual target program on Windows.
