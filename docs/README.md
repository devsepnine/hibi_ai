# hibi-ai 프로젝트 문서

> 마지막 업데이트: 2026-06-19

## 개요

hibi-ai는 Claude Code와 Codex CLI를 위한 TUI(터미널 사용자 인터페이스) 인스톨러입니다.
AI 에이전트, 커스텀 명령어, 스킬, 훅, MCP 서버, 플러그인 등 다양한 컴포넌트를 대화형 인터페이스로 쉽게 설치하고 관리할 수 있습니다.

## 프로젝트 구조

```
hibi_ai/
├── src/                 # 소스 디렉토리 (Git 관리)
│   ├── agents/          # AI 에이전트 정의 (architect, tdd-guide 등)
│   ├── commands/        # 슬래시 커맨드 (/code-review, /e2e 등)
│   ├── contexts/        # 컨텍스트 프리셋 (개발/리서치/리뷰)
│   ├── hooks/           # 라이프사이클 훅 (전부 deprecated — 인스톨러가 기존 설치본에서 자동 제거)
│   ├── mcps/            # MCP 서버 설정
│   ├── output-styles/   # 출력 스타일 정의
│   ├── plugins/         # 플러그인 설정
│   ├── rules/           # (비어 있음 — 정책은 skills로 이동)
│   ├── skills/          # 도메인별 스킬
│   │   ├── coding-standards/         # 코딩 표준 (code-thresholds, patterns, review-checklist 포함)
│   │   ├── composition-patterns/     # React 컴포지션 패턴
│   │   ├── ratatui_rs/              # Ratatui TUI 개발
│   │   ├── react-native-skills/     # React Native 모바일 개발
│   │   ├── rust-best-practices/     # Rust 베스트 프랙티스
│   │   ├── vercel-react-best-practices/ # Vercel React 성능 최적화
│   │   └── web-design-guidelines/   # 웹 디자인 가이드라인
│   ├── statusline/      # 상태 표시줄 바이너리
│   ├── AGENTS.md        # 에이전트 가이드
│   ├── CLAUDE.md        # Claude 설정
│   ├── mcp.md           # MCP 문서
│   └── settings.json    # 전역 설정
├── dist/                # 빌드 산출물 (gitignore)
│   ├── hibi             # macOS Universal Binary
│   ├── hibi-linux       # Linux 바이너리
│   └── hibi.exe         # Windows 바이너리
├── docs/                # 프로젝트 문서
│   ├── README.md        # 메인 문서
│   ├── RUNBOOK.md       # 운영 가이드
│   └── INDEX.md         # 문서 인덱스
├── tools/               # 개발 도구
│   ├── hooks/           # 훅 소스 코드
│   ├── installer/       # TUI 인스톨러 소스 (Rust, 3,288 LOC)
│   │   └── src/
│   │       ├── app/           # 앱 상태 모듈 (7 파일)
│   │       ├── fs/installer/  # 설치 로직 모듈 (5 파일)
│   │       ├── fs/scanner/    # 스캔 로직 모듈 (2 파일)
│   │       └── main.rs        # 이벤트 루프
│   └── statusline/      # 상태 표시줄 소스 (Rust)
└── release/             # 릴리즈 아티팩트
    ├── v0.1.3/          # 버전별 릴리즈 패키지
    └── v0.1.4/          # 최신 릴리즈
```

## 주요 컴포넌트

### 에이전트 (Agents)

전문화된 AI 에이전트들:

- **architect**: 시스템 설계 및 아키텍처 결정
- **build-error-resolver**: 빌드 에러 해결
- **code-reviewer**: 코드 품질 검토 (security-review 스킬로 보안 검토 포함)
- **doc-updater**: 문서 업데이트
- **e2e-runner**: E2E 테스트 실행 (Playwright)
- **refactor-cleaner**: 데드 코드 정리
- **tdd-guide**: 테스트 주도 개발 가이드

### 명령어 (Commands)

사용 가능한 슬래시 명령어:

- `/plan`: 구현 계획 수립
- `/code-review`: 코드 리뷰 실행
- `/tdd`: TDD 워크플로우 강제
- `/e2e`: E2E 테스트 생성 및 실행
- `/build-fix`: 빌드 에러 수정
- `/update-docs`: 문서 업데이트
- `/refactor-clean`: 리팩토링 및 정리
- `/deps`: 의존성·결합도 감사
- `/checkpoint`: 체크포인트 생성
- `/verify`: 검증 실행
- `/commit`: 커밋 생성

### 훅 (Hooks)

활성 훅은 없다. 과거 라이프사이클 훅(`inject_guide`, `load-context`, `persist-session`, `preserve-context`, `suggest-compact`)은 전부 **deprecated**로 전환되어, 인스톨러가 기존 설치본에서 자동 제거한다. 키워드/컨텍스트 주입은 네이티브 Skill 시스템이 대체한다.

