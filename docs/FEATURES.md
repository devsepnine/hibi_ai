# hibi-ai 기능맵

검증 시점 `30573a5` · `2026-10-04` · 구조 뷰: 없음, 컴포넌트 위치는 `docs/INDEX.md`
전제: 서버 없음, 상태는 로컬 파일 ~/.claude · ~/.codex · ~/.hibi 뿐 · i18n 없음, UI 문구는 Rust 문자열 리터럴 그대로이고 번역본이 없다 · 인스톨러 테스트는 각 모듈의 `#[cfg(test)]` 블록과 `tools/installer/src/cli/tests.rs`, `tools/installer/src/ui/tests.rs` 에 있고 `cargo test --manifest-path tools/installer/Cargo.toml` 로 돈다 · 배포 설정 `src/` 는 산문이라 실행 테스트가 없고 일부 스킬만 `evals/` 평가셋을 가진다

## 이 맵 사용법

"사이드바에 뭐가 잘 안돼요", "입금 문구가 잘 입력이 안돼요" 같은 모호한 제보를 받으면:

1. **어디서** 일어났나? → 화면 인덱스 → 후보 기능 ID.
2. **어떤 단어**를 썼나? → 별칭 인덱스 → 후보 좁히기. `(ambiguous)` 표시된 별칭은 여러 기능에 걸린다. 전부 확인하거나, 둘을 가르는 질문 하나만 한다. `→ 없음` 행은 제품에 없는 기능이다.
3. **어떤 문구**를 봤나? → 항목에 적힌 실제 문자열로 grep.
4. 항목의 코드 경로를 위에서부터 연다. 디버깅 전에 **함정**부터 읽는다.
5. 어디에도 없으면 → 코드를 검색한 뒤, 빠진 기능·별칭·문구를 이 맵에 추가한다.

## 화면 인덱스

| 화면 / 영역 | 기능 ID |
|---|---|
| 첫 화면 HIBI AI 배너와 `Select target` 목록 | `cli-picker`, `sources-screen` |
| 시작 직후와 CLI 선택 직후의 스피너 상자 | `startup-loading` |
| 터미널 명령행, TUI 없이 | `cli-flags`, `install-script` |
| 목록 화면 상단 탭 바, 테두리 `[1]-` | `tab-bar`, `theme-toggle` |
| 목록 화면 본문, 테두리 `[2]-` | `component-list`, `mcp-list`, `plugin-list`, `default-style-statusline` |
| 목록 화면 하단 상태 바 | `status-bar` |
| `?` 로 뜨는 Keybindings 상자 | `help-overlay` |
| 본문에서 Esc 를 누르면 뜨는 Confirm 상자 | `exit-confirm` |
| `d` 나 Enter 로 여는 Diff 화면 | `diff-view` |
| MCP 탭 위에 뜨는 입력 대화상자 | `mcp-env-input`, `mcp-list` |
| `i` · `r` 후의 Progress 와 Log 화면 | `install-progress`, `settings-merge`, `install-manifest` |
| Sources 전체 화면 | `sources-screen`, `source-sync` |
| Sources 위에 뜨는 Add Source 단계 상자 | `source-wizard` |
| Claude Code 입력창 아래 상태줄 | `statusline-render`, `default-style-statusline` |
| 디스크의 ~/.claude/settings.json | `settings-merge`, `default-style-statusline`, `plugin-list` |
| 디스크의 ~/.hibi 디렉터리 | `install-manifest`, `sources-screen`, `source-sync` |
| GitHub 릴리즈, deb · rpm · apk, brew · scoop | `release-workflow`, `packaging`, `install-script` |
| Claude Code · Codex 대화 응답 | `commit-push-gating`, `response-language`, `comment-rules`, `review-gate`, `verification-phase7`, `skill-triggering` |
| 문서 산문 린트 | `prose-lint` |
| `src/` 아래 배포 설정 파일 | `component-lookup` |

## 별칭 인덱스

