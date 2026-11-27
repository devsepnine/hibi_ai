# hibi-ai 프로젝트 문서

기준 코드 `cc8978b` · `2026-11-27` · 공개 릴리스 `v2.0.0`

## 개요

hibi-ai는 Claude Code와 Codex CLI의 설정을 고르고 설치하는 Rust 앱이다. 기본 실행은 iced GUI이며 `hibi --tui`는 ratatui 화면을 연다. 같은 앱 모델이 소스, 파일 비교, 설치·제거, 설치 기록을 관리한다.

Claude Code에서는 에이전트, 커맨드, 스킬, MCP 서버, 플러그인, 출력 스타일과 설정 항목을 관리한다. Codex의 화면에는 Skills, Config, MCP가 있다. 설치 파일과 처음 실행하는 방법은 [루트 README 한국어판](../README.ko.md)을 참고한다.

## 프로젝트 구조

| 경로 | 용도 |
|---|---|
| `src/` | 배포 설정 원본과 상태줄 바이너리 |
| `tools/installer/` | GUI와 TUI가 들어 있는 인스톨러, 플랫폼별 패키징 도구, 스모크 테스트 |
| `tools/statusline/` | Claude Code용 독립 상태줄 바이너리 소스 |
| `docs/` | 운영 가이드, 아키텍처 규칙, 화면별 기능맵 |
| `.github/workflows/release.yml` | 검증, 플랫폼별 빌드, 패키징, 서명, 게시 |
| `package.sh`, `nfpm.yaml`, `install.sh` | 배포 아카이브와 시스템 패키지 구성, Linux 사용자 설치 |
| `dist/`, `release/` | Git에서 제외하는 로컬 빌드 결과와 릴리스 패키지 |

모듈과 문서 위치는 [문서 인덱스](INDEX.md), 동작별 코드와 오류 문구는 [기능맵](FEATURES.md)을 참고한다.

## 주요 컴포넌트

### 에이전트 8개

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

### 슬래시 커맨드 28개

| 커맨드 | 용도 |
|---|---|
| `/plan` | 요구사항 재정리, 리스크 평가, 데이터 형태 우선 설계, 처리량 체크포인트, 단계별 계획 |
| `/orchestrate` | 다중 에이전트 순차 워크플로 |
| `/bugfix` | 보고된 화면에서 재현, 런타임 증거로 원인 이분 탐색, 증거가 뒷받침하는 최소 수정, 같은 화면에서 검증 |
| `/refactor` | 동작을 먼저 고정하고 구조만 변경. 추가 전 삭제, 작은 녹색 단계, 읽는 부담이 줄 때만 유지 |
| `/perf` | 측정된 느림 개선. 기준선 먼저, 변경 하나에 측정 하나, 유지 또는 되돌림, 수치마다 한계 요인 설명 |
| `/code-review` | 미커밋 변경의 보안·품질 리뷰 |
| `/review-panel` | A/B 티어 diff를 관점 하나씩 맡은 `code-reviewer` 패널로 검토하고 Act on, Consider, Dismissed와 판정으로 합친다 |
| `/blast-radius` | 머지 전 diff가 바깥에서 깨뜨릴 수 있는 것 탐색, 안전 근거를 실제 코드 실행으로 증명 |
| `/security-review` | 10개 범주 보안 체크리스트 감사. OWASP·CWE 매핑 |
| `/commit` | 프로젝트 규약에 맞춘 커밋 생성 |
| `/pull-request` | PR 생성·리뷰. 제목 형식, 템플릿, PR 전 체크리스트 |
| `/tdd` | 테스트 우선 워크플로 강제 |
| `/test-coverage` | 커버리지 분석 + 부족한 파일 테스트 생성 |
| `/e2e` | Playwright E2E 생성·실행 |
| `/verify` | 빌드·타입·테스트·린트 일괄 검증 |
| `/build-fix` | 타입·빌드 에러 반복 수정 |
| `/refactor-clean` | 데드 코드 탐지 후 안전 제거 |
| `/deps` | 의존성 방향·결합도 감사 |
| `/architecture-rules` | 코드에서 아키텍처 규칙 `docs/ARCHITECTURE.md` 작성·갱신. 회색 지대는 Q&A, 기계적 규칙은 lint 초안 |
| `/do-178c` | 보증 티어 A부터 E까지 분류 및 rigor 적용 |
| `/qa-handoff` | git 이력 → 비개발자용 QA 인수 문서 |
| `/feature-map` | 증상→코드 기능맵 `docs/FEATURES.md` 작성·갱신 |
| `/eval` | eval 정의·pass@k 측정·회귀 리포트 |
| `/learn` | 완료한 작업을 돌아보고 재사용할 만한 검사나 스킬 개선을 제안 |
| `/upstream-pr` | 세션에서 얻은 개선을 상류 PR로 승격 |
| `/update-docs` | 문서 현행화 |
| `/update-codemaps` | 아키텍처 코드맵 생성 |
| `/checkpoint` | 워크플로 체크포인트 저장·검증, 재개 노트를 남기는 일시정지와 재개, 장시간 무인 실행 규칙, `/perf`와 같이 쓰는 결정 로그 형식 |

### 스킬

스킬은 해당 작업에서 참조한다. `technical-writing`은 Claude Code의 상대 경로 import와 Codex의 상시 지침으로 매 턴 적용한다. 아래 표는 기준 커밋에 포함된 스킬을 다룬다. 설명은 220자 이하로 유지하고 목록 예산은 `python3 src/skills/eval-harness/scripts/skill_budget.py src/skills`로 측정한다.

**정책: CLAUDE.md 라우팅 표의 SSOT**

| 스킬 | 정책 |
|---|---|
| `commit-rules` | 커밋 타입/티켓/제목 형식, 커밋 전 보안 점검, 커밋 분할 |
| `pull-request` | PR 제목, diff에서 쓰는 본문, 사전 게이트, 필요성 검증 리뷰, 리뷰 코멘트 분류, 업스트림 설정 기여 |
| `security-review` | 인증·입력·시크릿·결제·OWASP 체크리스트 |
| `tdd-workflow` | red/green/refactor, 커버리지 80%+, 단언 강도 |
| `coding-standards` | TS/JS/React/Node 표준, 타입 안전·경계 파싱, 주석 규칙, 코드 스멜 |
| `architecture-rules` | 레이어·import 방향·배치 규칙을 코드에서 근거와 함께 도출, 회색 지대 Q&A, lint 초안 |
| `technical-writing` | 짧은 문답은 첫 문장에 답하고, 장문 산출물은 독자가 내릴 결정이나 찾을 내용에 맞춰 구성한다. 기술 문서는 목적에 맞는 Diátaxis 형식을 따른다. |
| `dependency-design` | 단방향 의존성, 책임 격리, 모노레포 구조 |
| `verification-loop` | 빌드·타입·테스트·보안 검증 루프, 실제 산출물 확인 |
| `do-178c` | 리스크 티어, 양방향 추적성, 독립 검증 |

**프로세스**

| 스킬 | 용도 |
|---|---|
| `qa-handoff` | git 이력을 QA 인수 문서로 |
| `skill-improvement` | 작업 완료 후 지적과 후속 요청을 검토하고 재사용할 수 있는 개선안을 제안 |
| `feature-map` | 화면 위치·사용자 표현·UI 문구로 코드를 찾는 기능맵 |
| `blast-radius` | 머지 전 파급 범위 검증, 안전 근거를 증거 사다리로 증명 |
| `why` | 설계 의도 추적: 코드가 왜 이런 모양인지 git·PR·티켓·문서에서 찾고 주장마다 신뢰 등급 표시 |
| `how` | 런타임 동작 설명: 서브시스템이나 흐름을 실제 호출 체인으로 따라가고 주장마다 파일 인용 |
| `eval-harness` | eval 주도 개발, pass@k |
| `obsidian-notes` | Obsidian 볼트 노트: ADR, 릴리즈 노트, 회고 |

