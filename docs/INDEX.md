# hibi-ai 문서 인덱스

> 마지막 업데이트: 2026-09-15 · 버전 v1.17.0

컴포넌트 목록은 중복하지 않는다 — 이 문서는 "무엇이 어디에 있는가"만 다루고, 실제 목록은 [README.md](README.md)가 SSOT다.

## 문서 목록

### 이 디렉터리

- **[README.md](README.md)** — 프로젝트 전체 문서
  - 프로젝트 구조 / 주요 컴포넌트(에이전트·커맨드·스킬·MCP·플러그인) / 빌드·릴리즈 / 설치·사용법 / 멀티소스 / 최근 변경사항
- **[RUNBOOK.md](RUNBOOK.md)** — 운영 가이드
  - 릴리즈 절차(자동/수동) / 릴리즈 후 검증 / 문제 해결 / 롤백 / 긴급 대응 / 유지보수 체크리스트

### 저장소 루트

- **[../README.md](../README.md)** — 프로젝트 소개 (영문, 배포 대상 사용자용)
- **[../LICENSE](../LICENSE)** — MIT License

루트에는 `CLAUDE.md`·`AGENTS.md`가 없다. 두 파일은 배포되는 설정의 일부이므로 `src/` 아래에 있다:

- **[../src/CLAUDE.md](../src/CLAUDE.md)** — Claude Code 항상-로드 지침 (워크플로 오케스트레이션, effort×model 정책, 에이전트 라우팅, 정책 라우팅 표)
- **[../src/AGENTS.md](../src/AGENTS.md)** — Codex 항상-로드 지침 (Codex는 `skills/`와 이 파일만 설치되므로 출력 서식 규칙의 유일한 사본)

## 컴포넌트가 있는 곳

