# 인스톨러 시작 화면과 명령행

### `cli-picker`: 설치 대상 CLI를 고르거나 소스 관리로 들어가는 첫 화면

- **위치**: `hibi` 실행 직후 전체 화면, HIBI AI 배너 아래 `Select target` 상자. 상태 바는 그리지 않는다
- **별칭**: 첫 화면, 대상 선택, Claude 선택, Codex 선택, 인스톨러
- **UI 문구**: `"Select target"`, `"Claude Code"`, `"Anthropic's official CLI for Claude (~/.claude)"`, `"Codex CLI"`, `"OpenAI's ChatGPT-based CLI (~/.codex)"`, `"Manage Sources"`, `"Configure component sources (~/.hibi/sources.yaml)"`, `"Config Installer"`, `"HIBI AI"`
- **컨트롤**: ↑ · k, ↓ · j 는 `cli_selection_index` 이동, Enter → `confirm_cli_selection`, q → `should_quit`
- **코드 경로**:
  - UI: `tools/installer/src/ui/cli_selection.rs` → `OPTIONS`
  - 상태 / 핸들러: `tools/installer/src/cli/mod.rs` → `handle_cli_selection`, `tools/installer/src/cli/mod.rs` → `confirm_cli_selection`
  - 상태 / 핸들러: `tools/installer/src/app/mod.rs` → `select_cli` 가 탭 목록과 대상 디렉터리를 정하고 Loading 으로 넘긴다
  - 상태 / 핸들러: `tools/installer/src/app/types.rs` → `for_cli` 가 Codex 에 Skills · Config · MCP 세 탭만 준다
  - 데이터: 대상 디렉터리는 `tools/installer/src/target.rs` → `get_dest_dir` 가 ~/.claude 또는 ~/.codex 로 정한다
- **테스트**: `tools/installer/src/app/navigation.rs`
- **함정**: 목록에서 돌아와 다시 고른 뒤 Space 가 아무것도 체크하지 않음 → 재스캔 결과가 짧아 커서 인덱스가 범위를 벗어남, `select_cli` 에서 세 인덱스를 0으로 되돌려 막는다, 근거: `tools/installer/src/app/mod.rs` → `select_cli`
- **공유 의존**: `tools/installer/src/theme.rs`
- **관련**: `startup-loading`, `sources-screen`, `exit-confirm`

### `startup-loading`: 실행 직후와 CLI 선택 직후의 로딩

- **위치**: 화면 가운데 스피너 상자, 제목 `Config Installer`
- **별칭**: 로딩, 스피너, 멈춤, Scanning, 시작이 느림
- **UI 문구**: `"Loading..."`, `"Scanning components"`, `"Loading {} configuration..."`, `"Loading...  [q] Quit"`, `"Error loading: {}"`, `"Loading failed"`, `"Unexpected refresh payload during load"`, `"Auto-cleaned {} outdated item(s): {}"`, `"Selected {}"`, `"Removed orphaned bundled cache (~/.hibi/cache/bundled)"`, `"Failed to clean bundled cache: {}"`, `"Cannot find source directory"`
- **컨트롤**: 로딩 중 q 만 받는다, `handle_loading_view` 에서 `should_quit`
- **코드 경로**:
  - UI: `tools/installer/src/main.rs` → `main` 이 `App::new` 를 백그라운드 스레드로 돌리며 `tools/installer/src/ui/loading_screen.rs` → `draw` 를 그린다
  - UI: `tools/installer/src/ui/mod.rs` → `render_loading_screen`
  - 상태 / 핸들러: `tools/installer/src/app/mod.rs` → `load_init_data` 가 소스 디렉터리 탐색, 소스 해석, sources.yaml 읽기를 한다
  - 상태 / 핸들러: `tools/installer/src/source/mod.rs` → `find_package_source_dir` 가 exe 옆, ../share/hibi, 현재 디렉터리 순으로 번들 설정을 찾는다
  - 상태 / 핸들러: `tools/installer/src/loading/initial_load.rs` → `start_loading_thread` 가 deprecated 훅 정리, 컴포넌트 · MCP · 플러그인 스캔을 한 번에 한다
  - 상태 / 핸들러: `tools/installer/src/loading/initial_load.rs` → `handle_loading_view` 가 결과를 받아 `finish_loading` 을 부른다
  - 데이터: ~/.hibi/cache/bundled 삭제는 `tools/installer/src/source/git.rs` → `cleanup_bundled_cache`, ~/.claude/hooks 의 deprecated 바이너리 삭제는 `tools/installer/src/fs/installer/mod.rs` → `auto_cleanup_deprecated_hooks`