### 스킬 (Skills)

도메인별 전문 지식:

- **composition-patterns**: React 컴포지션 패턴 (React 19 호환)
- **dependency-design**: 의존성·결합도 설계 (Cynefin·공생성·DDD·Turbo 모노레포)
- **ratatui_rs**: Rust 터미널 UI 개발
- **react-native-skills**: React Native 모바일 앱 개발
- **rust-best-practices**: Rust 소유권, 에러 처리, 비동기 패턴
- **vercel-react-best-practices**: Vercel 엔지니어링 성능 가이드
- **web-design-guidelines**: 웹 인터페이스 가이드라인

### 규칙 (Rules)

`rules/` 디렉토리는 비어 있습니다. 상세 정책은 모두 Skills로 이관되었습니다:

- **commit-rules**: 커밋 메시지 규칙
- **pull-request**: PR 생성 가이드라인
- **security-review**: 보안 체크리스트
- **tdd-workflow**: 테스트 주도 개발 워크플로우
- **coding-standards**: 코딩 표준 (code-thresholds / patterns / review-checklist는 `coding-standards/references/`에 위치)

## 빌드 및 릴리즈

### 빌드 시스템

**Installer 빌드** (dist/로 출력):
```bash
cd tools/installer
./build.sh
```

**Hooks & Statusline 빌드** (src/로 출력):
```bash
# Statusline
cd tools/statusline
./build.sh
```

생성되는 바이너리:
- **macOS**: Universal Binary (Intel + Apple Silicon)
- **Linux**: x86_64 (musl static)
- **Windows**: x86_64 (mingw-w64)

### 릴리즈 패키징

```bash
./package.sh
```

- 버전별 디렉토리 구조: `release/v{VERSION}/`
- 모든 설정 파일 포함 (agents, commands, skills, hooks, mcps, plugins, rules)
- 자동 SHA256 체크섬 생성 (`checksums.txt`)

## 설치 방법

### Homebrew (macOS/Linux)

```bash
brew tap devsepnine/brew
brew install hibi
```

### Scoop (Windows)

```bash
scoop bucket add hibi-ai https://github.com/devsepnine/scoop-bucket
scoop install hibi-ai
```

### 수동 설치

