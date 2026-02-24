# Inject Guide Hook

사용자 프롬프트의 키워드를 분석하여 관련 agent 가이드를 자동으로 주입하는 Rust 기반 크로스 플랫폼 hook입니다.

## 기능

### 자동 가이드 주입

사용자가 프롬프트를 제출할 때 키워드를 분석하여 관련된 agent 파일을 자동으로 컨텍스트에 추가합니다.

**동작 방식:**
1. 사용자 프롬프트 분석
2. `~/.claude/agents/` 디렉토리의 모든 agent 파일 스캔
3. Frontmatter의 keywords와 프롬프트 매칭
4. 매칭된 agent 가이드를 자동 주입

## 사용 방법

### Agent 파일 작성

Agent 파일은 YAML frontmatter와 마크다운 내용으로 구성됩니다:

```markdown
---
keywords:
  - git
  - commit
  - pull request
---

# Git Workflow Guide

## Commit Convention
...
```

### 키워드 매칭

프롬프트에 키워드가 포함되면 해당 agent가 자동으로 주입됩니다:

**예시:**
- 프롬프트: "help me create a pull request"
- 매칭: `keywords: ["git", "pull request"]`
- 결과: Git workflow agent 자동 주입

### 파일 구조

```
~/.claude/
├── agents/
│   ├── git-workflow.md
│   ├── testing/
│   │   └── unit-test-guide.md
│   └── architecture/
│       └── design-patterns.md
└── hooks/
    └── inject_guide/
        ├── inject_guide_macos
        ├── inject_guide.exe
        └── inject_guide_linux
```

## Hook 설정

**Event:** `UserPromptSubmit`
**Type:** `command`
**Timeout:** 10000ms

프롬프트 제출 시점에 실행되어 관련 agent를 찾아 주입합니다.

## 지원 플랫폼

- ✅ **macOS** (Intel & Apple Silicon)
- ✅ **Windows** (x64)
- ✅ **Linux** (x64)

## 로그

디버깅을 위해 실행 로그가 기록됩니다:

```
~/.claude/hooks/inject_guide/inject-guide.log
```

로그 내용:
- 매칭된 agent 파일명
- 키워드 매칭 결과
- 에러 메시지

## 빌드

### 전체 빌드

```bash
cd tools/hooks/inject_guide
./build.sh
```

### 플랫폼별 빌드

```bash
# macOS
cargo build --release

# Windows (from macOS)
cargo build --release --target x86_64-pc-windows-gnu

# Linux (from macOS)
cargo build --release --target x86_64-unknown-linux-musl
```

## 사전 요구사항

### macOS에서 크로스 컴파일

```bash
# Rust 설치
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Windows 타겟
rustup target add x86_64-pc-windows-gnu
brew install mingw-w64

# Linux 타겟
rustup target add x86_64-unknown-linux-musl
brew install filosottile/musl-cross/musl-cross
```

## 작동 원리

1. **Agent 스캔**: `~/.claude/agents/` 하위의 모든 `.md` 파일 탐색
2. **Frontmatter 파싱**: YAML frontmatter에서 keywords 추출
3. **키워드 매칭**: 프롬프트와 키워드를 정규식으로 매칭 (대소문자 무시)
4. **가이드 주입**: 매칭된 agent의 내용을 컨텍스트에 추가

### 매칭 알고리즘

```rust
// 정규식 기반 키워드 매칭
let pattern = format!(r"(?i){}", regex::escape(keyword));
if re.is_match(&prompt_lower) {
    // Agent 주입
}
```

## 최적화

- **중복 제거**: 하나의 agent가 여러 키워드로 매칭되어도 한 번만 주입
- **하위 디렉토리 지원**: agents/ 하위의 모든 폴더 구조 탐색
- **빠른 파싱**: 효율적인 frontmatter 파싱

## 예시

### Agent 파일: `~/.claude/agents/testing/jest-guide.md`

```markdown
---
keywords:
  - test
  - jest
  - unit test
  - testing
---

# Jest Testing Guide

## Setup
...
```

### 사용자 프롬프트

```
"help me write unit tests for this component"
```

### 결과

Jest testing guide가 자동으로 컨텍스트에 포함되어 Claude가 Jest 관련 가이드를 참고하여 응답합니다.

## 라이센스

MIT