- **테스트**: none
- **함정**: 로딩이 실패해도 에러 없이 첫 화면으로 돌아옴 → `Error loading:` 을 status_message 에 넣지만 CLI 선택 화면은 상태 바 없이 전체 화면으로 그린다, 근거: `tools/installer/src/ui/mod.rs` → `render_full_screen`
- **함정**: sources.yaml 이 깨져도 경고가 안 보임 → 시작 경고를 status_message 에 담지만 CLI 를 고른 뒤의 `finish_loading` 이 정리된 훅 목록이나 `Selected` 문구로 덮어쓴다, 근거: `tools/installer/src/app/mod.rs` → `finish_loading`
- **함정**: 설치해 둔 훅이 실행할 때마다 사라짐 → 번들 훅의 hook.yaml 이 전부 `deprecated: true` 라 시작 스캔이 설치본 바이너리를 지우고 settings.json 등록도 푼다, 근거: `tools/installer/src/fs/installer/mod.rs` → `auto_cleanup_deprecated_hooks`
- **함정**: 바이너리를 다른 곳에 복사하면 `Cannot find source directory` 로 시작 실패 → 번들 설정을 실행 파일 기준 상대 경로로만 찾는다, 근거: `tools/installer/src/source/mod.rs` → `find_package_source_dir`
- **공유 의존**: `tools/installer/src/fs/scanner/mod.rs`, `tools/installer/src/source/mod.rs`
- **관련**: `cli-picker`, `component-list`, `mcp-list`, `install-script`

### `cli-flags`: TUI 없이 쓰는 명령행 옵션

- **위치**: 터미널에서 `hibi --help`, `hibi --version`, `hibi --sync`
- **별칭**: 버전 확인, -v, -h, 도움말 옵션, 동기화, sync
- **UI 문구**: `"Usage: hibi [OPTIONS]"`, `"Show this help message"`, `"Sync git sources without TUI"`, `"Run without options to launch the interactive installer."`, `"(Config Installer)"`, `"Syncing sources..."`, `"to install changes."`, `"bundled: local"`, `": stale"`, `": updated"`, `"re-resolve failed: {}"`, `"Warning: failed to clean bundled cache: {}"`
- **컨트롤**: 없음, 인자 파싱은 `tools/installer/src/main.rs` → `main` 에서 `args.iter().any` 로 한다
- **코드 경로**:
  - 상태 / 핸들러: `tools/installer/src/cli/mod.rs` → `print_help`
  - 상태 / 핸들러: `tools/installer/src/cli/mod.rs` → `run_sync` 가 `tools/installer/src/source/mod.rs` → `sync_all_sources` 를 부르고 요약만 찍는다
  - 데이터: 버전 문자열은 `tools/installer/src/fs/mod.rs` → `VERSION`, 값은 `tools/installer/Cargo.toml` 의 version
- **테스트**: `tools/installer/src/source/mod.rs`
- **함정**: `hibi --sync` 뒤에도 ~/.claude 가 그대로임 → 동기화는 소스 캐시만 갱신하고 설치는 하지 않으며 마지막 줄로 `hibi` 실행을 안내한다, 근거: `tools/installer/src/cli/mod.rs` → `run_sync`
- **함정**: `--sync` 가 `updated` 라고 찍는데 새 커밋이 안 들어옴 → sources.yaml 의 `auto_update: false` 면 캐시가 있을 때 fetch 없이 캐시를 그대로 쓰고 stale 도 아니어서 updated 로 보고한다, 근거: `tools/installer/src/source/mod.rs` → `resolve_git`
- **공유 의존**: `tools/installer/src/source/git.rs`
- **관련**: `source-sync`, `release-workflow`
