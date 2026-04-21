# hibi-ai 문서 인덱스

> 마지막 업데이트: 2026-02-26

## 📚 문서 목록

### 핵심 문서

- **[README.md](README.md)** - 프로젝트 전체 문서
  - 프로젝트 개요
  - 디렉토리 구조
  - 주요 컴포넌트 설명
  - 설치 및 사용 방법
  - 최근 변경사항

- **[RUNBOOK.md](RUNBOOK.md)** - 운영 가이드
  - 배포 절차
  - 모니터링 및 알림
  - 일반적인 문제 해결
  - 롤백 절차
  - 긴급 대응 프로토콜

### 프로젝트 루트 문서

- **[../README.md](../README.md)** - 프로젝트 소개 (영문)
- **[../CLAUDE.md](../CLAUDE.md)** - Claude Code 설정 및 규칙
- **[../AGENTS.md](../AGENTS.md)** - 에이전트 구성 설명

## 🔧 컴포넌트별 문서

### Agents

전문 에이전트 문서는 `agents/affaan-m/` 디렉토리에 위치:

- `planner.md` - 구현 계획 수립
- `architect.md` - 시스템 설계
- `tdd-guide.md` - 테스트 주도 개발
- `code-reviewer.md` - 코드 리뷰
- `security-reviewer.md` - 보안 검토
- `build-error-resolver.md` - 빌드 에러 해결
- `e2e-runner.md` - E2E 테스트
- `refactor-cleaner.md` - 리팩토링
- `doc-updater.md` - 문서 업데이트

### Commands

커스텀 명령어 문서는 `commands/affaan-m/` 디렉토리에 위치:

- `plan.md` - /plan 명령어
- `code-review.md` - /code-review 명령어
- `tdd.md` - /tdd 명령어
- `e2e.md` - /e2e 명령어
- `build-fix.md` - /build-fix 명령어
- `update-docs.md` - /update-docs 명령어
- `refactor-clean.md` - /refactor-clean 명령어
- 기타 명령어들...

### Skills

스킬 문서는 각 스킬 디렉토리의 `SKILL.md`에 위치:

- `composition-patterns/SKILL.md` - React 컴포지션 패턴
- `ratatui_rs/SKILL.md` - Ratatui TUI 개발
- `react-native-skills/SKILL.md` - React Native 개발
- `rust-best-practices/SKILL.md` - Rust 베스트 프랙티스
- `vercel-react-best-practices/SKILL.md` - Vercel React 최적화
- `web-design-guidelines/SKILL.md` - 웹 디자인 가이드

### Rules

코딩 규칙 문서는 `rules/` 디렉토리에 위치 (영문 통일):

- `essential-rules.md` - 필수 규칙
- `commit-convention.md` - 커밋 규칙
- `code-thresholds.md` - 코드 크기 제한
- `pull-request-rules.md` - PR 규칙
- `security.md` - 보안 규칙 및 체크리스트
- `development-workflow.md` - 개발 워크플로우 (TDD, 코드 리뷰 포함)
- `coding-style.md` - 코딩 스타일
- `testing.md` - 테스팅 규칙
- `performance.md` - 성능 최적화
- `patterns.md` - 공통 패턴
- `hooks.md` - 훅 시스템
- `agents.md` - 에이전트 오케스트레이션

## 🛠️ 개발 문서

### 인스톨러 소스 (tools/installer/src/)

모듈 구조:

- **app/** - 앱 상태 관리 (7 파일, 967 LOC)
  - `mod.rs` (App struct), `types.rs`, `navigation.rs`, `selection.rs`, `processing.rs`, `input.rs`, `settings.rs`
- **fs/installer/** - 설치/제거 로직 (5 파일, 845 LOC)
  - `mod.rs` (component ops), `process.rs` (spawn/cancel), `mcp.rs` (MCP), `plugin.rs` (plugin), `settings.rs` (settings.json)
- **fs/scanner/** - 컴포넌트 스캔 (2 파일, 682 LOC)
  - `mod.rs` (scan functions), `validation.rs` (검증 + 13 tests)
- **main.rs** - 이벤트 루프 (754 LOC)

### 빌드 시스템

- **tools/installer/build.sh** - 전체 플랫폼 빌드 스크립트
- **package.sh** - 릴리즈 패키징 스크립트

### 설정 파일

- **tools/installer/Cargo.toml** - Rust 프로젝트 설정
- **settings.json** - Claude Code 설정
- **plugins/plugins.yaml** - 플러그인 정의
- **mcps/mcps.yaml** - MCP 서버 설정

## 📖 사용법 가이드

### 신규 사용자

1. [README.md](README.md)의 "설치 방법" 섹션 참조
2. `hibi` 실행 후 대화형 인터페이스 사용
3. 필요한 컴포넌트 선택 및 설치

### 개발자

1. [RUNBOOK.md](RUNBOOK.md)의 "배포 절차" 참조
2. 빌드 시스템 이해: `tools/installer/build.sh`
3. 릴리즈 프로세스: `package.sh` → GitHub Release

### 기여자

1. [../CLAUDE.md](../CLAUDE.md)의 개발 규칙 숙지
2. 에이전트/스킬 추가 시 해당 디렉토리 구조 참조
3. PR 생성 전 `rules/pull-request-rules.md` 확인

## 🔍 빠른 찾기

### 자주 묻는 질문

**Q: macOS에서 "developer cannot be verified" 경고가 나옵니다.**
A: [RUNBOOK.md](RUNBOOK.md)의 "macOS Gatekeeper 경고" 섹션 참조

**Q: 빌드가 실패합니다.**
A: [RUNBOOK.md](RUNBOOK.md)의 "빌드 실패" 섹션 참조

**Q: 새로운 에이전트를 추가하려면?**
A: `agents/affaan-m/` 디렉토리의 기존 에이전트 참조

**Q: 커밋 메시지 형식은?**
A: `rules/commit-convention.md` 참조

## 📝 문서 작성 가이드

새로운 문서를 작성할 때:

1. **마크다운 형식** 사용
2. **한글 설명 + 영문 코드/경로** 조합
3. **명확한 제목 및 섹션 구조**
4. **코드 예제 포함** (가능한 경우)
5. **업데이트 날짜 명시**

## 📅 문서 업데이트 이력

- **2026-02-26**: 인스톨러 모듈 구조 재편 반영 (README.md 업데이트)
- **2026-02-25**: 초기 문서 생성 (README.md, RUNBOOK.md, INDEX.md)
