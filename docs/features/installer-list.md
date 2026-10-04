# 인스톨러 목록 화면

목록 화면은 세 줄 레이아웃이다: 상단 탭 바 `[1]`, 본문 `[2]`, 하단 상태 바. 키 입력은 전부 `tools/installer/src/cli/mod.rs` → `handle_list_input` 을 거친다. 전역 키 q · t · ? · 1 · 2 · Tab 이 먼저, 그다음 포커스된 pane 의 키가 처리된다.

### `tab-bar`: 상단 탭 바와 pane 포커스

- **위치**: 목록 화면 맨 위, 테두리 제목 `[1]-Claude Code Config Installer` 형태
- **별칭**: 탭, 탭 바, 탭 전환, 탭이 잘림, 숫자 키, pane, 포커스
- **UI 문구**: `"Agents"`, `"Commands"`, `"Contexts"`, `"Rules"`, `"Skills"`, `"Hooks"`, `"Styles"`, `"Statusline"`, `"Config"`, `"MCP"`, `"Plugins"`, `"Config Installer ({})"`, `"[{}]-{}"`, `"‹"`, `"›"`
- **컨트롤**: `"focus the tab bar"` → `focus_tabs`, `"focus the content list"` → `focus_content`, `"toggle between panes"` → `toggle_focus`, `"previous tab"` → `prev_tab`, `"next tab"` → `next_tab`, `"back to the list"` → `focus_content`
- **코드 경로**:
  - UI: `tools/installer/src/ui/tabs.rs` → `build_visible_tabs` 가 폭이 모자라면 선택 탭 주변만 남기고 ‹ › 를 붙인다
  - UI: `tools/installer/src/ui/mod.rs` → `pane_title` 과 `pane_border_style`
  - 상태 / 핸들러: `tools/installer/src/cli/mod.rs` → `handle_tab_focus_keys`
  - 상태 / 핸들러: `tools/installer/src/app/navigation.rs` → `next_tab`
  - 데이터: 숫자 키는 `tools/installer/src/app/types.rs` → `shortcut` 한 곳에서 정한다
- **테스트**: `tools/installer/src/ui/tabs.rs`, `tools/installer/src/ui/tests.rs`, `tools/installer/src/cli/tests.rs`, `tools/installer/src/app/navigation.rs`
- **함정**: 좁은 터미널에서 탭이 덜 보이거나 남은 칸이 생김 → 폭을 바이트로 재서 화살표 같은 다중 바이트 글자를 과대 계산했다, 근거: `9580428`
- **함정**: 탭 바에 포커스가 있을 때 ↑ 이나 k 가 반응 없음 → 의도된 no-op 이다, 근거: `tools/installer/src/cli/mod.rs` → `handle_tab_focus_keys`
- **함정**: Codex 를 고르면 Agents · Commands · Plugins 탭이 없음 → Codex 대상은 Skills · Config · MCP 만 연다, 근거: `tools/installer/src/app/types.rs` → `for_cli`
- **공유 의존**: `tools/installer/src/ui/layout.rs` → `columns`
- **관련**: `status-bar`, `help-overlay`

### `component-list`: 컴포넌트 트리 목록과 선택

- **위치**: 본문 pane, 테두리 제목 `[2]-Agents` 처럼 현재 탭 이름. MCP · Plugins 탭은 별도 목록이다
- **별칭**: 목록, 체크박스, 전체 선택, 폴더 펼치기, 에이전트, 커맨드, 스킬, modified, external
- **UI 문구**: `"[x]"`, `"[-]"`, `"[ ]"`, `"new"`, `"modified"`, `"installed"`, `"managed"`, `"external"`, `"[DEPRECATED]"`, `"[{}]"`, `"No items selected"`
- **컨트롤**: `"move the cursor"` → `next_item`, `"collapse folder, or parent"` → `handle_folder_collapse`, `"expand folder"` → `expand_folder`, `"toggle selection"` → `toggle_selected`, `"select all / none"` → `select_all`, `"expand folder, or diff file"` → `handle_enter`, `"install selected"` → `install_selected`, `"remove selected"` → `remove_selected`
- **코드 경로**:
  - UI: `tools/installer/src/ui/list.rs` → `render_tree_node`
  - 상태 / 핸들러: `tools/installer/src/cli/mod.rs` → `handle_content_focus_keys`
  - 상태 / 핸들러: `tools/installer/src/app/selection.rs` → `toggle_selected`, 폴더면 하위 전체를 뒤집는다
  - 상태 / 핸들러: `tools/installer/src/tree/build.rs` → `build_from_components`, 이동은 `tools/installer/src/tree/navigate.rs`
  - 데이터: 스캔은 `tools/installer/src/fs/scanner/mod.rs` → `scan_all_sources`, 상태 판정은 `tools/installer/src/fs/scanner/components.rs` → `determine_status`, 소스 없는 설치본은 `tools/installer/src/fs/scanner/external.rs` → `scan_externals`
