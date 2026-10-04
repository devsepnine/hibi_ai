# hibi-ai 문서 인덱스

> 마지막 업데이트: 2026-10-04 · 버전 v1.22.0

컴포넌트 목록은 중복하지 않는다. 이 문서는 "무엇이 어디에 있는가"만 다루고, 실제 목록은 [README.md](README.md)가 SSOT다.

## 문서 목록

### 이 디렉터리

- **[README.md](README.md)**: 프로젝트 전체 문서
  - 프로젝트 구조 / 주요 컴포넌트인 에이전트·커맨드·스킬·MCP·플러그인 / 빌드·릴리즈 / 설치·사용법 / 멀티소스 / 최근 변경사항
- **[RUNBOOK.md](RUNBOOK.md)**: 운영 가이드
  - 릴리즈 절차는 자동과 수동 / 릴리즈 후 검증 / 문제 해결 / 롤백 / 긴급 대응 / 유지보수 체크리스트
- **[FEATURES.md](FEATURES.md)**: 기능맵
  - 모호한 제보를 화면 위치, 사용자 표현, 실제 UI 문구로 코드에 연결한다. 화면별 항목은 [features/](features/) 아래에 있다
- **[ARCHITECTURE.md](ARCHITECTURE.md)**: 아키텍처 규칙
  - 파일, 모듈, import를 추가하기 전에 확인할 규칙 19개와 Known violations, Decisions. 기계적 규칙은 `python tools/lint-arch.py`가 검사한다

### 저장소 루트

- **[../README.md](../README.md)**: 프로젝트 소개. 영문, 배포 대상 사용자용
- **[../CLAUDE.md](../CLAUDE.md)**: 이 저장소를 개발할 때 읽는 지침이며 배포되지 않는다. 기능맵과 아키텍처 규칙을 가리키는 줄, `-ko.md` 미러 규칙, 커밋 전에 돌릴 검사 네 개를 담는다
- **[../LICENSE](../LICENSE)**: MIT License

루트에 `AGENTS.md`는 없다. 사용자에게 배포되는 항상-로드 지침 두 파일은 설정의 일부이므로 `src/` 아래에 있다:

- **[../src/CLAUDE.md](../src/CLAUDE.md)**: Claude Code 항상-로드 지침. 워크플로 오케스트레이션, effort×model 정책, 에이전트 라우팅, 정책 라우팅 표를 담는다
- **[../src/AGENTS.md](../src/AGENTS.md)**: Codex 항상-로드 지침. Codex는 `skills/`와 이 파일만 설치되므로 출력 서식 규칙의 유일한 사본이다

## 컴포넌트가 있는 곳