**기술 스택**

| 스킬 | 대상 |
|---|---|
| `composition-patterns` | React 컴포지션: compound, context, render props |
| `react-best-practices` | React/Next.js 성능. Vercel Engineering 기준 |
| `react-native-skills` | React Native / Expo |
| `svelte-5` | Svelte 5 runes + SvelteKit 2 |
| `zustand` | Zustand v5 상태 관리 |
| `web-design-guidelines` | 웹 인터페이스 가이드라인, 접근성 |
| `backend-patterns` | Node / Express / Next.js 라우트, REST, DB |
| `rust-best-practices` | 소유권, 에러 처리, async, 테스트 |
| `ratatui-rs` | Rust TUI, ratatui + crossterm |
| `iced-rs` | Rust GUI, iced |
| `clickhouse-io` | ClickHouse 쿼리 최적화·분석 스키마 |
| `superset` | Apache Superset, MCP 경유 |
| `deploy-to-vercel` | Vercel 배포 |

스킬은 `src/skills/<name>/SKILL.md`와 옆의 `SKILL-ko.md`로 구성한다. 부가 문서는 `references/`, 템플릿은 `assets/`, 실행 스크립트는 `scripts/`, 규칙 묶음은 `rules/`, 재사용할 평가는 `evals/`에 둔다. 파일 배치와 이름 규칙은 [아키텍처 규칙](ARCHITECTURE.md)을 따른다.

평가 정의인 `evals.json`은 요청과 기대 결과를 담는다. `trigger-eval.json`은 요청과 기대 트리거 값을 담으며 `trigger_eval.py --eval-set`으로 실행한다. 평가 실행 결과는 Git에서 제외하는 `skill-evaluations/`에 둔다.

### 훅: 활성 없음

`src/hooks/`의 5개인 `inject_guide`, `load-context`, `persist-session`, `preserve-context`, `suggest-compact`는 모두 `hook.yaml`에 `deprecated: true`가 표시돼 있고, 인스톨러가 기존 설치본에서 자동 제거한다. 키워드·컨텍스트 주입은 네이티브 Skill 시스템이 대체한다.

### MCP 서버 10개

`src/mcps/mcps.yaml`에서 설치할 서버를 고른다.

| 작업 | 서버 | 선택 기준 |
|---|---|---|
| 라이브러리 문서와 프레임워크 | `context7`, `next-devtools`, `sveltejs`, `shadcn-ui` | Context7은 버전별 문서 조회에 쓴다. 나머지는 프로젝트의 기술에 맞춰 고른다. |
| 브라우저 탐색 | `playwright` | 여러 화면을 오가며 브라우저 상태를 유지할 때 쓴다. |
| 기록 | `memory`, `sequential-thinking` | 지식 그래프 보존과 사고 단계 기록 중 필요한 쪽을 고른다. |
| 협업과 장애 조사 | `atlassian-remote`, `sentry` | Jira와 Confluence Cloud에는 Atlassian 공식 원격 서버를 쓴다. |
| 데이터 조회 | `clickhouse` | ClickHouse Cloud에서 MCP를 활성화한 뒤 연결한다. |

Claude에서 Context7, Playwright, Atlassian Cloud, Sentry 플러그인을 쓰면 해당 MCP도 함께 등록된다. 같은 서비스에는 설치 경로 하나를 선택한다.

Context7 API 키를 쓰는 경우 설치기를 실행하기 전에 `CONTEXT7_API_KEY`를 설정하고 Claude 또는 Codex를 실행하는 환경에도 제공한다. 설치기는 키가 있을 때 변수 이름만 등록하며, 키가 없으면 일반 HTTP 연결을 등록한다. 인증 방식을 바꾸려면 해당 연결을 다시 등록한다.

Codex가 OAuth 로그인을 요구하면 설치 결과에 표시된 `codex mcp login 서버이름`을 실행한다. Claude에서는 `/mcp`에서 연결 상태와 로그인을 확인한다.

Atlassian Cloud는 `atlassian-remote`를 선택한 뒤 브라우저에서 Atlassian 계정으로 로그인한다. Codex에서는 `codex mcp login atlassian-remote`로, Claude Code에서는 `/mcp`로 로그인을 시작할 수 있다.

Memory에는 사용자 소유 디렉터리의 JSONL 파일 절대 경로를 지정한다. 기존 파일의 심볼릭 링크를 선택하면 설치기는 링크가 가리키는 파일 경로를 등록한다. 기존 그래프를 보존하려면 서버를 종료하고 기존 데이터 파일을 백업한 뒤 보관할 위치로 복사해 그 경로를 선택한다.

MCP 또는 플러그인 설치를 취소하면 설치기는 실행 중인 CLI를 중단하고 목록을 갱신한다. CLI가 취소 전에 등록을 마쳤다면 목록에서 확인하고 필요할 때 직접 제거한다. Codex에서는 이미 등록한 이름을 건너뛰므로 기존 설정과 도구 권한을 보존한다. 기존 등록은 카탈로그 갱신 뒤에도 남는다. 이름이 같은 MCP를 교체할 때는 기존 연결의 스코프와 설정을 확인하고 해당 연결을 제거한 뒤 다시 설치한다. 목록에서 빠진 서버를 직접 정리하려면 `codex mcp remove 서버이름` 또는 `claude mcp remove --scope 스코프 서버이름`을 사용한다. Claude의 로컬 스코프는 등록한 프로젝트 디렉터리에서 명령을 실행한다.

### 플러그인: 마켓플레이스 4개, 플러그인 28개

`src/plugins/plugins.yaml`에 정의된다.

- `claude-plugins-official` 25개: 언어 서버 12종, `pr-review-toolkit`, `code-review`, `skill-creator`, `playwright`, `context7`, `security-guidance`, `commit-commands`, `frontend-design`, `atlassian`, `sentry`, `chrome-devtools-mcp`, `discord`, `ralph-loop`
- `anthropic-agent-skills` 1개: `document-skills`로 xlsx/docx/pptx/pdf를 다룬다
- `openai-codex` 1개: `codex` 위임
- `obsidian-skills` 1개: `obsidian`

### 출력 스타일

`src/output-styles/hibi_default.md`는 한국어 응답, 영문 코드·명령어 유지, 마크다운 구조와 완료 보고 형식을 규정한다. Codex의 응답 서식은 `src/AGENTS.md`와 `technical-writing` 스킬에 있다.

### 루트 설정 파일

| 파일 | 설치 대상 |
|---|---|
| `src/CLAUDE.md` | Claude Code |
| `src/AGENTS.md` | Codex |
| `src/settings.json` | Claude Code. 기존 설정과 병합 |

`-ko.md` 접미사 파일은 개발자 가독용 한국어 미러다. 인스톨러 스캐너가 건너뛰고 `package.sh`가 번들에서 제거하므로 **설치되지 않는다**.

## 개발과 배포

저장소 루트에서 안정 버전 Rust로 개발한다. 기본 실행과 터미널 전용 실행은 다음 명령을 사용한다.

```bash
cargo run --locked --manifest-path tools/installer/Cargo.toml
cargo run --locked --manifest-path tools/installer/Cargo.toml --no-default-features -- --tui
```