- **테스트**: `tools/installer/src/fs/scanner/components.rs`, `tools/installer/src/fs/scanner/external.rs`, `tools/installer/src/fs/scanner/mod.rs`, `tools/installer/src/tree/build.rs`, `tools/installer/src/tree/navigate.rs`, `tools/installer/src/tree/selection.rs`, `tools/installer/src/component.rs`
- **함정**: 고친 적 없는 파일이 modified 로 보임 → CRLF 와 LF 차이를 변경으로 셌다, 지금은 비교 전에 `\r` 을 지운다, 근거: `87aaf93`
- **함정**: 화면을 열면 new · modified 항목이 이미 체크돼 있어 i 한 번에 덮어씀 → installed 와 external 만 빼고 기본 선택된다, 근거: `tools/installer/src/component.rs` → `new`
- **함정**: external 항목을 체크하고 i 를 눌러도 설치 안 됨 → 소스가 없는 설치본은 제거만 가능하다, 근거: `tools/installer/src/component.rs` → `is_install_eligible`
- **함정**: Hooks 탭이 비어 있음 → 번들 훅이 전부 deprecated 이고 deprecated 훅은 설치본이 있을 때만 제거용으로 나온다, 근거: `tools/installer/src/fs/scanner/components.rs` → `scan_hook_entry`
- **함정**: 스킬 폴더의 `workspace` 하위나 `-ko` 파일이 목록에 없음 → 스캐너가 둘 다 건너뛴다, 근거: `tools/installer/src/fs/scanner/components.rs` → `scan_directory`
- **공유 의존**: `tools/installer/src/component.rs`, `tools/installer/src/fs/scanner/mod.rs` → `merge_scanned` 는 같은 키를 나중 소스가 덮는다
- **관련**: `install-progress`, `diff-view`, `sources-screen`

### `status-bar`: 하단 키 힌트와 상태 메시지

- **위치**: 목록 화면 맨 아래 한 줄, 왼쪽 키 힌트, 그 옆 노란 상태 메시지, 오른쪽 끝 버전
- **별칭**: 하단 바, 상태 바, 상태 메시지, 버전
- **UI 문구**: `"[1/2] Pane  [?] Keys  [q] Quit"`, `"[j/k/↑/↓] Scroll  [q/Esc] Close"`, `"[Enter] Submit  [Esc] Cancel  [Backspace] Delete"`, `"[Enter] Confirm  [Esc] Cancel  [Backspace] Delete"`, `"[Enter/q] Close"`, `"Checking CLI...  [Esc] Cancel  [q] Quit"`
- **컨트롤**: 없음
- **코드 경로**:
  - UI: `tools/installer/src/ui/mod.rs` → `render_status_bar`, 뷰별 힌트는 `tools/installer/src/ui/mod.rs` → `status_help`
  - 데이터: 메시지는 `status_message` 한 칸이라 마지막으로 쓴 것만 남는다
- **테스트**: `tools/installer/src/ui/tests.rs`
- **함정**: 버전 표시가 오른쪽 끝에서 안쪽으로 밀림 → 패딩을 바이트 길이로 계산했다, 근거: `9580428`
- **함정**: 상태 바에 d · i · r 같은 키가 안 보임 → 80칸에서 잘려 Quit 이 사라지던 문제로 목록을 `?` 상자로 옮겼다, 근거: `tools/installer/src/ui/mod.rs` → `LIST_HELP`
- **공유 의존**: `tools/installer/src/ui/layout.rs` → `columns`
- **관련**: `help-overlay`, `tab-bar`

