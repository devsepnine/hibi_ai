# hibi-ai 프로젝트 문서

> 마지막 업데이트: 2026-09-09 · 버전 v1.16.0

## 개요

hibi-ai는 Claude Code와 Codex CLI를 위한 TUI(터미널 사용자 인터페이스) 인스톨러다.
에이전트, 슬래시 커맨드, 스킬, MCP 서버, 플러그인, 출력 스타일을 대화형으로 선택해 `~/.claude` 또는 `~/.codex`에 설치한다.

`src/`가 배포되는 설정의 원본(SSOT)이고, `tools/installer/`가 그것을 설치하는 Rust TUI다.

## 프로젝트 구조

```
hibi_ai/
├── src/                    # 배포되는 설정 원본 (Git 관리)
│   ├── agents/             # 에이전트 정의 8개 (+ -ko 미러)
│   ├── commands/           # 슬래시 커맨드 21개 (+ -ko 미러)
│   ├── skills/             # 스킬 25개 (+ 각 SKILL-ko.md)
│   ├── hooks/              # 라이프사이클 훅 5개 — 전부 deprecated
│   ├── mcps/mcps.yaml      # MCP 서버 정의 21개
│   ├── plugins/plugins.yaml# 플러그인 마켓플레이스 4개 / 플러그인 28개
│   ├── output-styles/      # 출력 스타일 (hibi_default)
│   ├── statusline/         # 상태 표시줄 바이너리 (macOS/Linux/Windows)
│   ├── CLAUDE.md           # Claude Code 항상-로드 지침
│   ├── AGENTS.md           # Codex 항상-로드 지침
│   └── settings.json       # 전역 설정
├── tools/
│   ├── installer/          # TUI 인스톨러 (Rust, 49 파일 / 10,226줄)
│   └── statusline/         # 상태 표시줄 소스 (Rust)
├── docs/                   # 이 문서 디렉터리 (README / INDEX / RUNBOOK)
├── .github/workflows/      # release.yml — 태그 푸시로 릴리즈 자동화
├── package.sh              # 릴리즈 패키징
├── dist/                   # 빌드 산출물 (gitignore)
└── release/                # 릴리즈 아티팩트 v{VERSION}/ (gitignore)
```

`src/rules/`와 `src/contexts/`는 더 이상 존재하지 않는다. 정책은 전부 `src/skills/`로 이관됐고, `contexts/`는 Claude Code가 읽지 않는 디렉터리여서 v1.16.0에서 삭제했다. 다만 인스톨러는 두 컴포넌트 타입을 **여전히 지원**하므로 사용자가 자신의 소스에서 `rules/`·`contexts/`를 제공할 수 있다.

## 주요 컴포넌트

### 에이전트 (8개)

`Task`/`Agent`로 띄우는 격리된 워커. 각 항목의 `model`/`effort`는 정의 파일의 frontmatter에 있다.

| 에이전트 | 역할 | model / effort |
|---|---|---|
| `architect` | 시스템 설계, 트레이드오프 기록 | opus / xhigh |
| `assurance-auditor` | A/B 티어 독립 검증·추적성 감사 | sonnet / high |
| `build-error-resolver` | 빌드·타입 에러 최소 diff 수정 | sonnet / medium |
| `code-reviewer` | 품질·보안·유지보수성 리뷰 | sonnet / medium |
| `doc-updater` | 코드에서 코드맵·문서 재생성 | opus / xhigh |
| `e2e-runner` | Playwright E2E 작성·실행·안정화 | sonnet / xhigh |
| `refactor-cleaner` | 데드 코드·미사용 export·중복 제거 | sonnet / xhigh |
| `tdd-guide` | 실패하는 테스트 우선 작성 | sonnet / medium |

라우팅 표는 `src/CLAUDE.md`의 "Agent routing" 섹션이 SSOT다.

### 슬래시 커맨드 (21개)

