# hibi-ai 문서 인덱스

> 마지막 업데이트: 2026-06-19

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

전문 에이전트 문서는 `agents/` 디렉토리에 위치:

- `architect.md` - 시스템 설계
- `build-error-resolver.md` - 빌드 에러 해결
- `code-reviewer.md` - 코드 리뷰 (security-review 스킬로 보안 검토 포함)
- `doc-updater.md` - 문서 업데이트
- `e2e-runner.md` - E2E 테스트
- `refactor-cleaner.md` - 리팩토링
- `tdd-guide.md` - 테스트 주도 개발

### Commands

커스텀 명령어 문서는 `commands/` 디렉토리에 위치:

- `code-review.md` - /code-review 명령어
- `tdd.md` - /tdd 명령어
- `e2e.md` - /e2e 명령어
- `build-fix.md` - /build-fix 명령어
- `update-docs.md` - /update-docs 명령어
- `refactor-clean.md` - /refactor-clean 명령어
- `deps.md` - /deps 명령어 (의존성·결합도 감사)
- 기타 명령어들...

### Skills

스킬 문서는 각 스킬 디렉토리의 `SKILL.md`에 위치:

- `composition-patterns/SKILL.md` - React 컴포지션 패턴
- `dependency-design/SKILL.md` - 의존성·결합도 설계 (Cynefin·공생성·DDD·모노레포)
- `ratatui_rs/SKILL.md` - Ratatui TUI 개발
- `react-native-skills/SKILL.md` - React Native 개발
- `rust-best-practices/SKILL.md` - Rust 베스트 프랙티스
- `vercel-react-best-practices/SKILL.md` - Vercel React 최적화
- `web-design-guidelines/SKILL.md` - 웹 디자인 가이드

### Rules / 정책

`rules/` 디렉토리는 비어 있다. 상세 정책은 모두 **Skill**로 이동했다 (트리거 시에만 로드되어 always-on 컨텍스트를 가볍게 유지):

- 커밋 규칙 → `commit-rules` skill
- PR 규칙 → `pull-request` skill
- 보안 규칙 / OWASP → `security-review` skill
- 테스트 & TDD → `tdd-workflow` skill
- 코딩 스타일 / 클린 코드 → `coding-standards` skill
- 빌드 & 타입 에러 → `verification-loop` skill

코드 임계값·리뷰 체크리스트·공통 패턴은 `skills/coding-standards/references/`(`code-thresholds.md`, `review-checklist.md`, `patterns.md`)에 위치. 항상 적용되는 핵심 불변식·effort/agent 라우팅은 `CLAUDE.md`에 있다.

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
A: `agents/` 디렉토리의 기존 에이전트 참조

**Q: 커밋 메시지 형식은?**
A: `commit-rules` skill 참조 (또는 `/commit` 커맨드)

## 📝 문서 작성 가이드

새로운 문서를 작성할 때:

1. **마크다운 형식** 사용
2. **한글 설명 + 영문 코드/경로** 조합
3. **명확한 제목 및 섹션 구조**
4. **코드 예제 포함** (가능한 경우)
5. **업데이트 날짜 명시**

## 📅 문서 업데이트 이력

- **2026-06-19**: `dependency-design` 스킬 및 `/deps` 커맨드 추가 반영
- **2026-02-26**: 인스톨러 모듈 구조 재편 반영 (README.md 업데이트)
- **2026-02-25**: 초기 문서 생성 (README.md, RUNBOOK.md, INDEX.md)