### `help-overlay`: `?` 키바인딩 표

- **위치**: 목록 위 가운데 상자, 제목 `Keybindings`, 아래 테두리에 닫기 키
- **별칭**: 단축키, 키 목록, 도움말, ?
- **UI 문구**: `"Keybindings"`, `"[j/k] Scroll  [?/Esc/q] Close"`, `"Panes"`, `"Tab bar"`, `"Content list"`, `"Global"`, `"this help"`
- **컨트롤**: `"this help"` → `open_help`, 상자 안에서 q · ? · Esc → `close_help`, j · ↓ → `scroll_help_down`, k · ↑ → `scroll_help_up`
- **코드 경로**:
  - UI: `tools/installer/src/ui/help.rs` → `SECTIONS`, 높이는 `tools/installer/src/ui/help.rs` → `box_height`
  - 상태 / 핸들러: `tools/installer/src/cli/mod.rs` → `handle_help_input`, 스크롤 한도는 `tools/installer/src/cli/mod.rs` → `help_max_scroll`
  - 상태 / 핸들러: `tools/installer/src/app/navigation.rs` → `open_help`
- **테스트**: `tools/installer/src/ui/tests.rs`, `tools/installer/src/cli/tests.rs`, `tools/installer/src/app/navigation.rs`
- **함정**: 도움말에서 q 를 눌렀는데 인스톨러가 안 꺼짐 → 상자 안의 q 는 닫기다, 근거: `tools/installer/src/cli/mod.rs` → `handle_help_input`
- **함정**: 새 키를 추가했는데 도움말에 없음 → 표는 수동 목록이라 `handle_list_input` 과 함께 고쳐야 한다, 근거: `tools/installer/src/ui/help.rs` → `SECTIONS`
- **공유 의존**: `tools/installer/src/ui/layout.rs` → `centered`
- **관련**: `status-bar`, `exit-confirm`

### `exit-confirm`: Esc 로 CLI 선택 화면 복귀 확인

- **위치**: 본문 pane 에서 Esc, 선택된 항목이 있을 때만 뜨는 노란 테두리 `Confirm` 상자
- **별칭**: Esc, 뒤로가기, 나가기, 선택 날아감, 확인 창
- **UI 문구**: `"Leave for the CLI picker?"`, `"Selected items will be discarded."`, `"[y] "`, `"[Esc] "`, `"Confirm"`
- **컨트롤**: `"back to the CLI picker"` → `request_exit_to_main`, y → `exit_to_main`, Esc · n → `cancel_exit`
- **코드 경로**:
  - UI: `tools/installer/src/ui/confirm_exit.rs` → `render`
  - 상태 / 핸들러: `tools/installer/src/cli/mod.rs` → `handle_confirm_exit`
  - 상태 / 핸들러: `tools/installer/src/app/navigation.rs` → `request_exit_to_main` 가 `tools/installer/src/app/selection.rs` → `has_selection` 으로 전 탭 선택을 본다
- **테스트**: `tools/installer/src/cli/tests.rs`, `tools/installer/src/ui/tests.rs`
- **함정**: 확인 상자에서 Enter 를 눌러도 안 나가짐 → y 만 확정한다, 근거: `tools/installer/src/cli/mod.rs` → `handle_confirm_exit`
- **함정**: 탭 바에서 Esc 를 누르면 나가지 않고 목록으로만 감 → 탭 바의 Esc 는 본문 포커스 복귀다, 근거: `tools/installer/src/cli/mod.rs` → `handle_tab_focus_keys`
- **함정**: 아무것도 안 골랐을 때 Esc 한 번에 바로 첫 화면 → 선택이 없으면 확인을 건너뛴다, 근거: `tools/installer/src/app/navigation.rs` → `request_exit_to_main`
- **공유 의존**: `tools/installer/src/ui/layout.rs` → `centered`
- **관련**: `cli-picker`, `help-overlay`

### `diff-view`: 원본과 설치본 비교