| 사용자 표현 | 기능 ID |
|---|---|
| 인스톨러, hibi, TUI, 설정 설치기 | `cli-picker`, `component-list` |
| 설치, install, 인스톨 `(ambiguous)` | `install-progress`, `install-script`, `component-list` |
| Claude 선택, Codex 선택, 대상 선택, 첫 화면 | `cli-picker` |
| 로딩, 로딩 안 끝남, 멈춤, 스피너, Scanning | `startup-loading`, `install-progress` |
| 시작했더니 에러 없이 첫 화면으로 돌아감, 경고가 안 보임 | `startup-loading` |
| 훅이 사라짐, 훅 자동 삭제, Auto-cleaned, 훅 탭이 비어 있음, 훅 설치 안 됨 | `startup-loading`, `component-list`, `component-lookup` |
| Cannot find source directory, 바이너리 옮겼더니 실행 안 됨 | `startup-loading`, `install-script` |
| --version, 버전 확인, -v, --help, 도움말 옵션 | `cli-flags` |
| 버전 `(ambiguous)` | `cli-flags`, `status-bar`, `release-workflow`, `install-manifest` |
| 동기화, sync, --sync, 업데이트 받기, 최신화 `(ambiguous)` | `source-sync`, `cli-flags` |
| 탭, 탭 바, 탭 전환, 탭 이동, 탭이 잘림, ‹ › | `tab-bar` |
| 숫자 키, 1번 2번, pane, 패널, 포커스 `(ambiguous)` | `tab-bar`, `source-wizard` |
| 목록, 리스트, 체크박스, 전체 선택, 선택 해제, 폴더 펼치기, 트리, 미리 체크됨 | `component-list` |
| Space 안 먹힘, 체크가 안 됨 | `component-list`, `cli-picker` |
| 에이전트, 커맨드, 스킬, Styles, Config 탭 | `component-list` |
| modified, installed, new, external, managed, 상태 표시 | `component-list` |
| 바꾼 적 없는데 modified, 줄바꿈 CRLF | `component-list` |
| 하단 바, 상태 바, 상태 메시지, 메시지가 잘림 | `status-bar` |
| 상태줄, 상태표시줄, statusline `(ambiguous)` | `statusline-render`, `default-style-statusline`, `status-bar`, `settings-merge` |
| 상태줄이 안 나옴, 상태줄 빈칸 | `statusline-render`, `default-style-statusline` |
| 단축키, 키 목록, 키바인딩, 도움말, ?, 키가 안 보임 | `help-overlay`, `status-bar` |
| 종료, 나가기, quit, q 안 먹힘 `(ambiguous)` | `help-overlay`, `install-progress`, `sources-screen`, `exit-confirm` |
| Esc, 뒤로가기, 선택 날아감, 확인 창 `(ambiguous)` | `exit-confirm`, `tab-bar`, `install-progress` |
| 엔터로 확인 안 됨, 엔터 안 먹힘, y 눌러야 함 | `exit-confirm`, `sources-screen`, `mcp-env-input` |
| diff, 비교, 변경사항 보기, 차이, diff 스크롤 | `diff-view` |
| 인스톨러가 갑자기 꺼짐, 튕김, 패닉, Error: 찍고 종료 | `diff-view`, `default-style-statusline`, `mcp-env-input` |
| 기본 스타일, output style, 기본값 설정, DEFAULT, s u 키 | `default-style-statusline` |
| settings.json `(ambiguous)` | `settings-merge`, `default-style-statusline` |
| 설정 병합, 설정 덮어씀, permissions 날아감, CLAUDE.md 덮어씀 | `settings-merge` |
| MCP, 엠씨피, MCP 서버, mcp list, Not Installed 로만 보임, mcps.yaml 서버가 안 보임 | `mcp-list` |
| 스코프, scope, local, user, 프로젝트 경로 | `mcp-list` |
| 환경변수, env, API 키, 토큰 입력 | `mcp-env-input` |
| 한글, CJK, 한글 깨짐, 한글 입력, 글자 폭 `(ambiguous)` | `mcp-env-input`, `status-bar`, `tab-bar`, `statusline-render` |
| 플러그인, plugin, 마켓플레이스, marketplace | `plugin-list` |
| 진행률, 로그, Progress, 설치 중, 취소 `(ambiguous)` | `install-progress`, `source-sync`, `mcp-env-input` |
| CLI not found, claude 못 찾음, PATH | `install-progress`, `mcp-list` |
| install.json, 매니페스트, 어디서 설치했는지, 출처, 업스트림 | `install-manifest` |
| 테마, 다크모드, 라이트모드, Mocha, Latte, 색 | `theme-toggle` |
| 라이트 테마에서 제목이 안 보임 `(ambiguous)` | `sources-screen`, `theme-toggle` |
| 소스, sources, sources.yaml, 소스 관리, 소스 삭제, bundled 수정 안 됨 | `sources-screen` |
| 소스 우선순위, 같은 이름 컴포넌트가 다른 소스 것으로 바뀜 | `sources-screen`, `component-list` |
| 소스 추가, git 소스, 로컬 소스, 브랜치, root, map_to, 소스 수정, credentials 오류 | `source-wizard` |
| stale, 캐시, 오프라인, fetch 실패 | `source-sync` |
| 토큰 사용량, ctx, 5h, 7d, todos, 줄바꿈 | `statusline-render` |
| 설치 스크립트, curl, install.sh, 리눅스 설치 | `install-script` |
| 패키징, dist, deb, rpm, apk, nfpm, 빌드 | `packaging` |
| 릴리즈, 배포, 태그, GitHub Actions, 체크섬 | `release-workflow`, `packaging`, `statusline-render` |
| 커밋, 푸시, PR, 묻지 않고 커밋, 자동 커밋, Co-Authored-By, 티켓 번호 | `commit-push-gating` |
| 영어로 답함, 한국어 응답, 응답 언어, 출력 스타일 | `response-language` |
| 주석, 주석이 너무 많음, 주석 언어 | `comment-rules` |
| 리뷰, 리뷰어, 잔소리, 닛픽, nit, 지적이 너무 많음, 리뷰가 안 돌아옴, code-reviewer | `review-gate` |
| 검증, verify, READY 안 나옴, PARTIAL | `verification-phase7` |
| 스킬이 안 뜸, 스킬 트리거, description, 예산, /learn | `skill-triggering` |
| 대시, 괄호, 줄표, lint-prose, 문체 린트 | `prose-lint` |
| 커맨드 파일 어디, 스킬 파일 위치, 에이전트 정의, -ko 파일, 한국어 미러 | `component-lookup` |
| 목록 검색, 필터, 찾기 | → 없음; 가장 가까운 기능: `component-list` |
| 마우스 클릭, 스크롤 휠 | → 없음; 가장 가까운 기능: `help-overlay` |
| 테마 기억, 테마 저장 | → 없음; 가장 가까운 기능: `theme-toggle` |
| MCP diff, 플러그인 diff | → 없음; 가장 가까운 기능: `diff-view` |
| diff 안에서 검색, PgDn 페이지 넘김 | → 없음; 가장 가까운 기능: `diff-view` |
| 설치 로그 복사, 로그 저장 | → 없음; 가장 가까운 기능: `install-progress` |
| 명령행 일괄 설치, --install, 비대화형 설치 | → 없음; 가장 가까운 기능: `cli-flags` |
| hibi 자동 업데이트, self-update | → 없음; 가장 가까운 기능: `install-script` |
| macOS 용 install.sh, arm64 리눅스 설치 | → 없음; 가장 가까운 기능: `install-script` |
| SSH git 소스, git@ 주소 | → 없음; 가장 가까운 기능: `source-wizard` |
| 소스 순서 바꾸는 화면, auto_update 토글 | → 없음; 가장 가까운 기능: `sources-screen` |
| Codex 탭이 적음, Codex 플러그인, Codex 에이전트, Codex 커맨드 `(ambiguous)` | `tab-bar`, `component-lookup`, `cli-picker` |
| 한국어 UI, 언어 설정 화면 | → 없음 |
| 상태줄 색 변경, 상태줄 항목 설정 | → 없음; 가장 가까운 기능: `statusline-render` |

