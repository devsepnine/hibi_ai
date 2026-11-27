# hibi-ai 문서 인덱스

기준 코드 `cc8978b` · `2026-11-27` · 공개 릴리스 `v2.0.0`

## 시작할 문서

| 필요한 정보 | 문서 |
|---|---|
| 앱 설치와 기본 사용법 | [English](../README.md), [한국어](../README.ko.md), [日本語](../README.ja.md) |
| 저장소 구조, 배포 컴포넌트, 변경 이력 | [프로젝트 문서](README.md) |
| 화면이나 오류 문구에서 구현과 테스트 찾기 | [기능맵](FEATURES.md) |
| 모듈과 의존성 경계, 리뷰 기준 | [아키텍처 규칙](ARCHITECTURE.md) |
| 빌드, 릴리스, 서명 검증, 롤백 | [운영 가이드](RUNBOOK.md) |
| 이 저장소의 개발 지침과 커밋 전 검사 | [AGENTS.md](../AGENTS.md) |
| 배포 라이선스 | [MIT License](../LICENSE) |

## 화면과 동작

| 영역 | 문서 |
|---|---|
| 실행 인자, 대상 선택, 초기 스캔 | [인스톨러 시작](features/installer-startup.md) |
| GUI 헤더, 탐색, 검색, 필터와 대화상자 | [GUI 기능과 검수 기준](features/installer-gui.md) |
| TUI 탐색과 공용 설치·제거 동작 | [인스톨러 목록](features/installer-list.md) |
| Git과 로컬 소스, 동기화, 캐시 | [소스 관리](features/installer-sources.md) |
| 설치 스크립트, 패키지, 앱 업데이트 | [배포](features/distribution.md) |
| Claude Code 상태줄 | [상태줄](features/statusline.md) |
| 배포 설정의 리뷰, 산출물 작성과 스킬 트리거 | [배포 설정 동작](features/config-behaviors.md) |

## 인스톨러와 상태줄 소스

| 경로 | 주요 책임 |
|---|---|
| [tools/installer/src/main.rs](../tools/installer/src/main.rs) | `--help`, `--version`, `--sync` 분기. 기본 GUI 진입, `--tui` 이벤트 루프와 터미널 복원 |
| [tools/installer/src/gui/](../tools/installer/src/gui/) | `mod.rs`에서 iced 앱을 구성한다. `actions.rs`는 입력, `runtime.rs`는 백그라운드 작업, `window_bar.rs`와 `styles.rs`는 창과 테마, `locale.rs`와 `locale/`는 번역을 맡는다. |
| [tools/installer/src/ui/](../tools/installer/src/ui/) | ratatui 화면. `cli_selection.rs`는 대상 선택, `mod.rs`는 전체 화면과 하단 영역, `confirm.rs`는 이탈과 제거 확인, `help.rs`는 키 안내를 그린다. |
| [tools/installer/src/cli/](../tools/installer/src/cli/) | 키 라우팅과 창 없는 동기화. `dispatch_terminal_key`는 검색과 제거 스크롤을 별도로 처리한다. |
| [tools/installer/src/app/](../tools/installer/src/app/) | 공용 상태를 보유한다. `filter.rs`와 `search.rs`는 표시, `selection.rs`와 `removal.rs`는 작업 대상과 확인, `visibility.rs`는 숨김을 담당한다. |
| [tools/installer/src/tree/](../tools/installer/src/tree/) | 컴포넌트 폴더 구조, 커서 이동, 펼침과 선택 |
| [tools/installer/src/loading/](../tools/installer/src/loading/)와 [process_exec.rs](../tools/installer/src/process_exec.rs) | 초기 스캔과 목록 갱신, CLI 사전 점검, 작업 스레드 시작과 진행·취소 |
| [tools/installer/src/fs/scanner/](../tools/installer/src/fs/scanner/) | 파일, MCP, 플러그인 탐색과 상태 판정, 다중 소스 병합 |
| [tools/installer/src/fs/installer/](../tools/installer/src/fs/installer/) | 설치·제거, 설정 키 병합, GUI 환경설정과 폴더 숨김 저장, 이전 설치 정리 |
| [tools/installer/src/fs/manifest.rs](../tools/installer/src/fs/manifest.rs) | `~/.hibi/install.json`과 파일 해시 |
| [tools/installer/src/fs/update/](../tools/installer/src/fs/update/) | 릴리스 조회, 설치 경로 분류, 서명 확인, 업데이트 교체와 복구 |
| [tools/installer/src/source/](../tools/installer/src/source/) | 번들 탐색, `sources.yaml`, Git 클론과 캐시 |
| [tools/installer/src/exec.rs](../tools/installer/src/exec.rs), [paths.rs](../tools/installer/src/paths.rs), [target.rs](../tools/installer/src/target.rs) | 외부 명령 실행과 타임아웃, 홈 경로, 대상 CLI와 설치 경로 |
| [tools/installer/tests/](../tools/installer/tests/) | GUI와 PTY, 설치 스크립트, 서명 업데이트, 플랫폼별 패키지 스모크 테스트 |
| [tools/statusline/](../tools/statusline/) | 인스톨러와 독립적인 Rust 상태줄 크레이트 |

## 배포 설정과 도구

| 파일 또는 경로 | 용도 |
|---|---|
| [src/CLAUDE.md](../src/CLAUDE.md), [src/AGENTS.md](../src/AGENTS.md) | Claude Code와 Codex에 각각 배포하는 상시 지침 |
| [src/agents/](../src/agents/), [src/commands/](../src/commands/), [src/skills/](../src/skills/) | 에이전트, 커맨드, 스킬 원본. 역할은 [컴포넌트 목록](README.md#주요-컴포넌트)을 참고한다. |
| [src/settings.json](../src/settings.json), [mcps.yaml](../src/mcps/mcps.yaml), [plugins.yaml](../src/plugins/plugins.yaml) | Claude Code 설정과 MCP·플러그인 카탈로그 |
| [tools/installer/build.sh](../tools/installer/build.sh) | 실행한 플랫폼에 맞는 인스톨러 빌드 |
| [package.sh](../package.sh), [nfpm.yaml](../nfpm.yaml) | 아카이브, Windows MSI, Linux 패키지와 체크섬 |
| [install.sh](../install.sh) | Linux 아카이브 설치 |
| [tools/lint-prose.py](../tools/lint-prose.py), [tools/lint-arch.py](../tools/lint-arch.py) | 산문과 아키텍처 규칙 검사 |
| [.github/workflows/release.yml](../.github/workflows/release.yml) | 태그 검증, 테스트, 플랫폼별 빌드와 패키징, 서명과 게시 |

기여자는 루트 `AGENTS.md`를 먼저 읽는다. `src/`의 마크다운을 바꿀 때는 같은 위치의 `-ko.md` 미러도 함께 갱신한다. 배포 패키지는 미러를 제외한다.