인스톨러 배포 빌드는 macOS Universal, Windows x64 MSVC, glibc 2.34 이상을 사용하는 Linux x86_64, Alpine x86_64용 동적 musl로 나뉜다. 상태줄은 별도 크레이트로 빌드하고, 릴리스는 커밋된 바이너리를 패키지에 복사한다. 테스트 명령과 패키징 절차는 [운영 가이드](RUNBOOK.md)를 참고한다.

## 멀티소스 설정

번들 설정으로 시작하고 **소스 관리** 화면이나 `~/.hibi/sources.yaml`에서 추가 소스를 등록한다. 같은 ID가 여러 소스에 있으면 목록에서 마지막에 있는 소스가 우선한다. 번들 소스는 최저 우선순위다.

번들 디렉터리에는 `agents/`와 `settings.json`이 모두 있어야 한다. 사용자 소스에는 다음 중 하나 이상이 필요하다. `agents/`, `commands/`, `contexts/`, `rules/`, `skills/`, `hooks/`, `output-styles/`, `statusline/`, `mcps/mcps.yaml`, `plugins/plugins.yaml`, `settings.json`, `CLAUDE.md`, `AGENTS.md`.

Git 소스에는 HTTPS와 Git 실행 파일이 필요하다. 명시적으로 동기화하면 `auto_update: false`여도 fetch한다. 네트워크 오류 시 해당 URL과 브랜치의 캐시가 있으면 오래된 사본을 표시하고, 없으면 소스를 건너뛴다. Git 캐시는 `~/.hibi/cache/git/`에 있다. 입력 검증과 캐시 동작은 [소스 관리 기능맵](features/installer-sources.md)을 참고한다.

## 설치 이력 `install.json`

파일 컴포넌트의 설치·제거 후 목록을 갱신할 때 `~/.hibi/install.json`에 출처를 기록한다.

```json
{
  "source": "https://github.com/devsepnine/hibi_ai",
  "version": "v2.0.0",
  "target": ".claude",
  "updated_at": "2026-11-27T00:00:00Z",
  "components": ["agents/architect.md", "commands/qa-handoff.md", "skills/qa-handoff/SKILL.md"]
}
```

`components`의 ID는 타입 디렉터리 이름 뒤에 그 안의 상대 경로를 붙인 것이다. 번들 소스에서 온 것만 나열하고, 추가 소스는 `other_sources`에 분리된다. `pull-request` 스킬이 이 파일로 클론 없이 상류 저장소를 찾는다. 기록은 hibi 자신의 디렉터리에만 쓴다.

v1.22.0부터 기록에 `hashes`가 더 붙는다. hibi가 쓴 그대로의 파일 내용을 줄바꿈만 LF로 맞춰 잰 SHA-256이고, 키는 컴포넌트 ID다.

```json
{
  "hashes": {
    "agents/architect.md": "<SHA-256 hex>",
    "skills/qa-handoff/SKILL.md": "<SHA-256 hex>"
  }
}
```

- 번들 소스의 `agents`, `commands`, `contexts`, `rules`, `skills`, `output-styles` 파일만 담는다. 훅, 상태 표시줄, config 파일은 담지 않는다
- 스캔에서 소스와 같았던 파일은 설치본이 아니라 소스를 해시한다. 스캔 뒤에 사용자가 저장한 수정을 hibi가 쓴 것으로 기록하지 않기 위해서다
- 소스와 달랐던 파일은 같은 대상의 이전 기록에 있던 해시와 아직 같을 때만 그 해시를 유지한다. 옛 hibi가 기록한 내용 그대로라는 뜻이다. 그 밖의 파일은 해시가 없다
- `hashes`가 없는 옛 기록도 그대로 읽힌다

인스톨러는 시작할 때 이름을 바꾼 스킬을 정리한 다음 `tools/installer/src/fs/installer/unshipped.rs`로 기록에는 있지만 번들 소스가 더는 내보내지 않는 파일을 판정한다.

- 기록된 해시와 같으면 지운다. 그래서 비게 된 상위 디렉터리도 지우되 타입 디렉터리는 남긴다
- 해시가 다르면 사용자가 고친 파일로 보고 남긴다
- 해시가 없거나, 경로가 링크를 지나거나, 일반 파일이 아니면 확인할 수 없는 파일로 남긴다
- 기록에 없는 파일은 건드리지 않는다. 소스에 그 타입 디렉터리가 없으면 그 타입의 파일은 판정하지 않는다
- 판정한 ID는 `manifest::forget`이 install.json에서 지우므로 같은 알림은 한 번만 뜬다. 지우지 못한 파일은 기록에 남아 다음 실행에 다시 시도한다

결과는 목록 화면 상태 바의 `Auto-cleaned N outdated item(s)` 뒤에 `N file(s) no longer shipped`, `N file(s) no longer shipped could not be removed`, `kept N edited file(s) no longer shipped`, `kept N unverified file(s) no longer shipped`로 붙는다.

## 최근 변경사항

날짜별 항목은 당시 변경 기록이다. 현재 파일 위치와 실행 절차는 [문서 인덱스](INDEX.md)에서 찾는다.

### 2026-11-27, 인스톨러 개선

- GUI의 라이트·다크 테마와 헤더 여백을 조정했다. 대상 카드 탐색, 검색창 포커스, 필터 키보드 조작을 개선했다.
- GUI와 TUI에서 실제로 설치하거나 제거할 수 있는 항목 수를 표시한다. 제거 전에 이름과 범위를 확인하며, 체크나 경로가 달라지면 다시 확인한다.
- TUI는 짧은 창에서 시작 화면을 줄인다. 제거 목록을 스크롤할 수 있고 상태 메시지는 키 힌트와 버전을 남겨 둔 채 줄바꿈한다.

### 2026-11, v2.0.0