| 컴포넌트 | 경로 | 목록 |
|---|---|---|
| 에이전트 | `src/agents/<name>.md` | [README 에이전트 표](README.md#에이전트-8개) |
| 슬래시 커맨드 | `src/commands/<name>.md` | [README 커맨드 표](README.md#슬래시-커맨드-21개) |
| 스킬 | `src/skills/<name>/SKILL.md` | [README 스킬 표](README.md#스킬-24개) |
| 훅 | `src/hooks/<name>/hook.yaml` | 전부 `deprecated: true` — 인스톨러가 자동 제거 |
| MCP 서버 | `src/mcps/mcps.yaml` | 단일 파일 |
| 플러그인 | `src/plugins/plugins.yaml` | 단일 파일 |
| 출력 스타일 | `src/output-styles/hibi_default.md` | 단일 파일 |
| 전역 설정 | `src/settings.json` | 단일 파일 |

`-ko.md` 접미사 파일은 각 원본의 한국어 미러다. 개발자 가독용이며 설치되지 않는다.

`src/rules/`와 `src/contexts/`는 존재하지 않는다 — 정책은 스킬로 이관됐고 `contexts/`는 Claude Code가 읽지 않아 v1.16.0에서 삭제했다. 인스톨러는 두 타입을 여전히 지원하므로 사용자 소스에서는 사용할 수 있다.

## 정책이 있는 곳

상세 정책은 **스킬**에 있다. 트리거될 때만 로드되어 always-on 컨텍스트를 가볍게 유지한다. 라우팅 표의 SSOT는 `src/CLAUDE.md`의 "Policy routing" 섹션이다.

| 정책 | 스킬 | 로드 |
|---|---|---|
| 커밋 규약 | `commit-rules` | `/commit` 또는 git commit 시 트리거 |
| PR 가이드라인 · 업스트림 설정 기여 | `pull-request` | `/pull-request`, `/upstream-pr` 또는 PR 작업 시 트리거 |
| 보안 / OWASP | `security-review` | `/security-review` 또는 인증·입력·시크릿 작업 시 트리거 |
| 테스트 & TDD | `tdd-workflow` | `/tdd` 또는 기능 추가·버그 수정 |
| 코딩 스타일 | `coding-standards` | 코드 작성·리뷰 시 |
| 의존성·결합도 | `dependency-design` | `/deps` 또는 모듈·모노레포 설계 |
| 빌드·타입 에러 | `verification-loop` | `/verify`, `/build-fix` |
| 보증 티어·추적성 | `do-178c` | `/do-178c` 또는 safety-critical 작업 |

항상 적용되는 핵심 불변식(커밋 절대 규칙, 주석 절대 규칙, effort×model, 에이전트 라우팅)은 `src/CLAUDE.md`에 있다.

온디맨드 참조 파일은 `src/skills/coding-standards/references/`에 있다: `code-thresholds.md`(LOC·복잡도 임계값), `review-checklist.md`(SOLID·심각도·동시성·크로스플랫폼), `patterns.md`(공통 TS 패턴).

## 개발 문서

### 인스톨러 소스 (`tools/installer/src/`)

전체 63 파일 / 11,714줄(테스트·공백 포함 raw 라인 수).

| 모듈 | 파일 / LOC | 내용 |
|---|---|---|
| `app/` | 10 / 1,925 | 앱 상태 — `mod.rs`(App), `types.rs`, `navigation.rs`, `selection.rs`, `processing.rs`, `input.rs`, `settings.rs`, `sources.rs`, `source_wizard.rs`, `test_support.rs`(테스트용 App·MCP·플러그인 픽스처) |
| `ui/` | 17 / 2,722 | 렌더링 — 리스트, diff, 탭, MCP/플러그인 목록, 소스 위저드, 로딩 화면, `help.rs`(`?` 키바인딩 오버레이 — 전체 키 목록의 SSOT), `confirm_exit.rs`(`Esc` 이탈 확인 프롬프트), `layout.rs`(오버레이 중앙 배치 헬퍼), `tests.rs`(판 테두리·타이틀·상태바·오버레이 렌더 검증) |
| `fs/scanner/` | 6 / 1,300 | 컴포넌트 스캔 — `mod.rs`, `components.rs`, `validation.rs`, `external.rs`, `mcp.rs`, `plugin.rs` |
| `fs/installer/` | 6 / 1,080 | 설치·제거 — `mod.rs`, `process.rs`(spawn/cancel), `settings.rs`, `merge.rs`, `mcp.rs`, `plugin.rs` |
| `fs/` (직속) | 3 / 862 | `mod.rs`, `diff.rs`, `manifest.rs`(install.json) |
| `source/` | 3 / 895 | `mod.rs`(find/sync/resolve), `git.rs`, `config.rs`(sources.yaml) |
| `loading/` | 6 / 684 | 배경 스레드를 기다리는 세 화면 — `channels.rs`(채널 소유), `scan.rs`(refresh 페이로드), `initial_load.rs`, `install.rs`, `preflight.rs` |
| `tree/` | 4 / 586 | 접히는 폴더 트리 — `mod.rs`(`TreeNode`·`TreeView`), `build.rs`(경로→노드), `navigate.rs`(커서·펼침), `selection.rs`(폴더 단위 선택) |
| `cli/` | 2 / 513 | `mod.rs`(키 디스패치 + `--sync`), `tests.rs`(패인 digit·`?` 오버레이·`Esc` 이탈 확인 키 라우팅 검증) |
| 루트 직속 | 6 / 1,147 | `main.rs`(114, 터미널 셋업 + 이벤트 루프), `component.rs`, `mcp.rs`, `plugin.rs`, `process_exec.rs`, `theme.rs` |

테스트: `cargo test --manifest-path tools/installer/Cargo.toml` — 133 tests.

파일 길이 임계값은 `coding-standards` 스킬의 `references/code-thresholds.md`가 SSOT다 — soft 300줄 / hard 500줄, 공백·주석 전용 줄만 제외한다(`#[cfg(test)]` 블록은 제외 대상이 아니다 — 면제는 top-of-file 주석으로 사유를 밝힌 경우만 인정된다). 이 기준으로 **soft·hard 초과 모두 0개다.** 직전까지 초과했던 `loading.rs`(456)와 `tree.rs`(337)는 각각 `loading/`·`tree/` 디렉터리 모듈로 분리했고, 302까지 올라간 `cli.rs`는 `cli/mod.rs`(254) + `cli/tests.rs`(119)로 나눴다. 최댓값은 `fs/manifest.rs` 300으로 soft 한도와 같다. 위 표의 LOC은 raw 라인 수이므로 임계값 판정에 그대로 쓰지 않는다.

### 빌드·릴리즈 스크립트

- `tools/installer/build.sh` — 전 플랫폼 크로스 컴파일 (macOS Universal + Linux musl + Windows mingw)
- `tools/statusline/build.sh` — 상태 표시줄 바이너리
- `package.sh` — 릴리즈 아카이브 + `checksums.txt`. `VERSION` 상수가 릴리즈 워크플로의 검증 기준
- `.github/workflows/release.yml` — 태그 `v*.*.*` 푸시로 트리거. 태그 == `package.sh` VERSION == `Cargo.toml` version 검증 후 빌드·패키징·GitHub Release 발행

### 설정 파일

- `tools/installer/Cargo.toml` — Rust 프로젝트 설정 (버전이 바이너리와 `install.json`에 각인됨)
- `src/settings.json` — Claude Code 전역 설정 (설치 시 기존 설정과 병합)
- `src/mcps/mcps.yaml` / `src/plugins/plugins.yaml` — MCP·플러그인 정의

## 사용법 가이드

### 신규 사용자

1. [README.md](README.md) "설치 방법"
2. `hibi` 실행 → 대상 CLI 선택 → 컴포넌트 선택 → 설치

### 메인테이너

1. [RUNBOOK.md](RUNBOOK.md) "릴리즈 절차"
2. 빌드: `tools/installer/build.sh`
3. 릴리즈: 버전 3곳 동기화 → 태그 푸시 → Actions → Homebrew/Scoop 수동 갱신

### 기여자

1. `src/CLAUDE.md`의 워크플로·코드 품질 규칙 숙지
2. 컴포넌트 추가 시 해당 디렉터리의 기존 파일을 형식 기준으로 삼는다. 스킬을 추가하면 `description`이 목록 예산(8,000자)을 공유하므로 220자 이하로 유지한다
3. `-ko.md` 미러를 함께 갱신한다 — frontmatter는 원본과 바이트 단위로 동일하게 둔다
4. PR 전 `/pull-request`로 제목 형식·템플릿·PR 전 체크리스트를 확인한다

## 자주 묻는 질문

**Q: macOS에서 "developer cannot be verified" 경고가 납니다.**
A: [RUNBOOK.md](RUNBOOK.md) "macOS Gatekeeper 경고" 참조

**Q: 빌드가 실패합니다.**
A: [RUNBOOK.md](RUNBOOK.md) "빌드 실패" 참조

**Q: 새 에이전트/스킬을 추가하려면?**
A: `src/agents/`·`src/skills/`의 기존 파일 참조. 스킬은 `/learn`으로 세션 패턴에서 추출할 수도 있다

**Q: 커밋 메시지 형식은?**
A: `commit-rules` 스킬 (또는 `/commit`)

**Q: `-ko.md` 파일은 왜 설치되지 않나요?**
A: 개발자 가독용 미러다. 인스톨러 스캐너가 stem이 `-ko`로 끝나는 파일을 건너뛰고, `package.sh`가 릴리즈 번들에서 제거한다

## 문서 작성 규칙

1. 마크다운, 한글 설명 + 영문 코드·경로
2. 헤더 계층 명확히, 이모지 사용하지 않음
3. 파일 경로는 실제 존재를 확인하고 쓴다 — 끊어진 링크가 있는 문서는 없는 문서보다 나쁘다
4. 목록·수치를 중복 서술하지 않는다. 한 곳(SSOT)에 두고 나머지는 링크한다
5. 상단에 "마지막 업데이트" 날짜와 기준 버전을 명시한다

## 문서 업데이트 이력

- **2026-09-11**: `pull-request`가 §2 본문-diff 도출, §5 필요성 검증 리뷰, §6 리뷰 코멘트 분류를 갖게 됨(§1–§7). 통합으로 생긴 댕글링 지시문(`"run /upstream-pr"` → 존재하지 않는 스킬 호출)을 `src/CLAUDE.md:36`·`commands/learn.md:98`·`commands/upstream-pr.md:25`에서 정리하고, 후자의 방법론 참조를 삭제된 `upstream-pr` 스킬에서 `pull-request` §1–§7로 재지정. `upstream-pr` 스킬이 `pull-request`로 통합됨을 반영 (스킬 25 → 24). 정책 라우팅 표에서 `pull-request`가 PR 가이드라인과 업스트림 설정 기여를 함께 소유하고 `/upstream-pr`이 두 번째 진입점이 됐다. README 스킬 표 앵커(`#스킬-24개`)를 헤딩과 함께 갱신 — 한쪽만 바꾸면 링크가 끊긴다
- **2026-09-09**: `src/` 마크다운 재개편(v1.16.0 이후) 반영. 3개 문서 전면 현행화 — 컴포넌트 수·인스톨러 모듈·릴리즈 자동화 반영, 끊어진 링크(`../CLAUDE.md`, `../AGENTS.md`, `rules/pull-request-rules.md`) 정정, 중복 목록을 README로 단일화. `/pull-request`·`/security-review` 커맨드 신규 추가로 정책 라우팅 표 10행 전부가 실제 커맨드를 갖게 됐다 (커맨드 19 → 21)
- **2026-06-19**: `dependency-design` 스킬 및 `/deps` 커맨드 추가 반영
- **2026-02-26**: 인스톨러 모듈 구조 재편 반영
- **2026-02-25**: 초기 문서 생성 (README.md, RUNBOOK.md, INDEX.md)