1. [Releases](https://github.com/devsepnine/hibi_ai/releases/latest)에서 플랫폼별 파일 다운로드
2. 압축 해제 후 실행:

```bash
# macOS/Linux
tar xzf hibi-ai-*-macos.tar.gz
./hibi

# Windows
# zip 파일 압축 해제 후:
hibi.exe
```

## 사용법

```bash
hibi
```

TUI 인터페이스에서:
1. 대상 CLI 선택 (Claude Code 또는 Codex)
2. 설치할 컴포넌트 선택
3. 변경 사항 검토
4. 설치 실행

## 멀티소스 지원

기본적으로 hibi는 릴리스 패키지에 번들된 설정을 사용합니다. `~/.hibi/sources.yaml` 파일을 통해 추가 소스(git 레포 또는 로컬 디렉터리)를 설정할 수 있습니다.

### 설정 파일

```yaml
# ~/.hibi/sources.yaml
sources:
  # Git 소스: 원격 레포지토리에서 pull
  - type: git
    url: "https://github.com/your-org/shared-configs.git"
    branch: main

  # 로컬 소스: 로컬 디렉터리 사용
  - type: local
    path: "~/dotfiles/claude-configs"

# git 소스 자동 업데이트 비활성화 (기본: true)
auto_update: true
```

### 우선순위

리스트 순서대로 적용되며, 마지막 항목이 최우선(last wins):

```
bundled (최저) → sources.yaml 첫 번째 → ... → sources.yaml 마지막 (최고)
```

같은 이름의 파일이 여러 소스에 존재하면 마지막 소스의 파일이 사용됩니다.

### CLI 명령

```bash
# TUI 실행 (소스 자동 resolve + 스캔)
hibi

# git 소스만 업데이트 (TUI 없이)
hibi --update
```

### 오프라인 동작

| 상황 | 동작 |
|------|------|
| git 미설치 | 경고 + 해당 소스 skip |
| 네트워크 실패 + 캐시 있음 | 경고 + stale 캐시 사용 |
| 네트워크 실패 + 캐시 없음 | 경고 + 해당 소스 skip |
| bundled | 항상 동작 (오프라인 보장) |

### 소스 디렉터리 요구사항

각 소스 디렉터리에는 다음 중 하나 이상이 존재해야 합니다:
`agents/`, `commands/`, `rules/`, `skills/`, `mcps/mcps.yaml`

### TUI 표시

멀티소스가 활성화되면 각 항목 옆에 소스 태그가 표시됩니다:

```
[x] my-agent.md      (new)      [bundled]
[x] custom-agent.md  (new)      [~/dotfiles/claude-configs]
```

단일 소스(bundled만)일 때는 태그가 생략됩니다.

### 보안

- git URL: **HTTPS만** 허용, credentials(`@`) 포함 불가
- 로컬 경로: `..` path traversal 금지, `~/.claude/` 내부 경로 금지 (symlink 해제 후 검증)
- git 캐시 위치: `~/.hibi/cache/<sanitized_url>/`

## 최근 변경사항

### 2026-06-19

**추가됨:**
- `dependency-design` 스킬: 바이브코딩 의존성 관리 방법론 (복잡성/Cynefin, 결합 모델·공생성, 추상화, AI 오너십, Turbo 모노레포). references/ + rules/ + evals/ + 컴파일된 AGENTS.md
- `/deps` 커맨드: 의존성 방향·결합도 감사 (위협순위 기반 리포트)
- CLAUDE.md/AGENTS.md 라우팅 및 architect/code-reviewer 에이전트에 스킬 연동

### 2026-02-26

**리팩토링:**
- 인스톨러 모듈 구조 재편 (300 LOC 파일 한도 적용)
  - `installer.rs` (857 LOC) → `fs/installer/` 디렉토리 (5 파일)
    - `mod.rs` (110) + `process.rs` (206) + `mcp.rs` (163) + `plugin.rs` (64) + `settings.rs` (302)
  - `scanner.rs` (672 LOC) → `fs/scanner/` 디렉토리 (2 파일)
    - `mod.rs` (476) + `validation.rs` (206, 13 테스트 포함)
  - `app.rs` (931 LOC) → `app/` 디렉토리 (7 파일, 이전 완료)

**보안 강화:**
- `shlex::split()` 도입으로 MCP 커맨드 파싱 시 인자 주입 방지 (기존 `split_whitespace()` 대체)
- 경로 순회 방지: scanner 및 copy_file에서 `..` (ParentDir) 컴포넌트 거부

**코드 품질 개선:**
- `ProcessConfig` / `McpInstallConfig` 구조체로 파라미터 수 감소 (6-7 → 2)
- `read_settings` / `write_settings` 헬퍼 추출로 8회 중복 제거
- 매직 넘버 상수화: `CLEANUP_TIMEOUT_SECS`, `KILL_WAIT_MS`, `POLL_INTERVAL_MS`
- `hook_exists_in_array` / `build_hook_entry` 헬퍼 추출

**테스트:**
- 전체 20개 테스트 통과 (7 plugin + 13 scanner validation)

### 2026-02-25 (v0.1.4)

**추가됨:**
- 모든 hooks 및 statusline에 Universal Binary 지원 확대
  - inject_guide: 3.6MB (macOS Universal)
  - load-context: 1.1MB (macOS Universal)
  - preserve-context: 1.1MB (macOS Universal)
  - persist-session: 1.0MB (macOS Universal)
  - suggest-compact: 1.0MB (macOS Universal)
  - statusline: 988KB (macOS Universal)

**수정됨:**
- 프로젝트 구조 재편: src/ 및 dist/ 디렉토리 분리
  - src/: Git 관리 대상 (설정, 훅/statusline 바이너리)
  - dist/: 빌드 산출물 (installer 바이너리, gitignore)
- 모든 빌드 스크립트 Universal Binary 지원
  - tools/installer/build.sh
  - tools/statusline/build.sh
  - tools/hooks/*/build.sh
- package.sh: src/에서 dist/로 복사 후 패키징
- 버전: 0.1.3 → 0.1.4

**기술 상세:**
- 구형 Intel Mac 완전 지원 (모든 컴포넌트)
- 바이너리 관리 전략 개선 (빈번히 변경되는 installer는 dist/, 안정적인 hooks/statusline은 src/에서 Git 관리)

### 2026-02-25 (v0.1.3)

**추가됨:**
- macOS Universal Binary 지원 (installer)
- 버전별 릴리즈 디렉토리 구조 (`release/v{VERSION}/`)
- 자동 SHA256 체크섬 생성 (`package.sh`)
- 빌드 타겟 검증 로직

**수정됨:**
- installer 빌드 시스템: lipo를 사용한 Universal Binary
- 버전: 0.1.2 → 0.1.3

### 2026-02-24 이전

**추가됨:**
- Scoop 설치 지원 (Windows)
- Homebrew share 경로 지원
- Windows 경로 정규화 (diff 출력)

**제거됨:**
- Homebrew Formula (별도 tap으로 이관)

## 개발 요구사항

- Rust 2024 edition
- macOS: Xcode Command Line Tools (lipo 사용)
- Cross-compilation targets:
  - `aarch64-apple-darwin`
  - `x86_64-apple-darwin`
  - `x86_64-pc-windows-gnu`
  - `x86_64-unknown-linux-musl`

## 라이선스

MIT License - 자세한 내용은 [LICENSE](../LICENSE) 참조