- 기본 실행을 iced GUI로 전환했다. `--tui`로 터미널에서 같은 설치 기능을 사용한다. GUI는 한국어, 영어, 일본어로 표시한다.
- macOS DMG와 업데이트용 ZIP, Windows MSI, GNU Linux와 Alpine 패키지를 제공한다. 앱은 직접 설치본의 업데이트 번들 서명을 확인한다.
- GUI와 TUI는 Claude 설정 항목 선택과 작업 결과를 공유한다. 릴리스 워크플로에 GUI, PTY, 패키지 설치·업그레이드와 업데이트 복구 검증을 추가했다. [v2.0.0 릴리스](https://github.com/devsepnine/hibi_ai/releases/tag/v2.0.0)

### 2026-11-09, v1.24.0

- 파일 목록에서 `f`로 전체, 변경된 파일, 외부 파일, 미설치 파일을 전환한다. `a`는 필터에 맞는 파일만 선택하며 설치와 제거도 현재 필터 범위에 적용한다
- statusline 설치와 설정 병합에서 다른 운영체제의 기본 명령을 현재 플랫폼에 맞게 보정한다. 사용자 정의 경로와 인수가 있는 명령은 유지한다
- macOS와 Fedora에서 필터 선택, 실제 파일 설치와 제거, 설치 순서별 statusline 등록을 검증했다

### 2026-11-09, v1.23.4

- 코드, 빌드, 테스트, 개발 서버 실행 전에 `rundebugconfig` MCP 연결을 확인하고 저장된 실행 설정과 세션을 우선 사용하도록 공통 지침을 보강했다
- `verification-loop`에 설정 선택, 출력과 종료 상태 확인, 중복 실행 방지, MCP를 사용할 수 없을 때의 대체 실행 절차를 추가했다

### 2026-11-06, v1.23.3

- `qa-handoff`에 번역 변경사항을 추가했다. 번역 키, 언어, 변경 전후 문구, 사용 위치를 정리하고 치환 변수와 대소문자를 그대로 유지한다
- 번역 리소스 반영, 시트 등록, 실제 화면 확인 상태를 구분한다. 확인하지 못한 번역은 임의로 작성하지 않는다

### 2026-11-06, v1.23.2

- `why`가 회귀로 판단하기 전에 오류 내용, 실패 단계, 깨진 동작이 이전 문제와 같은지 확인한다
- `qa-handoff`가 전체 요약과 변경 전후 비교를 먼저 보여 주고, QA 체크리스트를 전제·재현 절차·기대 결과로 정리한다. 내부 유지보수 목록과 우선순위 표시는 제외하고 회귀 확인, 자동 검증 결과, 용어 설명, 개발자 확인 사항을 유지한다
- API 계약이 바뀌면 변경된 값과 이를 사용하는 화면을 함께 설명한다. 실제 서버 연동 여부와 테스트용 응답으로 확인한 결과를 구분하고, Jira 수정 시 구현 티켓과 기존 내용을 확인한다
- 로컬 스킬 평가 산출물은 `skill-evaluations/`에 보관한다. 배포할 스킬과 평가 결과를 분리해 소스 구조 검사에 섞이지 않도록 한다

### 2026-10-17, v1.23.1

- `feature-map` 6단계 검사가 zsh에서 아무것도 보고하지 않던 문제를 고쳤다. zsh는 따옴표 없는 `$MAP`, `$SRC`를 단어로 나누지 않아 `grep`이 실패했고, 네 검사가 모두 빈 출력으로 끝나 통과처럼 보였다. 검사 블록을 `bash <<'EOF'`로 감싸 호출한 셸과 상관없이 bash에서 돈다
- 같은 검사가 확장자를 다섯 글자까지만 잡아 `.svelte`, `.coffee` 경로를 건너뛰던 것을 열 글자로 넓혔다

### 2026-10-12, v1.23.0

- `feature-map`과 `architecture-rules`가 포인터 줄을 프로젝트 루트 `AGENTS.md` 한 곳에만 쓴다. 파일이 없으면 만들고, `CLAUDE.md`는 새로 만들지 않는다. Claude Code는 루트에 `CLAUDE.md`가 없을 때 `AGENTS.md`를 읽기 때문이다. 루트 `CLAUDE.md`가 이미 있으면 `@AGENTS.md` 한 줄을 덧붙이고, 이전 실행이 남긴 포인터 줄을 지운다. 두 스킬에 이 경우를 다루는 eval을 하나씩 더했다
- 이 저장소의 개발용 지침을 루트 `AGENTS.md` 하나로 합치고 루트 `CLAUDE.md`를 지웠다. Codex와 Claude Code가 같은 파일을 읽는다. `lint-prose.py`의 기본 검사 대상도 `CLAUDE.md` 대신 `AGENTS.md`가 됐다. 배포되는 `src/CLAUDE.md`는 그대로다. 사용자 레벨 `~/.claude/`에는 이 대체 동작이 없다

### 2026-10-04, v1.22.0

- 인스톨러가 번들 소스에서 빠진 파일을 정리한다. `~/.hibi/install.json`에 hibi가 쓴 파일마다 SHA-256을 남기는 `hashes`를 더했고, 시작할 때 `fs/installer/unshipped.rs`가 기록에는 있지만 소스가 더는 내보내지 않는 파일을 그 해시와 대조한다. 같으면 지우고, 사용자가 고쳤거나 확인할 수 없는 파일은 남긴 채 상태 바에 알린다. 기록에 없는 파일은 건드리지 않고, 판정한 ID는 기록에서 지워 알림이 한 번만 뜬다. 해시는 새 직접 의존성 `sha2`로 잰다

### 2026-10-04, v1.21.0

- `how` 스킬을 추가했다. 스킬은 29 → 30. 서브시스템이나 흐름이 런타임에 어떻게 동작하는지 실제 호출 체인을 따라 설명하고 주장마다 파일을 인용한다. 코드가 왜 그런 모양인지는 계속 `why`가 다룬다
- `/review-panel` 커맨드를 추가했다. 커맨드는 27 → 28. A/B 티어 diff를 관점 하나씩 맡은 `code-reviewer` 에이전트 패널로 검토하고 결과를 Act on, Consider, Dismissed와 판정으로 합친다. 수정, 커밋, 푸시는 하지 않는다. 기본 리뷰 게이트는 계속 `/code-review`다
- 스킬 `iced_rs`, `ratatui_rs`의 이름을 `iced-rs`, `ratatui-rs`로 바꿨다. 인스톨러는 시작할 때 `fs/installer/renamed.rs`로 옛 디렉터리를 정리한다. 설치 기록에 있는 파일만 지우고, 사용자 파일과 그 파일이 든 디렉터리는 남기며, 링크는 따라가지 않는다. 소스에 새 이름이 있을 때만 정리한다
- 스킬 하위 구조를 표준으로 고정했다. `deploy-to-vercel/resources/`는 `scripts/`로, 벤더링 규칙 세트 4개의 `README.md`는 `rules/_README.md`로 옮겼다
- 인스톨러 모듈을 정리했다. `TargetCli`를 `app`에서 `target.rs`로 옮겨 `app`과 `fs`의 순환을 끊었다. 홈 디렉터리는 `paths.rs` 한 곳에서만 얻고, git을 포함한 짧은 외부 명령은 `exec.rs`의 공용 실행기 하나로 돌린다
- 인스톨러 크레이트에 rustfmt를 적용했다. 포맷으로 줄이 늘어서 `arch-file-size`는 테스트를 뺀 프로덕션 코드만 잰다
- 릴리즈 워크플로가 빌드 전에 `lint-prose.py`, `lint-arch.py`, `cargo fmt --check`, `cargo test`를 실행한다. `lint-arch.py`가 상태 표시줄 소스와 바이너리의 커밋 이력을 비교하므로 checkout은 전체 이력을 받는다. `lint-arch.py`의 검사 규칙은 9개에서 12개가 됐고, 늘어난 셋은 홈 디렉터리 접근 지점, 상태 표시줄 바이너리 갱신, 스킬 구조다
- 이 저장소를 개발할 때 읽는 루트 `CLAUDE.md`를 추가했다. 배포되지 않는다. 기능맵과 아키텍처 규칙을 가리키는 줄, `-ko.md` 미러 규칙, 커밋 전에 돌릴 검사 네 개를 담는다. `lint-prose.py`의 기본 검사 대상에 이 파일을 더했다
- `feature-map`과 `architecture-rules`가 포인터 줄을 제안하는 대신 프로젝트 루트 `CLAUDE.md`에 바로 쓴다. 파일이 없으면 만들고, `AGENTS.md`가 있으면 같은 줄을 넣는다. 기능맵 템플릿은 제목과 필드 라벨을 영어로 통일했다. 단계 6 검사가 그 라벨을 grep하기 때문이다. 산문은 프로젝트 문서의 언어로 쓴다
- `/checkpoint`에 결정 로그 형식을 정했다. 장시간 실행, `/perf`, 변경을 시도하고 유지하거나 되돌리는 루프가 `.claude/resume/<name>-decisions.tsv`에 시도마다 한 줄씩 같은 열로 남긴다
- `eval-harness`에 Claude Code 세션 안에서 중첩 `claude -p`가 시작하지 않아 모든 행이 INCONCLUSIVE가 되는 경우와 그 대처를 적었다
- 인스톨러 테스트가 137개에서 151개가 됐다

### 2026-10-04, v1.20.0

- `architecture-rules` 스킬과 `/architecture-rules` 커맨드를 추가하고 정책 라우팅 표에 올렸다. 스킬은 28 → 29, 커맨드는 26 → 27. 기존 코드베이스의 import 그래프, 배치, 이름에서 규칙을 찾고 규칙마다 개수와 측정 명령을 붙인다. 패턴이 섞였거나 의도를 코드로 알 수 없는 회색 지대는 근거와 권장 답을 보여 주고 묻는다. 결과물은 `docs/ARCHITECTURE.md`다. 기계적인 `MUST` 규칙은 lint 설정 초안으로 내고 사용자가 동의해야 적용한다. 결합이 건강한지는 계속 `dependency-design`이 판단한다
- 이 저장소의 `docs/ARCHITECTURE.md`를 추가했다. 인스톨러와 상태 표시줄 크레이트, 배포 설정에 대한 규칙 19개가 각각 수준, 근거와 측정 명령, 예외, 강제 수단을 갖는다. 남은 마이그레이션은 Known violations 표에 있다. `TargetCli`를 `app` 밖으로 옮기기, git 실행을 `fs`로 모으기, 홈 디렉터리 접근 지점 하나로 모으기, 스킬 구조와 이름 정리가 그 내용이다
- `tools/lint-arch.py`를 추가했다. `docs/ARCHITECTURE.md`의 기계적 `MUST` 규칙 9개를 검사하고 Known violations 표를 허용 목록으로 읽는다. 표에 있지만 더는 위반하지 않는 행은 stale로 보고하므로 수정이 진행될수록 표가 줄어든다. 새 위반이나 stale 행이 있으면 exit 1이다
- `tools/lint-prose.py`가 220자를 넘는 `description`을 실패로 처리한다. folded와 block 값도 재고, 짝이 맞는 따옴표만 벗긴 뒤 잰다. 한도를 넘던 `blast-radius`, `technical-writing`, `qa-handoff` 스킬과 `assurance-auditor` 에이전트의 설명을 트리거 어휘를 유지한 채 줄였다
- 기능맵 `docs/FEATURES.md`를 추가했다. 인스톨러 TUI, 상태 표시줄, 배포, 설정 동작을 화면 위치, 사용자 표현, 실제 UI 문구로 찾는다. 화면별 항목은 `docs/features/` 아래에 나눴다
- MCP env 입력 대화상자가 수집한 값을 가릴 때 앞 4바이트를 잘라서, 한국어처럼 다중 바이트 문자로 시작하는 값에서 `byte index 4 is not a char boundary`로 패닉했다. 앞 4문자를 쓰도록 고쳤고 ASCII 값은 이전과 같게 보인다
- 설치 때 상태 표시줄 자동 등록이 `statusLine`을 파일 이름 문자열로 써서, `{type, command}` 객체를 읽는 Statusline 탭이 새 설치에서 `[No default set]`를 보였다. 자동 등록도 `set_statusline`을 거쳐 같은 객체 형태로 쓴다. 옛 설치가 남긴 객체가 아닌 값은 교체하고, 사용자가 이미 설정한 객체는 유지한다

### 2026-10-04, v1.19.0

- `blast-radius` 스킬과 `/blast-radius` 커맨드를 추가했다. 머지 전에 diff가 자기 바깥에서 깨뜨릴 수 있는 것을 찾고, 변경이 안전하다는 근거가 되는 사실 하나를 실제 코드를 실행해 증명한다. 증명하지 못하면 `UNPROVEN`으로 표시한다
- `technical-writing` 스킬을 추가하고 정책 라우팅 표에 올렸다. 문서마다 Diátaxis 모드 하나, 쉬운 직설 문장, AI 문체 삭제 목록, 산문의 대시와 괄호 금지를 다룬다. 구두점 규칙은 `tools/lint-prose.py`가 강제한다
- 저장소 전체 산문을 이 규칙으로 다시 썼다. 대상은 `src/`, `docs/`, 루트 `README.md`다. 대시를 콜론으로 바꾸면 frontmatter `description`에 `: `가 남아 YAML이 중첩 매핑으로 읽으므로 그런 값은 따옴표로 감쌌다
- `tdd-workflow`에 단언 강도 섹션을 추가했다. 테스트 대상이 `undefined`를 반환해도 통과하는 테스트는 동작을 관찰하지 않으므로 다시 쓰거나 지운다
- `verification-loop`에 Phase 7 실제 산출물 확인을 추가했다. Phase 1부터 6은 대리 지표이므로 Phase 7을 포함한 실행만 `Overall: READY`를 보고한다
- `coding-standards`의 Type Safety를 보강하고 Boundaries 섹션을 신설했다. 외부 입력은 들어오는 지점에서 한 번 도메인 타입으로 파싱하고, 안쪽에서는 타입을 믿는다
- `code-reviewer`에 보고 전 필터를 추가해 가설, 취향, 이 변경이 만들거나 드러내지 않은 기존 스타일 문제를 걸러낸다. `architect`에는 shallow module, 정보 누출, 시간순 분해, pass-through 메서드 red flag를 추가했다
- `src/CLAUDE.md`·`src/AGENTS.md`에 원칙 줄을 더했다. 같은 가정을 공유하는 수정이 두 번 실패하면 그 가정부터 검증한다. 완료 전에는 대리 지표가 아니라 실제 산출물을 확인하고, 검증 주장마다 measured / inferred / guess 라벨을 붙인다
- `/bugfix`, `/refactor`, `/perf` 커맨드를 추가했다. 커맨드는 23 → 26. `/bugfix`는 보고된 화면에서 버그를 재현하고 런타임 증거로 원인을 좁힌 뒤 같은 화면에서 수정을 증명한다. `/refactor`는 동작을 먼저 고정하고 추가하기 전에 삭제하며, 읽는 부담이 줄어들 때만 결과를 남긴다. `/perf`는 기준선을 먼저 재고 변경 하나마다 측정 하나로 유지와 되돌림을 정한다
- `why` 스킬을 추가했다. 스킬은 27 → 28. 코드가 왜 이런 모양인지 git, PR, 티켓, 문서에서 추적하고 주장마다 `references/epistemics.md`의 신뢰 등급을 붙인다. 무엇이 깨질지는 `blast-radius`가 다루고, 이 스킬은 왜 그렇게 만들었는지를 다룬다
- `/checkpoint`에 `pause`·`resume`과 장시간 무인 실행 섹션을 추가하고 `haiku`/`low`에서 `sonnet`/`medium`으로 옮겼다. `pause`는 안전한 경계에서 멈추고 `.claude/resume/<name>.md`에 재개 노트를 쓰며, 커밋도 stash도 하지 않는다. `resume`은 물려받은 완료 주장을 실제 산출물에서 다시 검증한다. 무인 실행은 확인 가능한 종료 조건을 먼저 정하고, 그 조건을 완화해서 끝내지 않는다
- `/plan` 산출물에 데이터 형태 우선 항목과 처리량 체크포인트를 추가했다. 체크포인트는 선행 단계, 독립 작업 흐름, 공유 가변 상태, 가장 작은 안전한 분해의 네 항목이다
- `/learn`은 새 스킬을 만들기 전에 먼저 라우팅한다. 린트, 훅, 타입, 테스트, 스크립트로 강제할 수 있으면 그 검사를 제안하고, 주제를 이미 가진 스킬이 있으면 그 스킬의 수정을 제안한다. 둘 다 아닐 때만 새 스킬 초안을 쓴다
- `/orchestrate`의 `refactor` 체인을 `architect` → `code-reviewer` → `tdd-guide`에서 `tdd-guide` → `architect` → `code-reviewer`로 바꿨다. 구조를 바꾸기 전에 `tdd-guide`가 동작을 고정한다. 체인 안에서 `bugfix`는 `/bugfix`의 단계를, `refactor`는 `/refactor`의 단계를 따른다
- `dependency-design`에 규칙 둘을 추가했다. `dependency-subtract-before-add`는 새 코드를 얹기 전에 죽은 코드와 pass-through 계층을 먼저 지운다. `dependency-migrate-then-delete`는 모든 호출자를 옮기고 옛 API를 같은 변경에서 지운다

### 2026-10-02, v1.18.0

- Linux 설치 경로 둘을 추가했다. `install.sh`는 체크섬을 검증한 뒤 `~/.local`에 설치하는 curl|sh 인스톨러이고, `nfpm.yaml`은 `.deb`·`.rpm`·`.apk`를 한 설정에서 만든다. 릴리즈 워크플로가 nfpm을 설치하고 세 패키지를 함께 올리며, `checksums.txt`가 여섯 산출물을 모두 덮는다
- 상태 표시줄이 너비를 넘는 줄을 자르지 않고 터미널 너비에서 줄바꿈한다. Claude Code가 설정하는 `COLUMNS`를 읽고 세그먼트 경계에서만 끊는다. `COLUMNS`가 없으면 출력은 이전과 같다

### 2026-09-18, v1.17.1

- 커밋 권한 규칙이 `commit`만 덮고 있던 비대칭을 닫았다. `src/CLAUDE.md`의 절대 규칙 섹션이 `commit`·`push`·`gh pr create` 셋을 함께 다루고, 제목도 "Absolute commit and push rules"가 됐다. §8 Git 안전 항목은 이 섹션을 가리킨다. 같은 규칙을 두 곳에서 서로 다른 강도로 말하지 않도록 하기 위해서다
- **포괄적 사전 승인은 없다**는 조항을 신설했다. 이전의 "알아서 해줘", 직전 커밋·푸시에 대한 승인, 승인받은 계획, 명령을 자동 승인하는 권한 모드는 어느 것도 요청이 아니다. 실패 양상이 "규칙이 없어서"가 아니라 "과거 승인을 현재 승인으로 읽어서"였기 때문에, 금지 문장보다 승인의 유효 범위를 못 박는 쪽이 실효가 있다
- 슬래시 커맨드의 승인 범위를 명시했다. **그 턴에 한해**, 커맨드가 지칭하는 행위 하나까지다. `/commit`은 그 커밋까지고 `/pull-request`·`/upstream-pr`은 push도 `gh pr create`도 승인하지 않는다. 이는 `commands/pull-request.md:15`·`pull-request/references/upstream-config.md:66`이 이미 요구하던 두 번째 확인과 일치한다. 명시하지 않으면 "커맨드를 불렀으니 푸시도 승인됐다"로 읽힌다
- 리뷰에서 초안 결함 둘을 걷어냈다. 첫째, 초안은 `/pull-request`·`/upstream-pr`이 "브랜치 준비와 커밋까지 승인"한다고 썼는데, 그 근거는 `/upstream-pr` 워크플로에만 있는 `upstream-config.md:66`의 문장이었고 커밋 SSOT인 `commit-rules/SKILL.md:42`에는 없는 예외였다. 상시 계층이 SSOT보다 느슨한 규칙을 말하게 되므로 그 주장을 삭제했다. 둘째, "방금 입력한"이 유효기간을 정하는 유일한 표현인데 정의되지 않아, 세션 초반의 커맨드 한 번이 계속 승인으로 읽힐 수 있었다. 규칙이 막으려던 이월과 같은 형태여서 "그 턴에 한해"로 닫았다
- 규칙을 조용히 무력화하던 경로 3곳을 함께 닫았다. 규칙 텍스트만 강화하면 이 경로들이 그대로 남기 때문이다. `commands/checkpoint.md:22`는 "stash 또는 commit"을 정규 2단계로 지시했다. `model: haiku`·`effort: low`라 되묻지 않고 실행되는 경로다. `/checkpoint`가 지칭하는 것은 체크포인트이지 커밋이 아니므로 stash를 기본으로 하고 커밋에는 별도 요청을 요구한다. `deploy-to-vercel/SKILL.md:130,158`은 같은 파일 59행의 push 승인 게이트로 되돌리는 참조 없이 `commit and push`·`git push`를 단계로 지시했다. 특히 130행은 112행의 "별도 확인을 요청하지 않는다"가 linking에만 한정된 것임에도 그 연장선으로 읽혔다. 두 곳 모두 승인 게이트를 명시했다. `commands/learn.md:98`은 전역 지침이 같은 문장에 달아둔 "사용자가 동의한 뒤에만"이 빠져 있어 복원했다. 각 `-ko` 트윈 동반
- 손대지 않은 항목: 번들 플러그인 `commit-commands`의 `/commit-push-pr`은 커맨드 이름 자체가 커밋·푸시·PR 셋을 지칭하므로 새 규칙인 "커맨드가 지칭하는 행위까지 승인" 하에서 호출이 셋 모두에 대한 요청이 된다. 구조적 충돌이 아니다. 외부 마켓플레이스 항목이라 `src/` 안에 지시문 텍스트도 없다
- 전파 지점 8곳을 함께 맞췄다. `src/CLAUDE.md`의 §8과 절대 규칙 섹션, `src/AGENTS.md` §6, `src/skills/commit-rules/SKILL.md:42`, `src/skills/pull-request/SKILL.md` §4 및 각 `-ko` 트윈이다. "자동 승인 권한 모드도 요청이 아니다" 조항은 처음에 `CLAUDE.md`에만 넣었는데, `AGENTS.md`는 `CLAUDE.md`를 로드하지 않는 별개 계통이어서 자동 승인 모드로 도는 Codex 세션에 구멍이 남았다. 그래서 네 계층 전부에 넣었다. 얇은 진입점인 `src/commands/commit.md:12`·`commands/pull-request.md:15`는 이미 같은 문턱을 명시하고 있어 대상이 아니다

### 2026-09-15, v1.17.0

아래 2026-09-09부터 2026-09-14까지의 항목도 이 릴리즈에 들어갔다. 그 항목에 없는 변경은 다음과 같다.

- 인스톨러 창을 번호로 지정한다. `1`은 탭 바, `2`는 콘텐츠 창으로 바로 가고 `Tab` 순환은 그대로다. 포커스를 가진 창은 강조 테두리를 받는다. 80열에서 잘리던 상태 표시줄 도움말을 짧은 한 줄로 줄이고 전체 키 표는 `?` 오버레이로 옮겼다. 목록에서 `Esc`는 CLI 선택 화면으로 돌아간다
- 탭 바 절삭 예산과 상태 표시줄 우측 정렬을 바이트가 아니라 화면 열 너비로 잰다. 화살표 같은 다중 바이트 글리프와 CJK 문자에서 어긋나던 정렬을 고쳤다
- `eval-harness`에 실행 스크립트 둘을 넣었다. `trigger_eval.py`는 스킬 설명이 실제로 자동 트리거되는지 재고, `skill_budget.py`는 스킬 목록 문자 예산을 재서 넘으면 exit 1로 끝난다. 스킬 7개에 트리거 회귀 세트 `trigger-eval.json`을 추가했다
- 주석 유지 규칙을 `coding-standards`, `code-reviewer`, `src/CLAUDE.md`, `src/AGENTS.md`에 넣었다. 추출이나 분리 때 주석을 코드와 함께 옮기고, 이름을 바꾸면 diff 바깥까지 옛 이름을 grep하고, 주장하는 이유는 검증한다
- `package.sh`가 컴포넌트 디렉터리를 복사하기 전에 `dist/`의 옛 사본을 지운다. `src/`에서 지운 파일이 반복 패키징에서 릴리즈 아카이브에 남을 수 있었다

### 2026-09-14

- `commit-rules`에서 조직 하드코딩 제거. 예컨대 `[PP-XXXX]`의 PP-6050이 배포되는 공개 설정에 박혀 있었다. `pull-request` §1과 같은 규칙으로 교체했다: 브랜치와 `git log`에서 추출하고, 어느 쪽에도 없으면 접두사를 완전히 생략한다. `[TICKET-1]` 같은 자리표시자는 없는 것보다 나쁘다
- 같은 수정으로 컨벤션이 자기 저장소와 어긋나던 문제가 닫혔다. 형식이 `<type>: [<ticket-number>] <title>` 하나뿐이어서 티켓을 필수로 요구했는데, 이 저장소 최근 커밋 30개 중 접두사를 가진 것은 0개였다. 즉 규약을 지킨 커밋이 하나도 없는 상태였다. 형식 블록이 티켓 있는 경우와 없는 경우 두 줄을 함께 보여준다
- 전파 지점 6곳을 함께 맞췄다. `src/skills/commit-rules/SKILL.md:13-14,31-38`, `src/commands/commit.md:13`, `src/CLAUDE.md:153` 및 각 `-ko` 트윈이다. `src/AGENTS.md`는 커밋 형식을 갖고 있지 않아 대상이 아니다. CLAUDE/AGENTS는 중복시키지 않는 구조이기 때문이다
- 티켓 추출의 결정 불가 구간을 닫았다. 브랜치엔 티켓이 없고 이력에만 접두사가 있으면 "둘 다 없으면 생략"이 발동하지 않아 무관한 커밋의 번호를 가져다 붙일 수 있었다. 트래커 쓰는 저장소의 `fix/typo` 브랜치나 main 직접 커밋이 그 예다. 남의 실재 티켓은 형식상 유효해 보여서 자리표시자보다 잡기 어렵다. `pull-request/SKILL.md:44`가 base 브랜치에 이미 쓰던 관용구인 "둘이 어긋나면 말하고 묻는다"를 티켓에도 적용했다: **판단은 브랜치가 한다**. `commit-rules`와 `pull-request` §1 양쪽을 같은 커밋에서 고쳤다. 한쪽만 고치면 "같은 규칙" 주장이 깨진다

### 2026-09-11

- `upstream-pr` 스킬을 `pull-request`로 통합했다. 스킬은 25 → 24. 실체가 PR의 변종이 아니라 설정 기여 판단 워크플로였고, PR 메커닉은 6개 섹션 중 하나뿐이었다. 방법론은 `pull-request/references/upstream-config.md`로 이동, `/upstream-pr` 커맨드는 진입점으로 유지. `src/CLAUDE.md`의 정책 라우팅 행이 두 진입점을 모두 명시한다. `AGENTS.md`는 Codex용이라 커맨드를 쓰지 않으므로 스킬 이름만 가리킨다
- 통합 과정에서 경계 결함 정리: 단방향 위임이라 역참조 `Related` 표가 없던 점, 티켓 없는 설정 PR에 `[TICKET-ID]`를 강제하던 모순, 권한 규칙의 두 버전인 베이스와 공개 상류 2단 확인의 관계 미명시, 일반 스킬에 없던 브랜치 네이밍 규약
- `pull-request` 스킬에서 조직 하드코딩 제거: `ggnetwork.atlassian.net`, `PP-XXXX`, `upstream/develop`. 규약을 기억이 아니라 저장소에서 읽는다. base 브랜치는 `gh`로 확인, 티켓은 브랜치·히스토리에서 추출하되 없으면 접두사 생략, 본문은 `.github/pull_request_template.md`가 무조건 우선한다. `commit-rules`와 `/commit`의 같은 하드코딩은 이번 범위를 벗어나 후속으로 미뤘고 2026-09-14 항목에서 정리했다
- `gh pr create` 메커닉인 `--draft`/`--base`/`--body-file`과 기존 PR 리뷰 절차를 신규 추가했다. 종전 description이 리뷰를 주장했으나 체크리스트만 있었다
- `pull-request/evals/`를 추가했다. 구 `upstream-pr` 트리거 세트를 옮길 때 네거티브 2건이 포지티브로 반전됐는데, 디렉터리명 기준 매칭이기 때문이다
- `pull-request`에 §2 "본문은 diff에서 쓴다"를 신규 추가했다. 점 세 개를 쓰는 `git diff <base>...HEAD`로 읽고, 본문과 hunk를 양방향으로 대응시켜 미대응 항목을 범위 이탈 또는 허구로 잡아낸다. 간결함의 정의를 "짧게"가 아니라 "diff가 보여줄 수 없는 것만"으로 못박았다
- §5 리뷰에 판단 순서와 **필요성 검증**을 추가했다. 판단 순서는 본문-diff 대응, 필요성, 정확성, 테스트다. 필요성 검증은 삭제 테스트, 호출자 수, 기존 구현 grep, 실행될 일 없는 가드, 목적 무관 hunk를 본다. 필요성은 판결이 아니라 질문이므로 판단이 안 되면 작성자에게 묻는다
- §6 "리뷰 코멘트를 처리한다"를 신규 추가했다. 코멘트를 지시가 아니라 분류할 주장으로 다룬다. 6분류 표는 범위 내 블로킹, 스타일, 기존 문제, 새 요구사항, 이미 처리됨, 잘못된 전제이고, 범위 테스트는 "이 PR이 없었어도 필요했나"다. n차 라운드는 코멘트·커밋 타임스탬프 비교로 이미 처리된 지적을 걸러낸다
- "추측하지 말고 묻는다"를 도입부 원칙으로 세우고 §2·§5·§6이 각각 구체적 트리거를 갖게 했다. 답이 쓸 내용을 바꿀 때만 묻고, 그렇지 않으면 가정을 본문에 적는다. `description`은 `리뷰 코멘트 반영` 어휘를 얻어 216자이고 한도는 220자다
- 통합이 만든 댕글링 지시문을 정리했다. `"run /upstream-pr"` 형태의 지시는 모델이 존재하지 않는 `Skill{upstream-pr}` 호출로 해석하며, 트리거 측정에서 3회 재현됐다. `src/CLAUDE.md:36`·`commands/learn.md:98`·`commands/upstream-pr.md:25`가 모두 커맨드임을 밝히고 로드할 스킬 이름 `pull-request`를 명시하도록 고쳤다. 같은 파일의 방법론 참조는 삭제된 `upstream-pr` 스킬에서 `pull-request` §1부터 §7로 재지정했다
- 사후 리뷰 findings를 반영했다. 첫째, 라우팅 대상 이름을 정정했다. `code-review`는 스킬로 존재하지 않고 실체는 `/code-review` 커맨드와 `code-reviewer` 에이전트다. `SKILL.md`·`SKILL-ko.md`의 §5c와 Related 표, `evals.json` #3의 expected_output을 함께 고쳤다. 둘째, §6 코멘트 조회에 `--paginate`를 추가했다. REST 한 페이지가 30건에서 끊겨 "해결된 라운드까지 스레드 전체를 읽는다"는 전제와 배치됐고, n차 리뷰에서 이미 처리된 지적을 놓치는 무음 실패가 된다. `per_page=2`로 실증했으며 플래그 없이 2건, 붙이면 3건이었다. 셋째, KO 표현 2건을 정정했다. "싼 질문"은 "가장 저렴한 질문"으로, "발화할 수 없는 가드"는 "실행될 일 없는 가드"로 바꿨다. 넷째, 통합 때 유실된 `/learn` 역포인터를 Related 표에 복구했다. 순방향 `learn.md:98` → `/upstream-pr`만 남아 있었다. 다섯째, §6의 커밋 조회를 `gh pr view --json commits`에서 `gh api --paginate .../pulls/<n>/commits`로 교체했다. 전자는 100건에서 끊기고 **가장 오래된** 100건을 주므로 긴 PR에서 head 커밋이 빠진다. kubernetes/kubernetes#141727로 실증했는데, 반환 100건의 마지막이 `3e8a81e`인데 실제 head는 `9c3b2d0`였다. n차 라운드 판별이 최근 커밋 타임스탬프에 걸려 있으므로 무음 오판이 된다. REST 페이지네이션은 같은 PR에서 head를 포함한다
- `description`은 고치지 않았다. "PR 템플릿" 어휘 손실과 설정 변경 트리거 갭이 지적됐으나 두 질의인 `이 저장소 PR 템플릿 …`과 `이번 세션에서 배운 규칙을 배포 설정에 반영해줘` 모두 격리 리그에서 트리거되어 전제가 재현되지 않았다. 결과는 2/2 PASS였다. 216/220자에서 재작성하는 대신 그 두 질의를 회귀 세트에 고정했고, 트리거 케이스는 9 → 11, true는 6 → 8이 됐다
- 2차 사후 리뷰로 diff 전체를 독립 검토해 findings 7건을 반영했다. 첫째, 이 변경 이력 자체의 허위 주장을 삭제했다. `commands/upstream-pr.md:25`의 `§1–§5` 참조를 §1부터 §7로 "정정"했다고 적었으나 HEAD의 그 줄에는 § 참조가 없었고, 삭제된 `upstream-pr` 스킬을 SSOT로 가리키는 문장이었으며 구 스킬은 5섹션도 아니었다. 작업 중 초안 수정을 HEAD 대비 정정으로 오기한 것이므로 "방법론 참조 재지정"으로 바꿨다. `docs/INDEX.md:144`도 동일하다. 둘째, §6 코멘트 조회 jq에 `\(.created_at)`을 추가했다. "어떤 커밋보다 먼저 쓰인 코멘트는 이미 처리됐을 수 있다"는 §6의 근거와 `evals.json` #10이 코멘트·커밋 타임스탬프 비교를 요구하는데 조회는 `path:line`·`user`·`body`만 투영해, 스킬을 그대로 따른 모델이 비교할 입력을 갖지 못했다. 필드 실재와 형식은 실측 확인했다. `cli/cli#12444`는 `2026-01-08T01:09:41Z`이고 `.commit.committer.date`와 같은 ISO-8601이라 직접 비교된다. 두 조회가 모두 타임스탬프로 시작하게 되어 본문 설명도 그에 맞췄다. 셋째, `evals.json` #8의 과잉 수용을 제거했다. "명시적으로 라벨링한 가정으로 기록"도 통과시켰는데, magic number와 skip된 테스트는 그 *이유*가 본문에 쓸 내용을 바꾸므로 스킬 자체 논리인 "답이 쓸 내용을 바꾸지 않을 때만 가정을 적는다"로는 물어야 한다. 라벨 붙인 추측을 통과시키던 절이었다. 넷째, 폴백 템플릿의 `perf`를 삭제했다. "커밋 type과 일치하는 하나"라고 하면서 `commit-rules`에 없는 type을 제시했다. 두 트윈 모두다. 다섯째, KO 조사 오류를 정정했다. `SKILL-ko.md:167` "hunk가 가장 빨리 찾을 수 있고"는 hunk를 찾는 주체로 만든다 → "hunk는 가장 빨리 찾아낼 수 있고". 여섯째, 과장 표현을 완화했다. "셸 보간을 온전히 통과하지 못한다"는 제대로 인용하면 통과하므로 반증 가능하다 → "셸 인용 과정에서 쉽게 망가진다". 일곱째, 트윈 마크업 비대칭 1건을 해소했다. `upstream-config-ko.md:4`의 볼드를 제거했다

### 2026-09-09, v1.16.0 이후

**`src/` 마크다운 전면 재개편**: 104 파일, +1,057 / −21,188

- 스킬 25개 `description`을 4부 구조로 재작성했다. 4부는 무엇, `Use when` 트리거, 한국어 어휘, `NOT for`다. 목록 예산을 8,754 → 5,091자로 축소해 설명 절삭 버그를 해소했다
- 벤더링 스킬 4개인 `composition-patterns`, `dependency-design`, `react-best-practices`, `react-native-skills`에서 생성물 `AGENTS.md`·상류 스캐폴딩을 제거했다. 줄 수로는 −17,857줄이다. `rules/`가 원본이므로 정보 손실은 없다
- `agents/` 6개를 전면 재작성했다. 타 프로젝트 사양인 암호화폐 마켓·Privy·Solana·Supabase·Redis를 제거하고 스킬에 위임한다
- `commands/` 정리, `keywords:` 안내를 검증된 예산 규칙으로 교체
- `src/CLAUDE.md`/`AGENTS.md`를 압축하고 `src/contexts/`·`src/mcp.md`를 삭제했다. 설치되지 않거나 읽히지 않는 죽은 파일이다

### 2026-08 ~ 2026-09, v1.13.0 ~ v1.16.0

- 릴리즈 자동화: 태그 푸시 → GitHub Actions 전 플랫폼 빌드·발행
- 설치 provenance 매니페스트 `~/.hibi/install.json`과 `upstream-pr` 플로우
- `do-178c`, `qa-handoff`, `dependency-design` 스킬 및 `/do-178c`, `/qa-handoff`, `/deps` 커맨드 추가
- 모델 정책을 Claude 5 계열로 이관
- `package.sh`가 릴리즈 번들에서 `-ko` 파일 제외
- 사후 리뷰인 `code-reviewer`를 완료 보고 전 필수 게이트로 승격

### 2026-06 ~ 2026-08, v1.9.x ~ v1.12.0

- 멀티소스 지원 `~/.hibi/sources.yaml`, 소스 위저드, `hibi --sync`
- Windows `CreateProcessW`가 PATHEXT를 무시해 npm-shim `.cmd`가 안 보이던 문제를 수정했다. v1.9.7
- 상류 history rewrite에 대한 sync 복원력: shallow는 `reset --hard FETCH_HEAD`, full clone은 `merge --ff-only`. v1.9.9
- 정책을 `rules/`에서 스킬로 통합 이관

### 2026-02 ~ 2026-06, v0.1.3 ~ v0.1.6

- 인스톨러 모듈 재편: `installer.rs`는 857줄에서 `fs/installer/`로, `scanner.rs`는 672줄에서 `fs/scanner/`로, `app.rs`는 931줄에서 `app/`로 옮겼다. 파일당 300줄 목표를 도입했다
- 보안: `shlex::split()` 도입, scanner·`copy_file`에서 `..` 컴포넌트 거부
- macOS Universal Binary는 전 컴포넌트에 적용하고, 버전별 릴리즈 디렉터리와 SHA256 체크섬 자동 생성을 넣었다
- `src/`는 Git 관리, `dist/`는 빌드 산출물로 분리했다
- Windows용 Scoop과 Homebrew share 경로 지원, Windows 경로 정규화

전체 이력은 `git log`와 [Releases](https://github.com/devsepnine/hibi_ai/releases)를 참조한다.

## 라이선스

[MIT License](../LICENSE).