| 커맨드 | 용도 |
|---|---|
| `/plan` | 요구사항 재정리, 리스크 평가, 단계별 계획 |
| `/orchestrate` | 다중 에이전트 순차 워크플로 |
| `/code-review` | 미커밋 변경의 보안·품질 리뷰 |
| `/security-review` | 10개 범주 보안 체크리스트 감사 (OWASP·CWE 매핑) |
| `/commit` | 프로젝트 규약에 맞춘 커밋 생성 |
| `/pull-request` | PR 생성·리뷰 (제목 형식, 템플릿, PR 전 체크리스트) |
| `/tdd` | 테스트 우선 워크플로 강제 |
| `/test-coverage` | 커버리지 분석 + 부족한 파일 테스트 생성 |
| `/e2e` | Playwright E2E 생성·실행 |
| `/verify` | 빌드·타입·테스트·린트 일괄 검증 |
| `/build-fix` | 타입·빌드 에러 반복 수정 |
| `/refactor-clean` | 데드 코드 탐지 후 안전 제거 |
| `/deps` | 의존성 방향·결합도 감사 |
| `/do-178c` | 보증 티어(A–E) 분류 및 rigor 적용 |
| `/qa-handoff` | git 이력 → 비개발자용 QA 인수 문서 |
| `/eval` | eval 정의·pass@k 측정·회귀 리포트 |
| `/learn` | 세션 패턴을 스킬로 추출 |
| `/upstream-pr` | 세션에서 얻은 개선을 상류 PR로 승격 |
| `/update-docs` | 문서 현행화 |
| `/update-codemaps` | 아키텍처 코드맵 생성 |
| `/checkpoint` | 워크플로 체크포인트 저장·검증 |

### 스킬 (25개)

트리거될 때만 로드된다. 설명은 스킬 목록 문자 예산(200K 윈도우 기준 8,000자)을 공유하므로 각 `description`을 220자 이하로 유지한다 — 예산을 넘으면 초과한 스킬의 설명이 **통째로** 사라져 자동 트리거가 불가능해진다. 예산에 계상되는 값은 `description` 합계가 아니라 목록 항목 합계(스킬명 + 4 + `description`, 항목 구분자 포함)다 — 현재 5,091 / 8,000자.

**정책 (CLAUDE.md 라우팅 표의 SSOT)**

| 스킬 | 정책 |
|---|---|
| `commit-rules` | 커밋 타입/티켓/제목 형식, 커밋 전 보안 점검, 커밋 분할 |
| `pull-request` | PR 제목·템플릿·사전 체크리스트 |
| `security-review` | 인증·입력·시크릿·결제·OWASP 체크리스트 |
| `tdd-workflow` | red/green/refactor, 커버리지 80%+ |
| `coding-standards` | TS/JS/React/Node 표준, 주석 규칙, 코드 스멜 |
| `dependency-design` | 단방향 의존성, 책임 격리, 모노레포 구조 |
| `verification-loop` | 빌드·타입·테스트·보안 검증 루프 |
| `do-178c` | 리스크 티어, 양방향 추적성, 독립 검증 |

**프로세스**

| 스킬 | 용도 |
|---|---|
| `qa-handoff` | git 이력을 QA 인수 문서로 |
| `upstream-pr` | 개선을 배포 설정으로 승격 |
| `eval-harness` | eval 주도 개발, pass@k |
| `obsidian-notes` | Obsidian 볼트 노트 (ADR, 릴리즈 노트, 회고) |

**기술 스택**

| 스킬 | 대상 |
|---|---|
| `composition-patterns` | React 컴포지션 (compound, context, render props) |
| `react-best-practices` | React/Next.js 성능 (Vercel Engineering) |
| `react-native-skills` | React Native / Expo |
| `svelte-5` | Svelte 5 runes + SvelteKit 2 |
| `zustand` | Zustand v5 상태 관리 |
| `web-design-guidelines` | 웹 인터페이스 가이드라인, 접근성 |
| `backend-patterns` | Node / Express / Next.js 라우트, REST, DB |
| `rust-best-practices` | 소유권, 에러 처리, async, 테스트 |
| `ratatui_rs` | Rust TUI (ratatui + crossterm) |
| `iced_rs` | Rust GUI (iced) |
| `clickhouse-io` | ClickHouse 쿼리 최적화·분석 스키마 |
| `superset` | Apache Superset (MCP 경유) |
| `deploy-to-vercel` | Vercel 배포 |