| 컴포넌트 | 경로 | 목록 |
|---|---|---|
| 에이전트 | `src/agents/<name>.md` | [README 에이전트 표](README.md#에이전트-8개) |
| 슬래시 커맨드 | `src/commands/<name>.md` | [README 커맨드 표](README.md#슬래시-커맨드-28개) |
| 스킬 | `src/skills/<name>/SKILL.md` | [README 스킬 표](README.md#스킬-30개) |
| 훅 | `src/hooks/<name>/hook.yaml` | 전부 `deprecated: true`이며 인스톨러가 자동 제거 |
| MCP 서버 | `src/mcps/mcps.yaml` | 단일 파일 |
| 플러그인 | `src/plugins/plugins.yaml` | 단일 파일 |
| 출력 스타일 | `src/output-styles/hibi_default.md` | 단일 파일 |
| 전역 설정 | `src/settings.json` | 단일 파일 |

`-ko.md` 접미사 파일은 각 원본의 한국어 미러다. 개발자 가독용이며 설치되지 않는다.

`src/rules/`와 `src/contexts/`는 존재하지 않는다. 정책은 스킬로 이관됐고 `contexts/`는 Claude Code가 읽지 않아 v1.17.0에서 삭제했다. 인스톨러는 두 타입을 여전히 지원하므로 사용자 소스에서는 사용할 수 있다.

## 정책이 있는 곳

상세 정책은 **스킬**에 있다. 트리거될 때만 로드되어 always-on 컨텍스트를 가볍게 유지한다. 라우팅 표의 SSOT는 `src/CLAUDE.md`의 "Policy routing" 섹션이다.

| 정책 | 스킬 | 로드 |
|---|---|---|
| 커밋 규약 | `commit-rules` | `/commit` 또는 git commit 시 트리거 |
| PR 가이드라인 · 업스트림 설정 기여 | `pull-request` | `/pull-request`, `/upstream-pr` 또는 PR 작업 시 트리거 |
| 보안 / OWASP | `security-review` | `/security-review` 또는 인증·입력·시크릿 작업 시 트리거 |
| 테스트 & TDD | `tdd-workflow` | `/tdd` 또는 기능 추가·버그 수정 |
| 코딩 스타일 | `coding-standards` | 코드 작성·리뷰 시 |
| 아키텍처 규칙 | `architecture-rules` | `/architecture-rules` 또는 코드베이스 규칙 문서화 시 트리거 |
| 산문 문체 | `technical-writing` | 문서 작성·리뷰 시 트리거. 구두점은 `python tools/lint-prose.py`가 강제 |
| 의존성·결합도 | `dependency-design` | `/deps` 또는 모듈·모노레포 설계 |
| 빌드·타입 에러 | `verification-loop` | `/verify`, `/build-fix` |
| 보증 티어·추적성 | `do-178c` | `/do-178c` 또는 safety-critical 작업 |

항상 적용되는 핵심 불변식은 `src/CLAUDE.md`에 있다. 커밋·푸시 절대 규칙, 주석 절대 규칙, effort×model, 에이전트 라우팅이 그 내용이다.

온디맨드 참조 파일은 `src/skills/coding-standards/references/`에 있다: `code-thresholds.md`는 LOC·복잡도 임계값, `review-checklist.md`는 SOLID·심각도·동시성·크로스플랫폼, `patterns.md`는 공통 TS 패턴을 다룬다.

## 개발 문서

### 인스톨러 소스 `tools/installer/src/`

전체 69 파일, 13,922줄. 테스트·공백을 포함한 raw 라인 수다.

| 모듈 | 파일 / LOC | 내용 |
|---|---|---|
| `app/` | 10 / 1,970 | 앱 상태. `mod.rs`는 App, 나머지는 `types.rs`, `navigation.rs`, `selection.rs`, `processing.rs`, `input.rs`, `settings.rs`, `sources.rs`, `source_wizard.rs`, `test_support.rs`이며 `test_support.rs`는 테스트용 App·MCP·플러그인 픽스처다 |
| `ui/` | 17 / 2,958 | 렌더링. 리스트, diff, 탭, MCP/플러그인 목록, 소스 위저드, 로딩 화면, `help.rs`, `confirm_exit.rs`, `layout.rs`, `tests.rs`가 있다. `help.rs`는 `?` 키바인딩 오버레이로 전체 키 목록의 SSOT이고, `confirm_exit.rs`는 `Esc` 이탈 확인 프롬프트, `layout.rs`는 오버레이 중앙 배치 헬퍼, `tests.rs`는 판 테두리·타이틀·상태바·오버레이 렌더 검증이다 |
| `fs/scanner/` | 6 / 1,390 | 컴포넌트 스캔: `mod.rs`, `components.rs`, `validation.rs`, `external.rs`, `mcp.rs`, `plugin.rs` |
| `fs/installer/` | 9 / 2,034 | 설치·제거: `mod.rs`, `process.rs`, `settings.rs`, `merge.rs`, `mcp.rs`, `plugin.rs`, `renamed.rs`, `unshipped.rs`, `test_support.rs`. `process.rs`는 spawn/cancel을 맡고, `renamed.rs`는 이름을 바꾼 스킬의 옛 디렉터리에서 설치 기록에 있는 파일만 지운다. `unshipped.rs`는 번들 소스가 더는 내보내지 않는 기록된 파일을 기록된 해시와 같을 때만 지우고, `test_support.rs`는 두 정리 모듈이 함께 쓰는 테스트 픽스처다 |
| `fs/` 직속 | 3 / 1,326 | `mod.rs`, `diff.rs`, `manifest.rs`이며 `manifest.rs`는 install.json과 그 안의 파일 해시를 다룬다. 스캐너와 해시가 함께 쓰는 줄바꿈 정규화 `normalize_line_endings`는 `mod.rs`에 있다 |
| `source/` | 3 / 898 | `mod.rs`는 find/sync/resolve, `git.rs`, `config.rs`는 sources.yaml을 다룬다 |
| `loading/` | 6 / 754 | 배경 스레드를 기다리는 세 화면: `channels.rs`는 채널 소유, `scan.rs`는 refresh 페이로드, 나머지는 `initial_load.rs`, `install.rs`, `preflight.rs` |
| `tree/` | 4 / 622 | 접히는 폴더 트리: `mod.rs`는 `TreeNode`·`TreeView`, `build.rs`는 경로→노드, `navigate.rs`는 커서·펼침, `selection.rs`는 폴더 단위 선택 |
| `cli/` | 2 / 571 | `mod.rs`는 키 디스패치와 `--sync`, `tests.rs`는 패인 digit·`?` 오버레이·`Esc` 이탈 확인 키 라우팅 검증 |
| 루트 직속 | 9 / 1,399 | `main.rs`는 125줄로 터미널 셋업과 이벤트 루프다. `target.rs`는 대상 CLI `TargetCli`와 설정 디렉터리, `paths.rs`는 홈 디렉터리를 얻는 유일한 지점, `exec.rs`는 `fs`와 `source`가 함께 쓰는 짧은 외부 명령 실행기다. 나머지는 `component.rs`, `mcp.rs`, `plugin.rs`, `process_exec.rs`, `theme.rs` |

테스트: `cargo test --manifest-path tools/installer/Cargo.toml`. Windows에서 잰 수로 177 tests. ignore로 표시된 1개는 타임아웃 테스트가 자식 프로세스로 띄우는 도우미다.

파일 길이 한도는 `coding-standards` 스킬의 `references/code-thresholds.md`가 정한 soft 300줄, hard 500줄이다. 이 저장소에서 재는 방법은 [ARCHITECTURE.md](ARCHITECTURE.md)의 `arch-file-size`가 정한다. 빈 줄, 주석, 인라인 `mod tests` 블록을 뺀 프로덕션 코드만 세고, rustfmt 도입으로 줄이 늘어 2026-10-04에 이 기준으로 바꿨다. 이 기준으로 soft 초과는 302줄인 `ui/mod.rs` 하나이며 Known violations에 올라 있고, hard 초과는 0개다. 직전까지 초과했던 `loading.rs` 456줄, `tree.rs` 337줄, `cli.rs` 302줄은 각각 `loading/`, `tree/`, `cli/` 디렉터리 모듈로 분리했다. 위 표의 LOC은 raw 라인 수이므로 임계값 판정에 그대로 쓰지 않는다.

### 빌드·릴리즈 스크립트

- `tools/installer/build.sh`: 전 플랫폼 크로스 컴파일. macOS Universal, Linux musl, Windows mingw를 포함한다
- `tools/statusline/build.sh`: 상태 표시줄 바이너리
- `package.sh`: 릴리즈 아카이브와 nfpm으로 만드는 Linux `.deb`/`.rpm`/`.apk`, 그리고 `checksums.txt`. `VERSION` 상수가 릴리즈 워크플로의 검증 기준이다
- `nfpm.yaml`: Linux 패키지 정의. `/usr/bin/hibi`와 `/usr/share/hibi` 레이아웃이며 `VERSION`은 환경 변수로 주입한다
- `install.sh`: Linux curl|sh 인스톨러. 체크섬 검증 후 `~/.local` 또는 `HIBI_PREFIX`에 설치한다
- `tools/lint-prose.py`: 산문의 em dash, en dash, 대시로 쓰인 하이픈, 괄호와 frontmatter `description`의 따옴표 없는 `: `를 잡는다. 220자를 넘는 `description`도 잡는다. 인자가 없으면 `src`, `docs`, `README.md`, `CLAUDE.md`를 검사하고, 코드 펜스·인라인 코드·링크 대상은 제외한다. 위반이 있으면 exit 1이며 릴리즈 워크플로가 빌드 전에 실행한다
- `tools/lint-arch.py`: `docs/ARCHITECTURE.md`의 기계적 `MUST` 규칙 12개를 검사한다. Known violations 표를 허용 목록으로 읽고, 더는 위반하지 않는 행은 stale로 보고한다. 새 위반이나 stale 행이 있으면 exit 1이며 릴리즈 워크플로가 빌드 전에 실행한다
- `.github/workflows/release.yml`: 태그 `v*.*.*` 푸시로 트리거된다. 태그, `package.sh` VERSION, `Cargo.toml` version이 같은지 검증하고, 두 린트와 `cargo fmt --check`, `cargo test`를 통과하면 빌드·패키징·GitHub Release를 발행한다. `lint-arch.py`가 커밋 이력을 비교하므로 checkout은 전체 이력을 받는다

### 설정 파일

- `tools/installer/Cargo.toml`: Rust 프로젝트 설정. 버전이 바이너리와 `install.json`에 각인된다
- `src/settings.json`: Claude Code 전역 설정. 설치 시 기존 설정과 병합한다
- `src/mcps/mcps.yaml` / `src/plugins/plugins.yaml`: MCP·플러그인 정의

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
2. 컴포넌트 추가 시 해당 디렉터리의 기존 파일을 형식 기준으로 삼는다. 스킬을 추가하면 `description`이 목록 예산 8,000자를 공유하므로 220자 이하로 유지한다
3. `-ko.md` 미러를 함께 갱신한다. frontmatter는 원본과 바이트 단위로 동일하게 둔다
4. 커밋 전 루트 [CLAUDE.md](../CLAUDE.md)의 검사 네 개를 실행한다. 릴리즈 워크플로도 빌드 전에 같은 검사를 돌린다
5. PR 전 `/pull-request`로 제목 형식·템플릿·PR 전 체크리스트를 확인한다

## 자주 묻는 질문

**Q: macOS에서 "developer cannot be verified" 경고가 납니다.**
A: [RUNBOOK.md](RUNBOOK.md) "macOS Gatekeeper 경고" 참조

**Q: 빌드가 실패합니다.**
A: [RUNBOOK.md](RUNBOOK.md) "빌드 실패" 참조

**Q: 새 에이전트/스킬을 추가하려면?**
A: `src/agents/`·`src/skills/`의 기존 파일 참조. 스킬은 `/learn`으로 세션 패턴에서 추출할 수도 있다

**Q: 커밋 메시지 형식은?**
A: `commit-rules` 스킬이나 `/commit`

**Q: `-ko.md` 파일은 왜 설치되지 않나요?**
A: 개발자 가독용 미러다. 인스톨러 스캐너가 stem이 `-ko`로 끝나는 파일을 건너뛰고, `package.sh`가 릴리즈 번들에서 제거한다

## 문서 작성 규칙

1. 마크다운, 한글 설명 + 영문 코드·경로
2. 헤더 계층 명확히, 이모지 사용하지 않음
3. 파일 경로는 실제 존재를 확인하고 쓴다. 끊어진 링크가 있는 문서는 없는 문서보다 나쁘다
4. 목록·수치를 중복 서술하지 않는다. 한 곳에 SSOT로 두고 나머지는 링크한다
5. 상단에 "마지막 업데이트" 날짜와 기준 버전을 명시한다

## 문서 업데이트 이력

- **2026-10-04**: v1.22.0 반영. 인스톨러 모듈 표에 `fs/installer/unshipped.rs`, `fs/installer/test_support.rs`를 넣고, `normalize_line_endings`가 `fs/mod.rs`로 옮겨 간 것과 `manifest.rs`의 파일 해시를 적었다. 파일 수 67 → 69, LOC과 테스트 수 151 → 177을 현행화
- **2026-10-04**: v1.21.0 반영. `how` 스킬과 `/review-panel` 커맨드 추가로 커맨드 27 → 28, 스킬 29 → 30. README 앵커 `#슬래시-커맨드-28개`, `#스킬-30개`를 헤딩과 함께 갱신. 저장소 루트 목록에 개발용 `CLAUDE.md`를 추가. 인스톨러 모듈 표에 `target.rs`, `paths.rs`, `exec.rs`, `fs/installer/renamed.rs`를 넣고 LOC과 테스트 수 137 → 151을 현행화. 파일 길이 판정을 `arch-file-size` 기준으로 맞췄다
- **2026-10-04**: v1.20.0 반영. `architecture-rules` 스킬과 `/architecture-rules` 커맨드 추가로 커맨드 26 → 27, 스킬 28 → 29. README 앵커 `#슬래시-커맨드-27개`, `#스킬-29개`를 헤딩과 함께 갱신. 문서 목록에 `FEATURES.md`·`ARCHITECTURE.md`를, 정책 표에 `architecture-rules` 행을, 개발 스크립트에 `tools/lint-arch.py`를 추가. 인스톨러 LOC과 테스트 수 133 → 137을 현행화
- **2026-10-04**: `blast-radius`·`technical-writing`·`why` 스킬과 `/blast-radius`·`/bugfix`·`/refactor`·`/perf` 커맨드 추가 반영. 커맨드 22 → 26, 스킬 25 → 28. README 앵커 `#슬래시-커맨드-26개`, `#스킬-28개`를 헤딩과 함께 갱신. 정책 표에 `technical-writing` 행을, 개발 스크립트에 `tools/lint-prose.py`를 추가
- **2026-10-04**: `feature-map` 스킬 및 `/feature-map` 커맨드 추가 반영. 커맨드 21 → 22, 스킬 24 → 25. README 커맨드·스킬 표 앵커 `#슬래시-커맨드-22개`, `#스킬-25개`를 헤딩과 함께 갱신
- **2026-09-18**: 커밋 권한 규칙이 `push`·`gh pr create`까지 덮도록 강화된 것을 반영했다. `src/CLAUDE.md`의 절대 규칙 섹션 제목이 "Absolute commit and push rules"로 바뀌었다
- **2026-09-11**: `pull-request`가 §2 본문-diff 도출, §5 필요성 검증 리뷰, §6 리뷰 코멘트 분류를 갖게 됐고 범위는 §1부터 §7까지다. 통합으로 생긴 댕글링 지시문 `"run /upstream-pr"`, 즉 존재하지 않는 스킬 호출을 `src/CLAUDE.md:36`·`commands/learn.md:98`·`commands/upstream-pr.md:25`에서 정리하고, 후자의 방법론 참조를 삭제된 `upstream-pr` 스킬에서 `pull-request` §1부터 §7로 재지정했다. `upstream-pr` 스킬이 `pull-request`로 통합됨을 반영했다. 스킬은 25 → 24. 정책 라우팅 표에서 `pull-request`가 PR 가이드라인과 업스트림 설정 기여를 함께 소유하고 `/upstream-pr`이 두 번째 진입점이 됐다. README 스킬 표 앵커 `#스킬-24개`를 헤딩과 함께 갱신했다. 한쪽만 바꾸면 링크가 끊긴다
- **2026-09-09**: `src/` 마크다운 재개편, 즉 v1.16.0 이후 변경을 반영했다. 3개 문서를 전면 현행화했고, 컴포넌트 수·인스톨러 모듈·릴리즈 자동화를 반영했으며, 끊어진 링크 `../CLAUDE.md`, `../AGENTS.md`, `rules/pull-request-rules.md`를 정정하고, 중복 목록을 README로 단일화했다. `/pull-request`·`/security-review` 커맨드 신규 추가로 정책 라우팅 표 10행 전부가 실제 커맨드를 갖게 됐다. 커맨드는 19 → 21
- **2026-06-19**: `dependency-design` 스킬 및 `/deps` 커맨드 추가 반영
- **2026-02-26**: 인스톨러 모듈 구조 재편 반영
- **2026-02-25**: 초기 문서 생성. README.md, RUNBOOK.md, INDEX.md
