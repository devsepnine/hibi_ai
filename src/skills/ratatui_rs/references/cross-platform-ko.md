# Cross-Platform TUI (Windows / macOS / Linux)

대부분의 ratatui 튜토리얼은 POSIX를 가정한다. 프로덕션 TUI는 Git Bash / MSYS / 네이티브 cmd를 실행하는 Windows 사용자에게 출시되며, 가정이 미묘한 방식으로 깨진다.

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

Git for Windows / MSYS는 `C:\Users\me`를 `/c/Users/me`로 표현한다. 네이티브 Windows API (그리고 Windows의 `std::fs::canonicalize`)는 이 형식을 이해하지 못한다.

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

Unix-only blocklist는 `%`, `^`, `!` (cmd.exe)를 놓치고, Windows-only blocklist는 `;`, `&`, `|`, `<`, `>`, `` ` ``를 놓친다.

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

크로스 플랫폼 도구 호출이 필요하면 추상화한다:

```rust
fn npm_command() -> Command {
    if cfg!(windows) { Command::new("npm.cmd") } else { Command::new("npm") }
}
```

## Path Display in the TUI

forward-slash 중심으로 빌드된 UI에서 `C:\Users\me\file.txt`를 보여주는 것은 어색해 보일 수 있다. 한 번 결정한다: POSIX 스타일 또는 네이티브로 표시할 것인가? 그런 다음 코드 전반이 아닌 표시 경계에서 변환한다.

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

- **Ctrl+C**: Windows에서 crossterm은 이를 `KeyModifiers::CONTROL`이 있는 `KeyCode::Char('c')`로 보고한다. SIGINT 핸들러를 같이 설치하려고 하지 마라 — OS가 콘솔 앱에 SIGINT를 동일한 방식으로 전달하지 않는다.
- **Function key / Alt 조합**: 터미널 기능은 매우 다양하다 (Windows Terminal vs ConHost vs MinTTY). 주요 액션에 F-키에 의존하지 말고; Ctrl-letter fallback을 제공한다.

## Testing Cross-Platform Code

OS-specific 경로에 `#[cfg(target_os = "...")]` 테스트를 사용한다:

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

- [ ] 어디에도 `cmd /c` 없음
- [ ] Windows에 cfg-gate된 `split_command`
- [ ] 모든 외부 입력 경계 (git remote, env var)에서 MSYS 경로 정규화
- [ ] `is_safe_command`가 `&|;<>$`와 `%^!` 둘 다 차단
- [ ] 네이티브 실행 파일 해석 (Windows의 `npm.cmd`, POSIX의 `npm`)
- [ ] CI에서 적어도 하나의 Windows 테스트 (GitHub Actions: `runs-on: windows-latest`)