스킬별 부가 자산: 점진적 공개용 `references/` 10개(`backend-patterns`, `coding-standards`, `dependency-design`, `do-178c`, `iced_rs`, `obsidian-notes`, `ratatui_rs`, `rust-best-practices`, `svelte-5`, `zustand`), 벤더링된 상류 규칙 `rules/` 4개(`composition-patterns`, `dependency-design`, `react-best-practices`, `react-native-skills`), 트리거 회귀 평가 `evals/` 9개(`dependency-design`, `do-178c`, `iced_rs`, `obsidian-notes`, `qa-handoff`, `ratatui_rs`, `svelte-5`, `upstream-pr`, `zustand`).

### 훅 (활성 없음)

`src/hooks/`의 5개(`inject_guide`, `load-context`, `persist-session`, `preserve-context`, `suggest-compact`)는 모두 `hook.yaml`에 `deprecated: true`가 표시돼 있고, 인스톨러가 기존 설치본에서 자동 제거한다. 키워드·컨텍스트 주입은 네이티브 Skill 시스템이 대체한다.

### MCP 서버 (21개)

`src/mcps/mcps.yaml`에 정의된다. 문서 조회(`context7`, `cloudflare-docs`, `sveltejs`, `next-devtools`), 브라우저(`playwright`, `firecrawl`), 협업(`atlassian`, `atlassian-remote`, `github`, `sentry`), 데이터(`supabase`, `clickhouse`), 배포·인프라(`vercel`, `railway`, `cloudflare-workers-builds`, `cloudflare-workers-bindings`, `cloudflare-observability`), 기타(`sequential-thinking`, `memory`, `shadcn-ui`, `magic`).

### 플러그인 (마켓플레이스 4개 / 플러그인 28개)

`src/plugins/plugins.yaml`에 정의된다.

- `claude-plugins-official` (25개) — 언어 서버 12종, `pr-review-toolkit`, `code-review`, `skill-creator`, `playwright`, `context7`, `security-guidance`, `commit-commands`, `frontend-design`, `atlassian`, `sentry`, `chrome-devtools-mcp`, `discord`, `ralph-loop`
- `anthropic-agent-skills` (1개) — `document-skills` (xlsx/docx/pptx/pdf)
- `openai-codex` (1개) — `codex` 위임
- `obsidian-skills` (1개) — `obsidian`

### 출력 스타일

`src/output-styles/hibi_default.md` — 한국어 응답, 영문 코드·명령어 유지, 마크다운 구조와 완료 보고 형식을 규정한다. Codex는 `skills/`와 `AGENTS.md`만 설치되므로 `src/AGENTS.md`가 Codex용 서식 규칙의 유일한 사본이다.

### 루트 설정 파일

| 파일 | 설치 대상 |
|---|---|
| `src/CLAUDE.md` | Claude Code |
| `src/AGENTS.md` | Codex |
| `src/settings.json` | Claude Code (기존 설정과 병합) |

`-ko.md` 접미사 파일은 개발자 가독용 한국어 미러다. 인스톨러 스캐너가 건너뛰고 `package.sh`가 번들에서 제거하므로 **설치되지 않는다**.

## 빌드 및 릴리즈

### 빌드

```bash
# Installer — 전 플랫폼 (dist/로 출력)
cd tools/installer && ./build.sh

# Statusline (필요 시만 — src/에서 Git 관리)
cd tools/statusline && ./build.sh
```

생성 바이너리: macOS Universal(Intel + Apple Silicon), Linux x86_64(musl static), Windows x86_64(mingw-w64).

### 테스트

```bash
cargo test --manifest-path tools/installer/Cargo.toml   # 98 tests
```

### 릴리즈 (GitHub Actions 자동화, ~v1.13부터)