- **위치**: 본문 pane 을 대신하는 화면, 제목 `Diff: agents/architect.md` 형태
- **별칭**: diff, 비교, 변경사항 보기, 튕김
- **UI 문구**: `"Diff: {}"`, `"No diff available"`, `"(new file)"`, `"external file -- no source"`, `"Cannot display diff for binary files."`, `"Failed to read source file as UTF-8"`, `"Failed to read external file as UTF-8"`
- **컨트롤**: `"diff against installed"` → `show_diff`, `"expand folder, or diff file"` → `handle_enter`, j · k → `scroll_diff_down`, q · Esc → `close_diff`
- **코드 경로**:
  - UI: `tools/installer/src/ui/diff.rs` → `render`
  - 상태 / 핸들러: `tools/installer/src/cli/mod.rs` → `handle_diff_input`
  - 상태 / 핸들러: `tools/installer/src/app/settings.rs` → `show_diff`
  - 데이터: `tools/installer/src/fs/diff.rs` → `compare_files`
- **테스트**: `tools/installer/src/fs/diff.rs`
- **함정**: UTF-8 이 아닌 텍스트 파일에서 d 를 누르면 인스톨러가 꺼지고 `Error:` 가 찍힘 → 읽기 실패가 `?` 로 `run_app` 까지 올라가 루프를 끝낸다, 근거: `tools/installer/src/main.rs` → `run_app`
- **함정**: MCP 나 Plugins 탭에서 d 가 아무 반응 없음 → 두 탭은 diff 를 지원하지 않고 바로 반환한다, 근거: `tools/installer/src/app/settings.rs` → `show_diff`
- **함정**: j 를 계속 누르면 빈 화면까지 내려감 → diff 스크롤에는 상한이 없다, 근거: `tools/installer/src/app/settings.rs` → `scroll_diff_down`
- **함정**: Windows diff 의 경로 앞에 `\\?\` 가 붙어 보임 → 확장 길이 경로 접두사를 그대로 찍었다, 지금은 표시 전에 지운다, 근거: `8b14688`
- **공유 의존**: similar 크레이트
- **관련**: `component-list`

### `default-style-statusline`: Styles · Statusline 탭에서 기본값 지정

- **위치**: Styles 또는 Statusline 탭 본문, 제목 끝의 `[Default: ...]` 와 항목 옆 `★ DEFAULT`
- **별칭**: 기본 스타일, output style, 기본값 설정, DEFAULT, 상태줄 지정, 상태줄이 안 나옴
- **UI 문구**: `"[Default: {}]"`, `"[No default set]"`, `"★ DEFAULT"`, `"(not installed)"`, `"Set default output style: {}"`, `"Unset default output style"`, `"Set statusline: {}"`, `"Unset statusline"`, `"Switch to Styles tab to set default"`, `"Switch to Statusline tab to set default"`
- **컨트롤**: `"set / unset default"` → `handle_default_toggle`, s → `set_default_style` · `set_statusline`, u → `unset_default_style` · `unset_statusline`
- **코드 경로**:
  - UI: `tools/installer/src/ui/list.rs` → `render_tree`
  - 상태 / 핸들러: `tools/installer/src/app/settings.rs` → `set_default_style`
  - 데이터: 쓰기 `tools/installer/src/fs/installer/settings.rs` → `set_output_style`, `tools/installer/src/fs/installer/settings.rs` → `set_statusline`, 읽기 `tools/installer/src/app/settings.rs` → `read_current_settings`, 키는 settings.json 의 `outputStyle` 과 `statusLine.command`
- **테스트**: `tools/installer/src/fs/installer/settings.rs`
- **함정**: Windows 에서 상태줄이 비어 있음 → stdin 파이프 문제로 명령을 `cat | /c/...` 형태로 쓰며 Git Bash 가 필요하다, 근거: `5c584c7`
- **함정**: 스타일을 설치해도 기본값이 안 바뀜 → 자동 등록은 `outputStyle` 이 비어 있을 때만 한다, 근거: `tools/installer/src/fs/installer/settings.rs` → `register_output_style_in_settings`
- **함정**: settings.json 이 깨진 JSON 이면 s 를 누르는 순간 인스톨러가 꺼짐 → 파싱 실패가 `?` 로 `run_app` 까지 올라간다, 근거: `tools/installer/src/fs/installer/settings.rs` → `read_settings`
- **공유 의존**: `tools/installer/src/fs/installer/settings.rs`
- **관련**: `statusline-render`, `settings-merge`, `response-language`

### `mcp-list`: MCP 탭 목록과 설치 스코프

- **위치**: MCP 탭 본문, 제목 `MCP Servers (scope: user)`, 항목마다 두 줄
- **별칭**: MCP, MCP 서버, 스코프, local, 프로젝트 경로, Not Installed 로만 보임
- **UI 문구**: `"MCP Servers (scope: {})"`, `"No MCP servers found. Create mcps/mcps.yaml to add servers."`, `"[*]"`, `"not installed"`, `"⚠ env"`, `"MCP scope: {}"`, `"MCP scope: local ({})"`, `"MCP scope: user"`, `"Local Scope - Project Path"`, `"Set project path for local MCP installation:"`, `"[Enter] Confirm  [Esc] Cancel (revert to user scope)"`
- **컨트롤**: `"MCP scope (user / project)"` → `toggle_mcp_scope`, 경로 입력 Enter → `project_path_submit`, Esc → `project_path_cancel`, 글자 → `project_path_char`
- **코드 경로**:
  - UI: `tools/installer/src/ui/mcp_list.rs` → `render`, 경로 대화상자 `tools/installer/src/ui/project_path.rs` → `render`
  - 상태 / 핸들러: `tools/installer/src/app/input.rs` → `toggle_mcp_scope`
  - 상태 / 핸들러: `tools/installer/src/fs/installer/mcp.rs` → `install_mcp_server` 가 `claude mcp add --scope` 또는 `codex mcp add` 를 부른다
  - 데이터: 카탈로그 `src/mcps/mcps.yaml`, 설치 여부는 `tools/installer/src/fs/scanner/mcp.rs` → `get_installed_claude_servers` 가 `mcp list` 출력에서 읽는다
- **테스트**: `tools/installer/src/fs/scanner/validation.rs`, `tools/installer/src/fs/mod.rs`
- **함정**: Windows 에서 모든 MCP 가 not installed 로 보임 → npm 설치 CLI 의 `.cmd` shim 을 못 찾아 `mcp list` 가 비었다, 지금은 PATH 를 직접 뒤진다, 근거: `e06dad2`
- **함정**: CLI 를 못 찾아도 경고 없이 전부 not installed → `MCP scan failed` 경고를 스캔 결과에서 버린다, 근거: `tools/installer/src/loading/initial_load.rs` → `start_loading_thread`
- **함정**: mcps.yaml 에 넣은 서버가 목록에 없음 → 이름 · URL · 명령 검증에 실패한 항목은 조용히 빠진다, 근거: `tools/installer/src/fs/scanner/mcp.rs` → `scan_with_installed`
- **함정**: Codex 에서 o 로 local 을 골라도 효과 없음 → Codex 분기는 scope 와 project_path 를 쓰지 않는다, 근거: `tools/installer/src/fs/installer/mcp.rs` → `install_mcp_server`
- **공유 의존**: `tools/installer/src/fs/mod.rs` → `create_cli_command`, `tools/installer/src/fs/scanner/validation.rs`
- **관련**: `mcp-env-input`, `install-progress`

### `mcp-env-input`: MCP 환경변수 입력

- **위치**: MCP 탭 위 가운데 대화상자, 제목 `Environment Variables for context7 (1/2)` 형태
- **별칭**: 환경변수, env, API 키, 토큰 입력, 한글 입력
- **UI 문구**: `"Environment Variables for {} ({}/{})"`, `"Variable: "`, `"Value: "`, `"Collected:"`, `"[Enter] Submit  [Esc] Cancel"`
- **컨트롤**: Enter → `env_input_submit`, Esc → `env_input_cancel`, Backspace → `env_input_backspace`, 글자 → `env_input_char`
- **코드 경로**:
  - UI: `tools/installer/src/ui/env_input.rs` → `render`
  - 상태 / 핸들러: `tools/installer/src/cli/mod.rs` → `handle_env_input`
  - 상태 / 핸들러: `tools/installer/src/app/processing.rs` → `complete_install_setup` 가 셸 환경에 없는 변수만 묻는다
  - 데이터: 값은 메모리에만 있다가 `tools/installer/src/process_exec.rs` → `prepare` 를 거쳐 `-e KEY=VALUE` 인자로 넘어간다
- **테스트**: `tools/installer/src/cli/tests.rs`
- **함정**: 값을 비운 채 Enter 를 누르면 아무 일도 없음 → 빈 값은 무시한다, 근거: `tools/installer/src/app/input.rs` → `env_input_submit`
- **함정**: env 가 필요한 MCP 를 둘 이상 고르면 두 번째는 값 없이 설치됨 → 첫 서버의 입력이 끝나면 다른 서버를 다시 확인하지 않고 진행하며 값은 그 첫 서버에만 붙는다, 근거: `tools/installer/src/app/processing.rs` → `continue_mcp_install`
- **공유 의존**: `tools/installer/src/process_exec.rs`
- **관련**: `mcp-list`, `install-progress`

### `plugin-list`: Plugins 탭 목록

- **위치**: Plugins 탭 본문, 둘째 줄에 `anthropics/claude-plugins-official` 같은 저장소와 설명
- **별칭**: 플러그인, plugin, 마켓플레이스, marketplace
- **UI 문구**: `"No plugins found. Create plugins/plugins.yaml to add plugins."`, `"Plugins"`, `"[*]"`, `"not installed"`
- **컨트롤**: component-list 와 같은 Space · a · n · i · r
- **코드 경로**:
  - UI: `tools/installer/src/ui/plugin_list.rs` → `render`
  - 상태 / 핸들러: `tools/installer/src/fs/installer/plugin.rs` → `install_plugin`, 마켓플레이스 등록은 `tools/installer/src/fs/installer/mcp.rs` → `ensure_marketplace_added`
  - 데이터: 카탈로그 `src/plugins/plugins.yaml` 을 `tools/installer/src/plugin.rs` → `parse_plugins_yaml` 로 읽고, 설치 여부는 `tools/installer/src/fs/scanner/plugin.rs` → `get_installed_plugins` 가 ~/.claude/settings.json 의 `enabledPlugins` 에서 읽는다
- **테스트**: `tools/installer/src/plugin.rs`, `tools/installer/src/fs/scanner/validation.rs`
- **함정**: 설치했지만 꺼 둔 플러그인이 not installed 로 보임 → `enabledPlugins` 값이 true 인 것만 설치로 센다, 근거: `tools/installer/src/fs/scanner/plugin.rs` → `get_installed_plugins`
- **함정**: plugins.yaml 의 항목이 목록에 없음 → 이름 · 마켓플레이스 검증 실패 항목은 조용히 빠진다, 근거: `tools/installer/src/fs/scanner/plugin.rs` → `scan_plugins`
- **공유 의존**: `tools/installer/src/fs/installer/process.rs`
- **관련**: `install-progress`

### `install-progress`: 사전 점검, 설치와 제거 진행

- **위치**: `i` 또는 `r` 뒤 본문 자리의 진행 화면, 위 제목 · `Progress` 막대 · `Log` 목록. MCP · Plugins 는 그 전에 `Preflight` 스피너가 뜬다
- **별칭**: 설치, 제거, 진행률, 로그, 설치 중, 취소, CLI not found, q 안 먹힘
- **UI 문구**: `"Checking {} availability..."`, `"Cannot start {}: {}"`, `"Preflight thread crashed"`, `"CLI reported: {}"`, `"Starting installation of {} items..."`, `"Starting removal of {} items..."`, `"✓ Complete"`, `"Refreshing status..."`, `"Installing..."`, `"Removing..."`, `"Progress"`, `"Log"`, `"[OK] {} complete!"`, `"[OK] Status refresh complete!"`, `"[WARN] Cancelling current operation..."`, `"[WARN] Cancelled by user"`, `"[INFO] Cleaning up cancelled installation..."`, `"[ERR] Process thread crashed"`, `"[ERROR] Refresh failed: {}"`, `"CLI not found in PATH"`, `"Installation timed out after {}s (cleanup may be incomplete)"`, `"Cancelled"`
- **컨트롤**: `"install selected"` → `install_selected`, `"remove selected"` → `remove_selected`, 진행 중 Esc → `handle_installing_input` 가 취소 신호, 완료 후 Enter · q · Esc → `close_processing`, Preflight 중 Esc → `handle_preflighting_input`
- **코드 경로**:
  - UI: `tools/installer/src/ui/installing.rs` → `render`, Preflight 상자는 `tools/installer/src/ui/mod.rs` → `render_preflighting_screen`
  - 상태 / 핸들러: `tools/installer/src/app/processing.rs` → `install_selected`, MCP · Plugins 만 `needs_cli_preflight`
  - 상태 / 핸들러: `tools/installer/src/loading/preflight.rs` → `handle_preflighting_view` 가 `tools/installer/src/fs/installer/mod.rs` → `preflight_cli_available` 를 8초 제한으로 돌린다
  - 상태 / 핸들러: `tools/installer/src/loading/install.rs` → `handle_installing_view` 가 큐에서 하나씩 `tools/installer/src/process_exec.rs` → `execute` 를 스레드로 띄운다
  - 상태 / 핸들러: 끝나면 `tools/installer/src/loading/scan.rs` → `start_refresh_thread` 가 현재 탭 범위만 다시 스캔한다
  - 데이터: 컴포넌트는 `tools/installer/src/fs/installer/mod.rs` → `install_component`, 외부 명령은 `tools/installer/src/fs/installer/process.rs` → `spawn_cancelable_process`
- **테스트**: `tools/installer/src/app/processing.rs`, `tools/installer/src/loading/channels.rs`, `tools/installer/src/loading/scan.rs`, `tools/installer/src/fs/mod.rs`
- **함정**: 설치 중 q 를 눌러도 안 꺼짐 → 진행 화면의 q 와 Enter 는 완료 뒤 닫기만 하고, 중단은 Esc 다, 근거: `tools/installer/src/loading/install.rs` → `handle_installing_input`
- **함정**: claude 가 설치돼 있는데 `CLI not found in PATH` → 셸 rc 를 안 읽은 터미널이면 PATH 와 몇몇 알려진 위치만 찾는다, 근거: `tools/installer/src/fs/mod.rs` → `resolve_cli_program`
- **함정**: 컴포넌트 하나가 실패해도 진행이 계속됨 → 컴포넌트 오류는 `[ERR]` 로그 줄로만 남고 다음 항목으로 간다, 근거: `tools/installer/src/process_exec.rs` → `execute`
- **함정**: MCP 를 설치했는데 다른 탭 상태가 그대로임 → 새로고침은 시작한 탭의 범위만 다시 스캔한다, 근거: `tools/installer/src/loading/scan.rs` → `for_tab`
- **공유 의존**: `tools/installer/src/loading/channels.rs` → `ProcessingChannels`, `tools/installer/src/fs/mod.rs` → `run_with_timeout`
- **관련**: `settings-merge`, `install-manifest`, `mcp-env-input`

### `settings-merge`: settings.json 병합, 훅 등록, Config 파일 설치

- **위치**: Config 탭의 `settings.json` · `CLAUDE.md` · `AGENTS.md` 항목과 Hooks 탭 설치 · 제거, 결과는 ~/.claude/settings.json
- **별칭**: settings.json, 설정 병합, 설정 덮어씀, permissions 날아감, CLAUDE.md 덮어씀, 훅
- **UI 문구**: `"managed"`, `"is deprecated and cannot be installed"`, `"Security: destination path contains '..' component"`
- **컨트롤**: component-list 의 `"install selected"` → `install_selected`, `"remove selected"` → `remove_selected`
- **코드 경로**:
  - 상태 / 핸들러: `tools/installer/src/fs/installer/mod.rs` → `install_component` 가 타입별로 나눈다
  - 데이터: 병합 `tools/installer/src/fs/installer/merge.rs` → `merge_settings_json`, 훅 등록 `tools/installer/src/fs/installer/settings.rs` → `register_hook_in_settings`, 제거 `tools/installer/src/fs/installer/settings.rs` → `remove_managed_settings_sections`
  - 데이터: 배포 원본은 `src/settings.json`, 상태 판정은 `tools/installer/src/fs/scanner/components.rs` → `add_config_files` 가 대상이 있으면 항상 managed 로 둔다
- **테스트**: `tools/installer/src/fs/installer/settings.rs`
- **함정**: settings.json 을 설치했더니 내 배열 설정이 원본 값으로 바뀜 → 병합은 객체만 깊게 합치고 `hooks` 가 아닌 배열은 통째로 덮는다, 근거: `tools/installer/src/fs/installer/merge.rs` → `merge_json_values`
- **함정**: CLAUDE.md 를 설치하면 내가 쓴 ~/.claude/CLAUDE.md 가 사라짐 → settings.json 만 병합하고 CLAUDE.md · AGENTS.md 는 그대로 복사한다, 근거: `tools/installer/src/fs/installer/mod.rs` → `install_component`
- **함정**: Config 탭에서 settings.json 을 제거하면 상태줄과 출력 스타일도 풀림 → `hooks` · `outputStyle` · `statusLine` 을 통째로 지운다, 근거: `tools/installer/src/fs/installer/settings.rs` → `remove_managed_settings_sections`
- **함정**: 훅 하나를 제거했는데 이름이 비슷한 다른 훅도 빠짐 → 명령 문자열에 훅 이름이 포함되기만 하면 지운다, 근거: `tools/installer/src/fs/installer/settings.rs` → `unregister_hook_from_settings`
- **공유 의존**: serde_json, `tools/installer/src/fs/installer/settings.rs` → `read_settings`
- **관련**: `default-style-statusline`, `commit-push-gating`, `response-language`

### `install-manifest`: ~/.hibi/install.json 출처 기록

- **위치**: 화면에는 없다. 컴포넌트 설치 · 제거 뒤 진행 로그에 실패 경고만 뜬다
- **별칭**: install.json, 매니페스트, 출처, 어디서 설치했는지, 업스트림, 버전
- **UI 문구**: `"[WARN] Install manifest not written: {}"`, `"Failed to replace {}"`, `"Failed to create {}"`
- **컨트롤**: 없음
- **코드 경로**:
  - 상태 / 핸들러: `tools/installer/src/loading/scan.rs` → `start_refresh_thread` 의 Components 범위에서만 쓴다
  - 데이터: 쓰기 `tools/installer/src/fs/manifest.rs` → `write`, 경로 `tools/installer/src/fs/manifest.rs` → `manifest_path`, 필드는 `tools/installer/src/fs/manifest.rs` → `InstallManifest`
  - 데이터: 읽기는 코드가 아니라 모델이 한다, `src/commands/upstream-pr.md` 와 `src/skills/pull-request/references/upstream-config.md` 가 이 파일에서 업스트림 URL 과 버전을 읽으라고 지시한다
- **테스트**: `tools/installer/src/fs/manifest.rs`, `tools/installer/src/loading/scan.rs`
- **함정**: MCP 나 플러그인만 설치했는데 install.json 이 그대로임 → 매니페스트는 파일 기반 컴포넌트만 기록하고 그 새로고침에서만 쓴다, 근거: `tools/installer/src/fs/manifest.rs` → `InstallManifest`
- **함정**: 처음 실행하고 아무것도 설치하지 않으면 install.json 이 없음 → 첫 로딩 스캔은 매니페스트를 쓰지 않는다, 근거: `tools/installer/src/loading/initial_load.rs` → `start_loading_thread`
- **공유 의존**: `tools/installer/src/fs/mod.rs` → `VERSION`
- **관련**: `release-workflow`, `commit-push-gating`

### `theme-toggle`: Mocha · Latte 테마 전환

- **위치**: 목록 화면 어디서나 `t`, 탭 바 제목 괄호 안에 테마 이름
- **별칭**: 테마, 다크모드, 라이트모드, 색
- **UI 문구**: `"Theme: {}"`, `"Mocha"`, `"Latte"`
- **컨트롤**: `"switch theme"` → `toggle`
- **코드 경로**:
  - 상태 / 핸들러: `tools/installer/src/cli/mod.rs` → `handle_list_input`
  - 데이터: 초기값은 `tools/installer/src/theme.rs` → `detect_system_theme` 가 터미널 배경 밝기로 고르고 저장하지 않는다
- **테스트**: none
- **함정**: 실행할 때마다 테마가 되돌아감 → 선택을 저장하지 않고 매번 배경 밝기로 다시 고른다, 근거: `tools/installer/src/theme.rs` → `detect_system_theme`
- **함정**: Sources 화면에서는 t 가 안 먹음 → `t` 는 목록 화면 키 처리에만 있다, 근거: `tools/installer/src/app/sources.rs` → `handle_sources_key`
- **공유 의존**: terminal-light 크레이트
- **관련**: `sources-screen`
