# hibi-ai 프로젝트 문서

> 마지막 업데이트: 2026-09-11 · 버전 v1.16.0

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
│   ├── skills/             # 스킬 24개 (+ 각 SKILL-ko.md)
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

### 스킬 (24개)

트리거될 때만 로드된다. 설명은 스킬 목록 문자 예산(200K 윈도우 기준 8,000자)을 공유하므로 각 `description`을 220자 이하로 유지한다 — 예산을 넘으면 초과한 스킬의 설명이 **통째로** 사라져 자동 트리거가 불가능해진다. 예산에 계상되는 값은 `description` 합계가 아니라 목록 항목 합계(스킬명 + 4 + `description`, 항목 구분자 포함)다 — 현재값은 `python3 src/skills/eval-harness/scripts/skill_budget.py src/skills` 로 측정한다.

**정책 (CLAUDE.md 라우팅 표의 SSOT)**

| 스킬 | 정책 |
|---|---|
| `commit-rules` | 커밋 타입/티켓/제목 형식, 커밋 전 보안 점검, 커밋 분할 |
| `pull-request` | PR 제목, diff에서 쓰는 본문, 사전 게이트, 필요성 검증 리뷰, 리뷰 코멘트 분류, 업스트림 설정 기여 |
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

스킬별 부가 자산: 점진적 공개용 `references/` 11개(`backend-patterns`, `coding-standards`, `dependency-design`, `do-178c`, `iced_rs`, `obsidian-notes`, `pull-request`, `ratatui_rs`, `rust-best-practices`, `svelte-5`, `zustand`), 벤더링된 상류 규칙 `rules/` 4개(`composition-patterns`, `dependency-design`, `react-best-practices`, `react-native-skills`), 평가 세트 `evals/` 9개(`dependency-design`, `do-178c`, `iced_rs`, `obsidian-notes`, `pull-request`, `qa-handoff`, `ratatui_rs`, `svelte-5`, `zustand`). `evals/`에는 두 종류가 들어간다 — `evals.json`은 출력 품질 평가(`prompt` + `expected_output`), `trigger-eval.json`은 설명이 실제로 발화하는지 보는 트리거 회귀 세트(`query` + `should_trigger`)로 `trigger_eval.py --eval-set`이 소비한다.

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
cargo test --manifest-path tools/installer/Cargo.toml   # 105 tests
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

`components`는 번들 소스에서 온 것만 나열하고, 추가 소스는 `other_sources`에 분리된다. `pull-request` 스킬이 이 파일로 클론 없이 상류 저장소를 찾는다. hibi 자신의 디렉터리만 쓰고 `~/.claude` 트리는 건드리지 않는다.

## 최근 변경사항

### 2026-09-14

- `commit-rules`에서 조직 하드코딩 제거 — `[PP-XXXX]`(예: PP-6050)가 배포되는 공개 설정에 박혀 있었다. `pull-request` §1과 같은 규칙으로 교체: 브랜치와 `git log`에서 추출하고, 어느 쪽에도 없으면 접두사를 완전히 생략한다(`[TICKET-1]` 같은 자리표시자는 없는 것보다 나쁘다)
- 같은 수정으로 컨벤션이 자기 저장소와 어긋나던 문제가 닫혔다 — 형식이 `<type>: [<ticket-number>] <title>` 하나뿐이어서 티켓을 필수로 요구했는데, 이 저장소 최근 커밋 30개 중 접두사를 가진 것은 0개였다. 즉 규약을 지킨 커밋이 하나도 없는 상태였다. 형식 블록이 티켓 있는 경우와 없는 경우 두 줄을 함께 보여준다
- 전파 지점 6곳을 함께 맞췄다 — `src/skills/commit-rules/SKILL.md:13-14,31-38`, `src/commands/commit.md:13`, `src/CLAUDE.md:151` 및 각 `-ko` 트윈. `src/AGENTS.md`는 커밋 형식을 갖고 있지 않아 대상이 아니다(CLAUDE/AGENTS는 중복시키지 않는 구조)
- 티켓 추출의 결정 불가 구간을 닫았다 — 브랜치엔 티켓이 없고 이력에만 접두사가 있으면(트래커 쓰는 저장소의 `fix/typo` 브랜치, main 직접 커밋) "둘 다 없으면 생략"이 발동하지 않아 무관한 커밋의 번호를 가져다 붙일 수 있었다. 남의 실재 티켓은 형식상 유효해 보여서 자리표시자보다 잡기 어렵다. `pull-request/SKILL.md:44`가 base 브랜치에 이미 쓰던 관용구("둘이 어긋나면 말하고 묻는다")를 티켓에도 적용: **판단은 브랜치가 한다**. `commit-rules`와 `pull-request` §1 양쪽을 같은 커밋에서 고쳤다 — 한쪽만 고치면 "같은 규칙" 주장이 깨진다