## 기능

항목은 화면별 파일로 나눴다. 조회할 때는 위 인덱스에서 ID를 고른 뒤 해당 파일을 연다.

| ID | 목적 | 파일 |
|---|---|---|
| `cli-picker` | Claude Code · Codex CLI · Manage Sources 중 대상 선택 | `docs/features/installer-startup.md` |
| `startup-loading` | 시작 스피너, 소스 해석, 첫 스캔, deprecated 훅 정리 | `docs/features/installer-startup.md` |
| `cli-flags` | `--help`, `--version`, `--sync` | `docs/features/installer-startup.md` |
| `tab-bar` | 상단 탭 바와 pane 포커스 이동 | `docs/features/installer-list.md` |
| `component-list` | 컴포넌트 트리 목록, 상태, 선택 | `docs/features/installer-list.md` |
| `status-bar` | 하단 키 힌트, 상태 메시지, 버전 | `docs/features/installer-list.md` |
| `help-overlay` | `?` 키바인딩 표 | `docs/features/installer-list.md` |
| `exit-confirm` | Esc 로 CLI 선택 화면 복귀 확인 | `docs/features/installer-list.md` |
| `diff-view` | 원본과 설치본 diff | `docs/features/installer-list.md` |
| `default-style-statusline` | Styles · Statusline 탭의 기본값 지정 | `docs/features/installer-list.md` |
| `mcp-list` | MCP 탭 목록과 user · local 스코프 | `docs/features/installer-list.md` |
| `mcp-env-input` | MCP 환경변수 입력 대화상자 | `docs/features/installer-list.md` |
| `plugin-list` | Plugins 탭 목록 | `docs/features/installer-list.md` |
| `install-progress` | 사전 점검, 설치 · 제거 진행, 취소, 새로고침 | `docs/features/installer-list.md` |
| `settings-merge` | settings.json 병합, 훅 등록, Config 파일 복사 | `docs/features/installer-list.md` |
| `install-manifest` | ~/.hibi/install.json 출처 기록 | `docs/features/installer-list.md` |
| `theme-toggle` | `t` 로 Mocha · Latte 전환 | `docs/features/installer-list.md` |
| `sources-screen` | Sources 목록, 삭제, sources.yaml | `docs/features/installer-sources.md` |
| `source-wizard` | 소스 추가 · 수정 단계 | `docs/features/installer-sources.md` |
| `source-sync` | `f` 동기화, git 캐시, stale | `docs/features/installer-sources.md` |
| `statusline-render` | Claude Code 상태줄 바이너리 출력과 줄바꿈 | `docs/features/statusline.md` |
| `install-script` | 루트 `install.sh` 리눅스 설치 | `docs/features/distribution.md` |
| `packaging` | `package.sh`, `nfpm.yaml`, 빌드 스크립트 | `docs/features/distribution.md` |
| `release-workflow` | 태그 푸시로 도는 릴리즈 워크플로 | `docs/features/distribution.md` |
| `commit-push-gating` | 커밋 · 푸시 · PR 은 명시 요청 때만 | `docs/features/config-behaviors.md` |
| `response-language` | 응답 언어와 출력 서식 | `docs/features/config-behaviors.md` |
| `comment-rules` | 코드 주석 규칙 | `docs/features/config-behaviors.md` |
| `review-gate` | 작업 후 리뷰 게이트와 code-reviewer 필터 | `docs/features/config-behaviors.md` |
| `verification-phase7` | 검증 루프와 실제 산출물 확인 단계 | `docs/features/config-behaviors.md` |
| `skill-triggering` | 스킬 자동 트리거와 description 예산 | `docs/features/config-behaviors.md` |
| `prose-lint` | 산문 구두점 린트 | `docs/features/config-behaviors.md` |
| `component-lookup` | 커맨드 · 스킬 · 에이전트 파일 찾는 규칙 | `docs/features/config-behaviors.md` |
