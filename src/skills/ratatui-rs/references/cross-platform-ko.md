# Cross-Platform TUI, Windows / macOS / Linux

대부분의 ratatui 튜토리얼은 POSIX를 가정한다. 프로덕션 TUI는 Git Bash / MSYS / 네이티브 cmd를 실행하는 Windows 사용자에게 출시되며, 가정이 미묘한 방식으로 깨진다.

## Windows 인자와 경로

### 인자 경계 보존

프로그램과 인자를 별도 필드로 받으면 빈 인자, 공백, 작은따옴표, 역슬래시를 그대로 보존할 수 있다. CLI에서는 `std::env::args_os`로 이미 분리된 인자를 받고 설정 파일에서는 인자 배열을 읽는다. 기존 설정이 명령줄 문자열이라면 그 형식의 인용 규칙을 유지한다. Windows의 `CommandLineToArgvW` 규칙을 약속한 필드에는 해당 파서를 사용하고, POSIX 인용을 약속한 필드에는 `shlex`를 사용한다.

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

Git for Windows / MSYS는 `C:\Users\me`를 `/c/Users/me`로 표현한다. 네이티브 Windows API와 Windows의 `std::fs::canonicalize`는 이 형식을 이해하지 못한다.

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

### 네이티브 실행 파일과 셸 스크립트

실행 파일 경로를 `Command::new`에 전달하고 각 데이터 인자를 `Command::arg` 또는 `Command::args`에 전달한다. 일반적인 인자 규칙을 따르는 네이티브 도구는 `Budget 100%!;final.txt`와 같은 파일명을 하나의 값으로 받는다. 옵션과 경로에는 해당 도구의 규칙을 적용한다.

셸과 Windows 배치 파일은 명령문을 해석한다. 배치 래퍼를 제공하는 도구에 임의의 파일명을 전달할 때는 네이티브 진입점을 사용하거나, 인터프리터에 신뢰할 수 있는 스크립트 경로와 각 인자를 순서대로 전달한다. 예를 들어 Windows에서 npm을 호출할 때 프로젝트가 지정한 Node 실행 파일의 첫 인자로 설치된 `npm-cli.js` 경로를 전달할 수 있다.

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

- **Ctrl+C**: Windows에서 crossterm은 이를 `KeyModifiers::CONTROL`이 있는 `KeyCode::Char('c')`로 보고한다. SIGINT 핸들러를 같이 설치하려고 하지 마라, OS가 콘솔 앱에 SIGINT를 동일한 방식으로 전달하지 않는다.
- **Function key / Alt 조합**: 터미널 기능은 매우 다양하다, Windows Terminal vs ConHost vs MinTTY. 주요 액션에 F-키에 의존하지 말고; Ctrl-letter fallback을 제공한다.

## Testing Cross-Platform Code

OS-specific 경로에 `#[cfg(target_os = "...")]` 테스트를 사용한다:

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

- [ ] 네이티브 프로세스 호출이 빈 인자와 파일명에 포함된 문장부호를 보존한다.
- [ ] 호출한 셸이나 배치 인터프리터 중 어디에서 인자를 해석하는지 확인한다.
- [ ] 입력 형식에서 MSYS 경로를 지정한 경우에만 변환한다.
- [ ] Windows를 지원한다면 Windows에서 실제 대상 프로그램이 받은 인자를 검사한다.