### 2026-09-11

- `upstream-pr` 스킬을 `pull-request`로 통합 (스킬 25 → 24). 실체가 PR의 변종이 아니라 설정 기여 판단 워크플로였고, PR 메커닉은 6개 섹션 중 하나뿐이었다. 방법론은 `pull-request/references/upstream-config.md`로 이동, `/upstream-pr` 커맨드는 진입점으로 유지. `src/CLAUDE.md`의 정책 라우팅 행이 두 진입점을 모두 명시한다(`AGENTS.md`는 Codex용이라 커맨드를 쓰지 않으므로 스킬 이름만 가리킨다)
- 통합 과정에서 경계 결함 정리 — 단방향 위임(역참조 `Related` 표 부재), 티켓 없는 설정 PR에 `[TICKET-ID]`를 강제하던 모순, 권한 규칙의 두 버전(베이스 vs 공개 상류 2단 확인) 관계 미명시, 일반 스킬에 없던 브랜치 네이밍 규약
- `pull-request` 스킬에서 조직 하드코딩 제거 — `ggnetwork.atlassian.net`, `PP-XXXX`, `upstream/develop`. 규약을 기억이 아니라 저장소에서 읽는다: base 브랜치는 `gh`로 확인, 티켓은 브랜치·히스토리에서 추출(없으면 접두사 생략), 본문은 `.github/pull_request_template.md`가 무조건 우선. `commit-rules`와 `/commit`의 같은 하드코딩은 이번 범위를 벗어나 후속으로 미뤘다 — 2026-09-14 항목에서 정리했다
- `gh pr create` 메커닉(`--draft`/`--base`/`--body-file`)과 기존 PR 리뷰 절차 신규 추가 — 종전 description이 리뷰를 주장했으나 체크리스트만 있었다
- `pull-request/evals/` 추가. 구 `upstream-pr` 트리거 세트를 옮길 때 네거티브 2건이 포지티브로 반전됨(디렉터리명 기준 매칭이므로)
- `pull-request`에 §2 "본문은 diff에서 쓴다" 신규 — `git diff <base>...HEAD`(점 세 개)로 읽고, 본문과 hunk를 양방향으로 대응시켜 미대응 항목을 범위 이탈 또는 허구로 잡아낸다. 간결함의 정의를 "짧게"가 아니라 "diff가 보여줄 수 없는 것만"으로 못박았다
- §5 리뷰에 판단 순서(본문-diff 대응 → 필요성 → 정확성 → 테스트)와 **필요성 검증** 추가 — 삭제 테스트, 호출자 수, 기존 구현 grep, 실행될 일 없는 가드, 목적 무관 hunk. 필요성은 판결이 아니라 질문이므로 판단이 안 되면 작성자에게 묻는다
- §6 "리뷰 코멘트를 처리한다" 신규 — 코멘트를 지시가 아니라 분류할 주장으로 다룬다. 6분류 표(범위 내 블로킹 / 스타일 / 기존 문제 / 새 요구사항 / 이미 처리됨 / 잘못된 전제), 범위 테스트("이 PR이 없었어도 필요했나"), n차 라운드는 코멘트·커밋 타임스탬프 비교로 이미 처리된 지적을 걸러낸다
- "추측하지 말고 묻는다"를 도입부 원칙으로 세우고 §2·§5·§6이 각각 구체적 트리거를 갖게 했다 — 답이 쓸 내용을 바꿀 때만 묻고, 그렇지 않으면 가정을 본문에 적는다. `description`은 `리뷰 코멘트 반영` 어휘를 얻고 216자(한도 220)
- 통합이 만든 댕글링 지시문 정리 — `"run /upstream-pr"` 형태의 지시는 모델이 존재하지 않는 `Skill{upstream-pr}` 호출로 해석한다(트리거 측정에서 3회 재현). `src/CLAUDE.md:36`·`commands/learn.md:98`·`commands/upstream-pr.md:25`가 모두 커맨드임을 밝히고 로드할 스킬 이름(`pull-request`)을 명시하도록 고쳤다. 같은 파일의 방법론 참조는 삭제된 `upstream-pr` 스킬에서 `pull-request` §1–§7로 재지정했다
- 사후 리뷰 findings 반영 — (a) 라우팅 대상 이름 정정: `code-review`는 스킬로 존재하지 않는다(`/code-review` 커맨드와 `code-reviewer` 에이전트가 실체). `SKILL.md`·`SKILL-ko.md`의 §5c와 Related 표, `evals.json` #3의 expected_output을 함께 고쳤다. (b) §6 코멘트 조회에 `--paginate` 추가 — REST 한 페이지가 30건에서 끊겨 "해결된 라운드까지 스레드 전체를 읽는다"는 전제와 배치됐고, n차 리뷰에서 이미 처리된 지적을 놓치는 무음 실패가 된다(`per_page=2`로 실증: 플래그 없이 2건, 붙이면 3건). (c) KO 표현 2건 정정 — "싼 질문"→"가장 저렴한 질문", "발화할 수 없는 가드"→"실행될 일 없는 가드". (d) 통합 때 유실된 `/learn` 역포인터를 Related 표에 복구 — 순방향(`learn.md:98` → `/upstream-pr`)만 남아 있었다. (e) §6의 커밋 조회를 `gh pr view --json commits`에서 `gh api --paginate .../pulls/<n>/commits`로 교체 — 전자는 100건에서 끊기고 **가장 오래된** 100건을 주므로 긴 PR에서 head 커밋이 빠진다(kubernetes/kubernetes#141727로 실증: 반환 100건, 마지막이 `3e8a81e`인데 실제 head는 `9c3b2d0`). n차 라운드 판별이 최근 커밋 타임스탬프에 걸려 있으므로 무음 오판이 된다. REST 페이지네이션은 같은 PR에서 head를 포함한다
- `description`은 고치지 않았다 — "PR 템플릿" 어휘 손실과 설정 변경 트리거 갭이 지적됐으나 두 질의(`이 저장소 PR 템플릿 …`, `이번 세션에서 배운 규칙을 배포 설정에 반영해줘`) 모두 격리 리그에서 트리거되어 전제가 재현되지 않았다(2/2 PASS). 216/220자에서 재작성하는 대신 그 두 질의를 회귀 세트에 고정했다 — 트리거 케이스 9 → 11(true 6 → 8)
- 2차 사후 리뷰(diff 전체 독립 검토) findings 7건 반영 — (a) 이 변경 이력 자체의 허위 주장 삭제: `commands/upstream-pr.md:25`의 `§1–§5` 참조를 §1–§7로 "정정"했다고 적었으나 HEAD의 그 줄에는 § 참조가 없었고(삭제된 `upstream-pr` 스킬을 SSOT로 가리키는 문장이었다) 구 스킬은 5섹션도 아니었다 — 작업 중 초안 수정을 HEAD 대비 정정으로 오기한 것이므로 "방법론 참조 재지정"으로 바꿨다(`docs/INDEX.md:144`도 동일). (b) §6 코멘트 조회 jq에 `\(.created_at)` 추가 — "어떤 커밋보다 먼저 쓰인 코멘트는 이미 처리됐을 수 있다"는 §6의 근거와 `evals.json` #10이 코멘트·커밋 타임스탬프 비교를 요구하는데 조회는 `path:line`·`user`·`body`만 투영해, 스킬을 그대로 따른 모델이 비교할 입력을 갖지 못했다. 필드 실재와 형식은 실측 확인(`cli/cli#12444` → `2026-01-08T01:09:41Z`, `.commit.committer.date`와 같은 ISO-8601이라 직접 비교된다). 두 조회가 모두 타임스탬프로 시작하게 되어 본문 설명도 그에 맞췄다. (c) `evals.json` #8의 과잉 수용 제거 — "명시적으로 라벨링한 가정으로 기록"도 통과시켰는데, magic number와 skip된 테스트는 그 *이유*가 본문에 쓸 내용을 바꾸므로 스킬 자체 논리("답이 쓸 내용을 바꾸지 않을 때만 가정을 적는다")로는 물어야 한다 — 라벨 붙인 추측을 통과시키던 절이었다. (d) 폴백 템플릿의 `perf` 삭제 — "커밋 type과 일치하는 하나"라고 하면서 `commit-rules`에 없는 type을 제시했다(두 트윈). (e) KO 조사 오류 정정 — `SKILL-ko.md:167` "hunk가 가장 빨리 찾을 수 있고"는 hunk를 찾는 주체로 만든다 → "hunk는 가장 빨리 찾아낼 수 있고". (f) 과장 표현 완화 — "셸 보간을 온전히 통과하지 못한다"는 반증 가능하다(제대로 인용하면 통과한다) → "셸 인용 과정에서 쉽게 망가진다". (g) 트윈 마크업 비대칭 1건 해소(`upstream-config-ko.md:4` 볼드 제거)

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
