# hibi-ai 프로젝트 문서

> 마지막 업데이트: 2026-02-25

## 개요

hibi-ai는 Claude Code와 Codex CLI를 위한 TUI(터미널 사용자 인터페이스) 인스톨러입니다.
AI 에이전트, 커스텀 명령어, 스킬, 훅, MCP 서버, 플러그인 등 다양한 컴포넌트를 대화형 인터페이스로 쉽게 설치하고 관리할 수 있습니다.

## 프로젝트 구조

```
hibi_ai/
├── src/                 # 소스 디렉토리 (Git 관리)
│   ├── agents/          # AI 에이전트 정의
│   │   └── affaan-m/    # 전문 에이전트 (planner, architect, tdd-guide 등)
│   ├── commands/        # 슬래시 커맨드
│   │   └── affaan-m/    # 커스텀 명령어 (/plan, /code-review, /e2e 등)
│   ├── contexts/        # 컨텍스트 프리셋
│   │   └── affaan-m/    # 개발/리서치/리뷰 컨텍스트
│   ├── hooks/           # 라이프사이클 훅 (바이너리 포함)
│   │   ├── inject_guide/    # 가이드 자동 주입
│   │   ├── load-context/    # 컨텍스트 자동 로드
│   │   ├── persist-session/ # 세션 영속화
│   │   ├── preserve-context/# 컨텍스트 보존
│   │   └── suggest-compact/ # 컨텍스트 압축 제안
│   ├── mcps/            # MCP 서버 설정
│   ├── output-styles/   # 출력 스타일 정의
│   ├── plugins/         # 플러그인 설정
│   ├── rules/           # 코드 스타일 및 워크플로우 규칙
│   │   └── affaan-m/    # 코딩 스타일, Git 워크플로우, 테스팅 등
│   ├── skills/          # 도메인별 스킬
│   │   ├── affaan-m/    # 백엔드/프론트엔드 패턴, TDD 워크플로우
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
│   ├── installer/       # TUI 인스톨러 소스 (Rust)
│   └── statusline/      # 상태 표시줄 소스 (Rust)
└── release/             # 릴리즈 아티팩트
    ├── v0.1.3/          # 버전별 릴리즈 패키지
    └── v0.1.4/          # 최신 릴리즈
```

## 주요 컴포넌트

### 에이전트 (Agents)

전문화된 AI 에이전트들:

- **planner**: 복잡한 기능 구현 계획 수립
- **architect**: 시스템 설계 및 아키텍처 결정
- **tdd-guide**: 테스트 주도 개발 가이드
- **code-reviewer**: 코드 품질 검토
- **security-reviewer**: 보안 취약점 분석
- **build-error-resolver**: 빌드 에러 해결
- **e2e-runner**: E2E 테스트 실행 (Playwright)
- **refactor-cleaner**: 데드 코드 정리
- **doc-updater**: 문서 업데이트

### 명령어 (Commands)

사용 가능한 슬래시 명령어:

- `/plan`: 구현 계획 수립
- `/code-review`: 코드 리뷰 실행
- `/tdd`: TDD 워크플로우 강제
- `/e2e`: E2E 테스트 생성 및 실행
- `/build-fix`: 빌드 에러 수정
- `/update-docs`: 문서 업데이트
- `/refactor-clean`: 리팩토링 및 정리
- `/checkpoint`: 체크포인트 생성
- `/verify`: 검증 실행
- `/commit`: 커밋 생성

### 훅 (Hooks)

설치된 훅 및 용도:

- **inject_guide**: 프롬프트 제출 시 가이드 자동 주입
- **load-context**: 세션 시작 시 컨텍스트 자동 로드
- **persist-session**: 세션 종료 시 상태 저장
- **preserve-context**: 컨텍스트 보존
- **suggest-compact**: 컨텍스트 크기 초과 시 압축 제안

### 스킬 (Skills)

도메인별 전문 지식:

- **composition-patterns**: React 컴포지션 패턴 (React 19 호환)
- **ratatui_rs**: Rust 터미널 UI 개발
- **react-native-skills**: React Native 모바일 앱 개발
- **rust-best-practices**: Rust 소유권, 에러 처리, 비동기 패턴
- **vercel-react-best-practices**: Vercel 엔지니어링 성능 가이드
- **web-design-guidelines**: 웹 인터페이스 가이드라인

### 규칙 (Rules)

코드 품질 및 워크플로우:

- **commit-convention**: 커밋 메시지 규칙
- **code-thresholds**: 파일/함수 크기 제한
- **pull-request-rules**: PR 생성 가이드라인
- **security**: 보안 체크리스트
- **development-workflow**: 개발 워크플로우

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

# Hooks
cd tools/hooks/inject_guide && ./build.sh
cd tools/hooks/memory-persistence && ./build.sh
cd tools/hooks/strategic-compact && ./build.sh
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

## 최근 변경사항

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
