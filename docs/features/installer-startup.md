# 인스톨러 시작 화면과 명령행

### `cli-picker`: 설치 대상 선택과 소스 관리

- **위치**: GUI의 시작 카드 또는 TUI의 `Select target` 상자
- **별칭**: 첫 화면, 대상 선택, Claude 선택, Codex 선택, 인스톨러
- **UI 문구**: `"Claude Code"`, `"Codex CLI"`, GUI `"Manage sources"` · ko `"소스 관리"`, TUI `"Manage Sources"`와 `"Select target"`
- **컨트롤**: 두 화면에서 Tab과 Shift+Tab으로 세 선택지를 순환하고 Enter로 연다. GUI는 좌우 방향키로 대상 카드를, 위아래 방향키로 카드와 소스 관리를 이동한다. TUI는 위아래 또는 j와 k를 사용한다.
- **코드 경로**:
  - GUI: `tools/installer/src/gui/home.rs` → `view`, 방향키는 `tools/installer/src/gui/shortcuts.rs` → `picker_arrow`
  - TUI: `tools/installer/src/ui/cli_selection.rs` → `render`, 키 입력은 `tools/installer/src/cli/mod.rs` → `handle_cli_selection`
  - 상태: `tools/installer/src/app/types.rs`의 `CliChoice`가 세 선택지를 표현한다. `tools/installer/src/app/mod.rs` → `select_cli`가 대상과 탭을 정하고 목록 커서를 초기화한다.
  - 데이터: `tools/installer/src/target.rs` → `get_dest_dir`가 `~/.claude`와 `~/.codex`를 정한다.
- **테스트**: `tools/installer/src/gui/keyboard_tests.rs`, `tools/installer/src/ui/tests.rs`, `tools/installer/src/app/navigation.rs`
- **함정**: 작은 터미널에서 배너와 옵션 설명이 줄어든다 → `render`가 높이에 맞춰 대상 이름과 조작 힌트를 남긴다, 근거: `tools/installer/src/ui/cli_selection.rs`
- **관련**: `startup-loading`, `sources-screen`, `exit-confirm`

### `startup-loading`: 실행 직후 초기화와 첫 스캔

- **위치**: 앱 초기화 화면과 CLI 선택 뒤의 로딩 화면
- **별칭**: 로딩, 스피너, 멈춤, Scanning, 시작이 느림
- **UI 문구**: `"Scanning components"`, `"Loading {} configuration..."`, `"Error loading: {}"`, `"Loading failed"`, `"Cannot find source directory"`
- **컨트롤**: TUI는 대상 선택 후 스캔 중 q로 닫을 수 있다. GUI의 초기화 오류 화면에서는 다시 시도하거나 창을 닫는다. 작업 중 GUI를 닫으면 정리를 기다린다.
- **코드 경로**:
  - GUI: `tools/installer/src/gui/runtime.rs` → `start_initialization`, 초기화 오류는 `tools/installer/src/gui/mod.rs`에서 표시한다.
  - TUI: `tools/installer/src/main.rs` → `run_terminal`가 백그라운드에서 앱을 만들고 `tools/installer/src/ui/loading_screen.rs` → `draw`를 실행한다.
  - 상태: `tools/installer/src/app/mod.rs` → `load_init_data`에서 번들과 소스를 해석한다. `tools/installer/src/loading/initial_load.rs` → `start_loading_thread`가 초기 정리와 세 유형의 스캔을 실행한다. `tick_loading`은 결과를 적용하고 MCP 범위 조회 경고를 상태 메시지에 추가한다.
  - 데이터: `tools/installer/src/source/mod.rs` → `find_package_source_dir`가 앱 번들 리소스, 실행 파일 주변의 패키지 경로와 개발용 현재 디렉터리를 확인한다. `tools/installer/src/source/git.rs`의 `cleanup_bundled_cache`와 `tools/installer/src/fs/installer/mod.rs`의 `auto_cleanup_deprecated_hooks`가 이전 배포물을 정리한다.
- **테스트**: `tools/installer/tests/cli_smoke.py`, `tools/installer/tests/gui_smoke.py`, `tools/installer/src/app/navigation.rs`
- **함정**: TUI 스캔 실패 후 대상 선택 화면으로 돌아가도 오류가 안 보임 → 스캔 오류를 상태 메시지에 저장하지만 대상 선택 화면은 별도 전체 화면이다, 근거: `tools/installer/src/ui/mod.rs` → `render_full_screen`
- **함정**: 구형 번들 훅이 실행 뒤 제거됨 → 초기 스캔에서 배포 설정에 deprecated로 표시한 훅을 정리한다, 근거: `tools/installer/src/fs/installer/mod.rs` → `auto_cleanup_deprecated_hooks`
- **함정**: 바이너리만 옮긴 뒤 `Cannot find source directory`로 실패 → 앱 리소스나 지원하는 상대 경로에 번들이 필요하다, 근거: `tools/installer/src/source/mod.rs` → `find_package_source_dir`
- **관련**: `component-list`, `mcp-list`, `install-script`

### `cli-flags`: 명령행 인자와 헤드리스 실행

- **위치**: `hibi --help`, `hibi --version`, `hibi --sync`, `hibi --tui`
- **별칭**: 버전 확인, -v, -h, 도움말, 동기화, sync, SSH, 화면 없이 실행
- **UI 문구**: `"Usage: hibi [OPTIONS]"`, `"Sync git sources without a window or TUI"`, `"Launch the terminal installer"`, `"No graphical display is available. Use --tui for terminal mode."`
- **코드 경로**:
  - `tools/installer/src/main.rs` → `main`이 도움말, 버전, 동기화 순으로 먼저 처리한다. 기본 빌드는 GUI를 열고 `--tui`는 터미널을 연다. `--no-default-features` 빌드는 인자 없이도 터미널을 연다.
  - `tools/installer/src/cli/mod.rs` → `print_help`와 `run_sync`. 동기화는 `tools/installer/src/source/mod.rs` → `sync_all_sources`를 호출한다.
  - `tools/installer/src/fs/mod.rs`의 `VERSION`은 `tools/installer/Cargo.toml`의 버전에서 가져온다.
- **테스트**: `tools/installer/tests/cli_smoke.py`, `tools/installer/src/main.rs`, `tools/installer/src/source/mod.rs`
- **함정**: `hibi --sync` 뒤에도 설치 파일이 그대로임 → 수동 동기화는 `auto_update: false`여도 소스를 가져오며 설치 파일 갱신은 대화형 설치에서 실행한다, 근거: `tools/installer/src/cli/mod.rs` → `run_sync`
- **함정**: GUI를 열 디스플레이가 없는 Linux에서 실행 실패 → 기본 시작 전에 디스플레이 설정을 확인하고 `--tui`를 안내한다, 근거: `tools/installer/src/main.rs` → `require_graphical_display`
- **관련**: `source-sync`, `release-workflow`