1. `tools/installer/Cargo.toml`과 `package.sh`의 버전을 올린다 (+ `cargo update -w`)
2. 커밋 → `main` 푸시 → 태그 `v{VERSION}` 푸시
3. `.github/workflows/release.yml`이 태그에서 트리거된다. 태그 == `package.sh` VERSION == `Cargo.toml` version을 검증한 뒤 전 플랫폼 빌드 → `package.sh` → 아카이브·`checksums.txt`와 함께 GitHub Release 발행
4. 릴리즈 후 **수동**: `homebrew-brew/Formula/hibi.rb`와 `scoop-bucket/hibi-ai.json` 갱신

상세 절차·트러블슈팅·롤백은 [RUNBOOK.md](RUNBOOK.md).

### 패키징

`./package.sh`는 `src/`의 설정 디렉터리와 바이너리를 `dist/`로 모아 플랫폼별 아카이브를 만들고 SHA256 체크섬을 생성한다. `-ko.md` 파일은 이 단계에서 제거된다.

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

[Releases](https://github.com/devsepnine/hibi_ai/releases/latest)에서 플랫폼별 아카이브를 받아 압축 해제 후 실행한다.

```bash
# macOS/Linux
tar xzf hibi-ai-*-macos.tar.gz    # 또는 *-linux.tar.gz
./hibi

# Windows: zip 해제 후 hibi.exe
```

## 사용법

```bash
hibi            # TUI 실행 (소스 resolve + 스캔)
hibi --sync     # git 소스만 업데이트 (TUI 없이)
```

TUI 흐름: 대상 CLI 선택(Claude Code / Codex) → 컴포넌트 선택 → 변경 사항 검토 → 설치.

## 멀티소스 지원

기본값은 릴리스 패키지에 번들된 설정이다. `~/.hibi/sources.yaml`로 git 레포나 로컬 디렉터리를 추가할 수 있다.

```yaml
sources:
  - type: git
    url: "https://github.com/your-org/shared-configs.git"
    branch: main

  - type: local
    path: "~/dotfiles/claude-configs"

auto_update: true   # git 소스 자동 업데이트 (기본 true)
```

### 우선순위

리스트 순서대로 적용되며 마지막이 최우선(last wins):

```
bundled (최저) → sources.yaml 첫 번째 → ... → sources.yaml 마지막 (최고)
```

같은 이름의 파일이 여러 소스에 있으면 마지막 소스의 파일이 사용된다.

### 소스 디렉터리 요구사항

- **번들 소스**(엄격): `agents/`와 `settings.json`이 **둘 다** 있어야 한다
- **사용자 소스**(허용적): 다음 중 **하나 이상** — `agents/`, `commands/`, `contexts/`, `rules/`, `skills/`, `hooks/`, `output-styles/`, `statusline/`, `mcps/mcps.yaml`, `plugins/plugins.yaml`, `settings.json`, `CLAUDE.md`, `AGENTS.md`

### 오프라인 동작

| 상황 | 동작 |
|------|------|
| git 미설치 | 경고 + 해당 소스 skip |
| 네트워크 실패 + 캐시 있음 | 경고 + stale 캐시 사용 |
| 네트워크 실패 + 캐시 없음 | 경고 + 해당 소스 skip |
| bundled | 항상 동작 (오프라인 보장) |

### TUI 표시

멀티소스가 활성화되면 항목마다 소스 태그가 붙는다. 단일 소스(bundled만)일 때는 생략된다.

```
[x] my-agent.md      (new)      [bundled]
[x] custom-agent.md  (new)      [~/dotfiles/claude-configs]
```

### 보안

- git URL: **HTTPS만** 허용, credentials(`@`) 포함 불가
- 로컬 경로: `..` path traversal 금지, `~/.claude/` 내부 경로 금지 (symlink 해제 후 검증)
- git 캐시 위치: `~/.hibi/cache/<sanitized_url>/`
- MCP 커맨드 파싱은 `shlex::split()` — 인자 주입 방지
- Windows에서 `cmd /c` 미사용 (셸 인젝션 방지)

## 설치 이력 (install.json)

설치·제거마다 `~/.hibi/install.json`에 출처를 기록한다.

```json
{
  "source": "https://github.com/devsepnine/hibi_ai",
  "version": "v1.16.0",
  "target": ".claude",
  "updated_at": "2026-09-09T00:00:00Z",
  "components": ["agents/architect", "commands/qa-handoff", "skills/qa-handoff"]
}
```

`components`는 번들 소스에서 온 것만 나열하고, 추가 소스는 `other_sources`에 분리된다. `upstream-pr` 스킬이 이 파일로 클론 없이 상류 저장소를 찾는다. hibi 자신의 디렉터리만 쓰고 `~/.claude` 트리는 건드리지 않는다.

## 최근 변경사항

### 2026-09-09 (v1.16.0 이후)

**`src/` 마크다운 전면 재개편** — 104 파일, +1,057 / −21,188

- 스킬 25개 `description`을 4부 구조(무엇 / `Use when` 트리거 / 한국어 어휘 / `NOT for`)로 재작성. 목록 예산 8,754 → 5,091자로 축소해 설명 절삭 버그 해소
- 벤더링 스킬 4개(`composition-patterns`, `dependency-design`, `react-best-practices`, `react-native-skills`)에서 생성물 `AGENTS.md`·상류 스캐폴딩 제거 (−17,857줄). `rules/`가 원본이므로 정보 손실 없음
- `agents/` 6개 전면 재작성 — 타 프로젝트 사양(암호화폐 마켓·Privy·Solana·Supabase·Redis) 제거, 스킬에 위임
- `commands/` 정리, `keywords:` 안내를 검증된 예산 규칙으로 교체
- `src/CLAUDE.md`/`AGENTS.md` 압축, `src/contexts/`·`src/mcp.md` 삭제 (설치되지 않거나 읽히지 않는 죽은 파일)

### 2026-08 ~ 2026-09 (v1.13.0 ~ v1.16.0)

- 릴리즈 자동화: 태그 푸시 → GitHub Actions 전 플랫폼 빌드·발행
- 설치 provenance 매니페스트(`~/.hibi/install.json`) + `upstream-pr` 플로우
- `do-178c`, `qa-handoff`, `dependency-design` 스킬 및 `/do-178c`, `/qa-handoff`, `/deps` 커맨드 추가
- 모델 정책을 Claude 5 계열로 이관
- `package.sh`가 릴리즈 번들에서 `-ko` 파일 제외
- 사후 리뷰(`code-reviewer`)를 완료 보고 전 필수 게이트로 승격

### 2026-06 ~ 2026-08 (v1.9.x ~ v1.12.0)

- 멀티소스 지원(`~/.hibi/sources.yaml`), 소스 위저드, `hibi --sync`
- Windows `CreateProcessW`가 PATHEXT를 무시해 npm-shim `.cmd`가 안 보이던 문제 수정 (v1.9.7)
- 상류 history rewrite에 대한 sync 복원력: shallow는 `reset --hard FETCH_HEAD`, full clone은 `merge --ff-only` (v1.9.9)
- 정책을 `rules/`에서 스킬로 통합 이관

### 2026-02 ~ 2026-06 (v0.1.3 ~ v0.1.6)

- 인스톨러 모듈 재편: `installer.rs`(857) → `fs/installer/`, `scanner.rs`(672) → `fs/scanner/`, `app.rs`(931) → `app/` (파일당 300줄 목표 도입)
- 보안: `shlex::split()` 도입, scanner·`copy_file`에서 `..` 컴포넌트 거부
- macOS Universal Binary(전 컴포넌트), 버전별 릴리즈 디렉터리, SHA256 체크섬 자동 생성
- `src/`(Git 관리)와 `dist/`(빌드 산출물) 분리
- Scoop(Windows)·Homebrew share 경로 지원, Windows 경로 정규화

전체 이력은 `git log`와 [Releases](https://github.com/devsepnine/hibi_ai/releases)를 참조한다.

## 개발 요구사항

- Rust 2024 edition
- macOS: Xcode Command Line Tools (`lipo`)
- 크로스 컴파일: `brew install mingw-w64 filosottile/musl-cross/musl-cross`
- 타겟: `aarch64-apple-darwin`, `x86_64-apple-darwin`, `x86_64-pc-windows-gnu`, `x86_64-unknown-linux-musl`

## 라이선스

MIT License — [LICENSE](../LICENSE) 참조
