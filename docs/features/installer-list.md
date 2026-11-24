# 인스톨러 목록 화면

목록 화면은 세 줄 레이아웃이다: 상단 탭 바 `[1]`, 본문 `[2]`, 하단 상태 바. TUI 검색은 `dispatch_terminal_key`가 처리한다. 일반 목록 키는 `handle_list_input`으로 전달한다. 전역 키 q · t · ? · 1 · 2 · Tab 이 먼저, 그다음 포커스된 pane 의 키가 처리된다.

탭이나 목록에서 `/`를 누르면 현재 탭을 검색한다. 입력 즉시 파일 이름, 경로, 출처 또는 MCP와 플러그인 정보를 검색한다. 상태 필터와 숨김 설정도 함께 적용한다. Enter로 결과를 탐색하고 Esc로 검색어를 지운다. 검색 결과 밖의 체크는 유지되며 설치와 제거에서 제외된다.

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
- **상태 필터**: 본문에서 `f`로 `All → Modified → External → New → All` 순서로 전환한다. `New`는 미설치 파일이다. 파일 탭 사이에 필터가 유지되고 CLI를 다시 선택하면 `All`로 초기화된다. MCP와 Plugins에는 적용하지 않는다
- **전체 선택 토글**: `a`는 현재 탭에서 필터와 숨김 표시 상태에 맞는 항목을 모두 선택한다. 그 범위가 이미 모두 선택되어 있으면 해제한다. 같은 탭에서 범위 밖에 있는 선택도 해제하고 다른 탭의 선택은 유지한다. 접힌 폴더 안의 일치 파일도 포함한다. MCP와 Plugins도 `a`로 전체 선택과 해제를 전환한다. 폴더 선택과 `i`, `r`도 필터와 숨김 표시 상태에 맞는 파일만 대상으로 한다
- **필터 표시**: 제목에 `Filter:`, `Files:`, `Selected:`를 표시한다. 결과가 없으면 `Press f to change filter.`로 안내한다. 필터 전환 자체는 체크 상태를 바꾸지 않는다
- **전체 폴더 토글**: 본문에서 `E`는 현재 탭의 폴더가 모두 펼쳐져 있으면 모두 접고, 아니면 모두 펼친다. 체크와 상태 필터, 숨김 설정은 유지한다. 모두 접으면 현재 항목의 최상위 폴더로 커서를 옮긴다. 탭 바와 MCP, Plugins에는 적용하지 않는다
- **폴더 숨김**: 본문에서 폴더에 `x`를 누르면 파일 상태와 관계없이 확인 레이어를 연다. `X`는 현재 탭과 상태 필터에서 완전히 체크한 폴더들을 모은다. 상위 폴더가 포함되면 그 안의 폴더는 중복으로 세지 않는다. 일부만 체크한 폴더는 제외한다. `.system`이나 `synced` 같은 이름을 자동으로 숨기지는 않는다
- **숨김 확인**: 레이어에 대상 폴더 목록과 해제할 체크 수를 표시한다. 긴 경로는 줄바꿈하며 `j`, `k`로 본문 전체를 스크롤한다. `y`로 확인하면 저장 성공 후 대상 폴더의 체크를 해제하고 숨긴다. `Esc`, `n`, `q`는 취소하며 저장값과 체크를 유지한다. `H`로 숨긴 폴더를 표시하고 `[hidden]` 폴더에서 `x`를 누르면 숨김을 취소한다. 체크는 자동 복원하지 않는다
- **숨김 저장**: `~/.hibi/visibility.json`에 대상 디렉터리와 파일 종류를 포함한 폴더 경로를 저장한다. 재실행해도 유지되고 Claude와 Codex는 따로 적용된다. 잠금으로 동시 저장을 막고 임시 파일을 교체한다. 다른 installer가 저장 중이면 재시도를 안내한다. 저장 실패 시 표시를 유지하고 오류를 알린다
- **숨김과 처리**: 실제 파일과 AI 서비스 동작은 바꾸지 않는다. 숨긴 항목이 표시되지 않는 동안 `a`와 설치·제거 대기열에서 제외한다. `H`로 표시한 뒤 명시적으로 체크하면 설치나 제거 대상에 포함할 수 있다. 다시 `H`로 감추면 모든 파일 탭과 상태 필터에 걸쳐 숨긴 항목의 체크를 해제한다. 일반 항목의 선택과 숨김 저장값은 유지하며 다시 표시해도 체크는 복원하지 않는다. 숨긴 폴더에는 이후 소스에 추가된 파일과 상태가 바뀐 파일도 표시하지 않는다. `Hidden:`은 현재 대상과 탭에 저장된 숨김 설정 수이며 폴더가 사라져도 설정은 유지한다
- **UI 문구**: `"[x]"`, `"[-]"`, `"[ ]"`, `"new"`, `"modified"`, `"installed"`, `"managed"`, `"external"`, `"[DEPRECATED]"`, `"[{}]"`, `"No items selected"`
- **컨트롤**: `"move the cursor"` → `next_item`, `"collapse folder, or parent"` → `handle_folder_collapse`, `"expand folder"` → `expand_folder`, `"toggle selection"` → `toggle_selected`, `"toggle all shown selections"` → `toggle_all_selected`, `"toggle all folders open / closed"` → `toggle_all_folders`, `"file filter: All/Modified/External/New"` → `cycle_component_filter`, `"expand folder, or diff file"` → `handle_enter`, `"install selected in filter"` → `install_selected`, `"remove selected in filter"` → `removal_confirmation_count`, `y` → `remove_confirmed`
- **코드 경로**:
  - UI: `tools/installer/src/ui/list.rs` → `render_tree_node`
  - 상태 / 핸들러: `tools/installer/src/cli/mod.rs` → `handle_content_focus_keys`
  - 상태 필터: `tools/installer/src/app/filter.rs` → `current_components`, `rebuild_tree_views`. 표시와 설치·제거 대기열에 같은 필터를 적용한다
  - 폴더 숨김: `tools/installer/src/app/visibility.rs` → `toggle_folder_visibility`, `confirm_folder_hide`, `toggle_show_hidden`. 일괄 숨김과 폴더 집계는 `tools/installer/src/app/visibility/folders.rs`의 `hide_selected_folders`와 `folder_states`가 맡는다. 화면과 대기열은 `visible_components`를 사용한다. 확인 레이어는 `tools/installer/src/ui/confirm_hide.rs`, 저장은 `tools/installer/src/fs/installer/visibility.rs`
  - 상태 / 핸들러: `tools/installer/src/app/selection.rs` → `toggle_selected`, 폴더면 하위 전체를 뒤집는다
  - 상태 / 핸들러: `tools/installer/src/tree/build.rs` → `build_from_components`, 이동은 `tools/installer/src/tree/navigate.rs`
  - 데이터: 스캔은 `tools/installer/src/fs/scanner/mod.rs` → `scan_all_sources`, 상태 판정은 `tools/installer/src/fs/scanner/components.rs` → `determine_status`, 소스 없는 설치본은 `tools/installer/src/fs/scanner/external.rs` → `scan_externals`
- **테스트**: `tools/installer/src/cli/tests.rs` → `status_filter`, `tools/installer/src/ui/tests.rs`, `tools/installer/src/fs/scanner/components.rs`, `tools/installer/src/fs/scanner/external.rs`, `tools/installer/src/fs/scanner/mod.rs`, `tools/installer/src/tree/build.rs`, `tools/installer/src/tree/navigate.rs`, `tools/installer/src/tree/selection.rs`, `tools/installer/src/component.rs`
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
- **UI 문구**: `"[1/2] Pane  [/] Search  [?] Keys  [q] Quit"`, `"[j/k/↑/↓] Scroll  [q/Esc] Close"`, `"[Enter] Submit  [Esc] Cancel  [Backspace] Delete"`, `"[Enter] Confirm  [Esc] Cancel  [Backspace] Delete"`, `"[Enter/q] Close"`, `"Checking CLI...  [Esc] Cancel  [q] Quit"`
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
  - UI: `tools/installer/src/ui/confirm.rs` → `render`
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
- **플랫폼 보정**: 상태줄을 설치하거나 Config에서 설정 항목을 읽을 때 다른 OS용 hibi 기본 명령을 설치된 바이너리 경로로 맞춘다. `repair_bundled_statusline`은 명령 전체가 일치할 때만 적용하므로 사용자 정의 경로와 인수를 보존한다.
- **회귀 테스트**: `tools/installer/src/fs/installer/entries.rs`, `tools/installer/src/fs/installer/settings.rs` → `linux_registration_repairs_the_bundled_macos_command`, `tools/installer/src/fs/scanner/components.rs` → `fresh_statusline_scan_installs_only_the_native_binary`
- **함정**: 스타일을 설치해도 기본값이 안 바뀜 → 자동 등록은 `outputStyle` 이 비어 있을 때만 한다, 근거: `tools/installer/src/fs/installer/settings.rs` → `register_output_style_in_settings`
- **저장 오류**: 설정 파일이 깨져 있으면 기본값을 저장할 수 없는 이유를 목록 화면에 표시한다. `tools/installer/src/cli/mod.rs` → `handle_default_toggle`
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
- **함정**: CLI로 등록 상태를 조회할 수 없으면 목록에 `unknown`을 표시한다. Claude 로컬 스코프에서는 선택한 프로젝트 디렉터리에서 목록을 다시 조회한다
- **함정**: mcps.yaml 에 넣은 서버가 목록에 없음 → 이름 · URL · 명령 검증에 실패한 항목은 조용히 빠진다, 근거: `tools/installer/src/fs/scanner/mcp.rs` → `scan_with_installed`
- **함정**: Codex는 사용자 스코프를 사용한다. 설치 직전에 기존 등록을 조회하며 이름이 같으면 건너뛴다
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
  - 상태 / 핸들러: `tools/installer/src/app/processing.rs` → `complete_install_setup` 가 사용할 수 있는 환경 값 또는 앞서 입력한 값이 없는 항목을 묻는다
  - 데이터: Memory 저장 경로는 stdio 서버의 환경변수로 기록한다. Context7 키를 설정하면 HTTP 인증에 클라이언트 환경변수 이름을 사용한다. Atlassian Cloud는 원격 서버에서 OAuth로 인증한다
- **테스트**: `tools/installer/src/cli/tests.rs`
- **함정**: 값을 비운 채 Enter 를 누르면 아무 일도 없음 → 빈 값은 무시한다, 근거: `tools/installer/src/app/input.rs` → `env_input_submit`
- **함정**: 여러 서버를 고르면 입력은 서버별로 보관하며 다음 서버의 필수 입력까지 확인한 뒤 설치한다, 근거: `tools/installer/src/app/processing.rs` → `continue_mcp_install`
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
  - 데이터: 카탈로그 `src/plugins/plugins.yaml` 을 `tools/installer/src/plugin.rs` → `parse_plugins_yaml` 로 읽고, 설치 여부는 `tools/installer/src/fs/scanner/plugin.rs` → `get_installed_plugins` 가 Claude 설정 디렉터리의 `plugins/installed_plugins.json`에서 사용자 스코프 기록을 읽는다
- **테스트**: `tools/installer/src/plugin.rs`, `tools/installer/src/fs/scanner/validation.rs`
- **함정**: 같은 이름의 플러그인은 마켓플레이스로 구분한다. 설치 기록이 있는 플러그인은 비활성화해도 `installed`로 표시된다. 이전 형식에서는 사용자 설정의 `enabledPlugins`를 참조한다
- **함정**: plugins.yaml 의 항목이 목록에 없음 → 이름 · 마켓플레이스 검증 실패 항목은 조용히 빠진다, 근거: `tools/installer/src/fs/scanner/plugin.rs` → `scan_plugins`
- **공유 의존**: `tools/installer/src/fs/installer/process.rs`
- **관련**: `install-progress`

### `install-progress`: 사전 점검, 설치와 제거 진행

- **위치**: `i` 또는 `r` 뒤 본문 자리의 진행 화면, 위 제목 · `Progress` 막대 · `Log` 목록. MCP · Plugins 는 그 전에 `Preflight` 스피너가 뜬다
- **별칭**: 설치, 제거, 진행률, 로그, 설치 중, 취소, CLI not found, q 안 먹힘
- **UI 문구**: `"Checking {} availability..."`, `"Cannot start {}: {}"`, `"Preflight thread crashed"`, `"CLI reported: {}"`, `"Starting installation of {} items..."`, `"Starting removal of {} items..."`, `"✓ Complete"`, `"Refreshing status..."`, `"Installing..."`, `"Removing..."`, `"Progress"`, `"Log"`, `"[OK] {} complete!"`, `"[OK] Status refresh complete!"`, `"Completed with errors"`, `"Operation cancelled"`, `"[WARN] Cancelled by user"`, `"[ERR] Process thread crashed"`, `"[ERROR] Refresh failed: {}"`, `"CLI not found in PATH"`, `"Timed out after {}s trying to {} {}"`, `"Cancelled"`
- `i`는 선택 항목을 설치한다. 모든 탭에서 `r`은 제거할 항목 수를 먼저 보여 준다. `y`는 제거를 시작하고 Esc는 기존 체크를 유지한다.
- 진행 중 Esc는 `installing_key`에서 대기열을 비우고 실행 중인 작업에 취소를 요청한다. 화면에는 시도한 수와 전체 수가 남는다. 완료 후 Enter, q, Esc로 목록에 돌아간다.
- 최종 오류 수는 설치나 제거 뒤의 목록 갱신 실패도 포함한다. 로그에서 항목 처리 결과와 갱신 오류의 원인을 확인할 수 있다.
- **코드 경로**:
  - UI: `tools/installer/src/ui/installing.rs` → `render`, Preflight 상자는 `tools/installer/src/ui/mod.rs` → `render_preflighting_screen`
  - 상태 / 핸들러: `tools/installer/src/app/processing.rs` → `install_selected`, MCP · Plugins 만 `needs_cli_preflight`
  - 상태 / 핸들러: `tools/installer/src/loading/preflight.rs` → `handle_preflighting_view` 가 `tools/installer/src/fs/installer/mod.rs` → `preflight_cli_available` 를 8초 제한으로 돌린다
  - 상태 / 핸들러: `tools/installer/src/loading/install.rs` → `handle_installing_view` 가 큐에서 하나씩 `tools/installer/src/process_exec.rs` → `execute` 를 스레드로 띄운다
  - 상태 / 핸들러: 끝나면 `tools/installer/src/loading/scan.rs` → `start_refresh_thread` 가 현재 탭 범위만 다시 스캔한다
  - 데이터: 컴포넌트는 `tools/installer/src/fs/installer/mod.rs` → `install_component`, 외부 명령은 `tools/installer/src/fs/installer/process.rs` → `spawn_cancelable_process`
- **테스트**: `tools/installer/src/app/processing.rs`, `tools/installer/src/loading/channels.rs`, `tools/installer/src/loading/scan.rs`, `tools/installer/src/fs/mod.rs`
- **함정**: 설치 중 q 를 눌러도 안 꺼짐 → 진행 화면의 q 와 Enter 는 완료 뒤 닫기만 하고, 중단은 Esc 다, 근거: `tools/installer/src/loading/install.rs` → `installing_key`
- **함정**: claude 가 설치돼 있는데 `CLI not found in PATH` → 셸 rc 를 안 읽은 터미널이면 PATH 와 몇몇 알려진 위치만 찾는다, 근거: `tools/installer/src/fs/mod.rs` → `resolve_cli_program`
- **함정**: 컴포넌트 하나가 실패해도 진행이 계속됨 → 컴포넌트 오류는 `[ERR]` 로그 줄로만 남고 다음 항목으로 간다, 근거: `tools/installer/src/process_exec.rs` → `execute`
- **함정**: MCP 를 설치했는데 다른 탭 상태가 그대로임 → 새로고침은 시작한 탭의 범위만 다시 스캔한다, 근거: `tools/installer/src/loading/scan.rs` → `for_tab`
- **공유 의존**: `tools/installer/src/loading/channels.rs` → `ProcessingChannels`, `tools/installer/src/fs/mod.rs` → `run_with_timeout`
- **관련**: `settings-merge`, `install-manifest`, `mcp-env-input`

### `settings-merge`: settings.json 항목별 설치와 제거

- **위치**: Claude Code의 Config 탭에 있는 `settings.json` 그룹. 설치와 제거 결과는 `~/.claude/settings.json`에 저장한다.
- **별칭**: settings.json, 설정 키, 설정 병합, 설정 덮어씀, permissions
- **컨트롤**: 항목을 선택한 뒤 Install 또는 Remove를 누른다. Diff에서 해당 항목의 현재 값과 소스 값을 비교할 수 있다.
- TUI에서는 Space로 항목을 선택하고 `d`로 차이를 확인한다. `i`는 설치를 시작한다. `r`은 제거할 수 있는 항목 수를 표시하며, `y`로 제거하거나 Esc로 취소한다.
- **코드 경로**:
  - 데이터: 번들 원본은 `src/settings.json`이다. `tools/installer/src/fs/scanner/mod.rs` → `scan_all_sources`가 우선순위가 높은 소스의 파일을 고른다.
  - 데이터: `tools/installer/src/fs/installer/entries.rs`가 설정 키를 나열하고 선택한 키를 설치하거나 제거한다. `tools/installer/src/fs/installer/settings.rs`가 파일을 저장한다.
- **선택 단위**: `env.MCP_TIMEOUT`처럼 중첩된 키를 따로 표시한다. 배열과 `statusLine` 객체는 각각 값 하나로 취급한다. Claude Code의 Config 항목은 처음에 모두 선택 해제되어 있다.
- **설치**: 선택한 키에 소스 값을 쓴다. 현재 값이 달라진 경우에는 목록을 갱신한 뒤 다시 선택한다. 같은 키 아래 기존 객체에 하위 값이 있으면 해당 객체를 보존한다.
- **제거**: 현재 값이 소스 값과 같은 항목만 제거할 수 있다. 다른 설정 키와 파일 자체는 보존한다.
- **읽기 오류**: 설정을 읽을 수 없으면 해당 항목을 변경 불가로 표시한다. Diff에서 원인을 확인할 수 있다.
- **테스트**: `tools/installer/src/fs/scanner/mod.rs`, `tools/installer/src/fs/installer/entries.rs`, `tools/installer/tests/gui_smoke.py`
- **등록 실패**: 훅, 출력 스타일, 상태줄을 설치할 때 설정 등록이 실패하면 기존 파일을 복구한다. 처음 설치하는 항목은 새 파일을 제거한다.
- **파일 권한**: Linux에서 기존 `settings.json`을 갱신할 때 소유자, 그룹, 접근 ACL을 보존한다.
- **제거**: 개별 훅을 제거하면 등록할 때 사용한 명령과 일치하는 항목만 지운다. 설치한 상태줄을 제거하면 해당 실행 파일을 등록한 명령도 정리한다.
- **공유 의존**: serde_json, `tools/installer/src/fs/installer/settings.rs` → `edit_settings_file`
- **관련**: `default-style-statusline`, `commit-push-gating`, `response-language`

### `install-manifest`: ~/.hibi/install.json 출처 기록

- **위치**: 전용 화면은 없다. 컴포넌트 설치 · 제거 뒤 진행 로그에 실패 경고만 뜨고, 시작할 때 기록으로 정리한 결과는 목록 화면 상태 바의 `Auto-cleaned` 메시지에 붙는다
- **별칭**: install.json, 매니페스트, 출처, 어디서 설치했는지, 업스트림, 버전, 해시, Auto-cleaned, no longer shipped, unverified, 업그레이드 뒤 옛 파일이 안 지워짐, 파일이 저절로 사라짐
- **UI 문구**: `"[WARN] Install manifest not written: {}"`, `"Failed to replace {}"`, `"Failed to create {}"`, `"Auto-cleaned {} outdated item(s): {}"`, `"{removed} file(s) no longer shipped"`, `"{failed} file(s) no longer shipped could not be removed"`, `"kept {edited} edited file(s) no longer shipped"`, `"kept {unverified} unverified file(s) no longer shipped"`
- **컨트롤**: 없음
- **코드 경로**:
  - 상태 / 핸들러: 기록 전체는 `tools/installer/src/loading/scan.rs` → `start_refresh_thread` 의 Components 범위에서만 다시 쓴다
  - 상태 / 핸들러: 시작할 때 `tools/installer/src/loading/initial_load.rs` → `start_loading_thread` 가 기록을 읽어 이름을 바꾼 스킬과 더는 출시되지 않는 파일을 정리하고, 판정한 ID 를 `tools/installer/src/fs/manifest.rs` → `forget` 으로 기록에서 지운다
  - 데이터: 쓰기 `tools/installer/src/fs/manifest.rs` → `write`, 경로 `tools/installer/src/fs/manifest.rs` → `manifest_path`, 필드는 `tools/installer/src/fs/manifest.rs` → `InstallManifest`, 파일 해시는 `tools/installer/src/fs/manifest.rs` → `hashes_for`
  - 데이터: 코드의 읽기는 `tools/installer/src/fs/manifest.rs` → `recorded_install` 이고, 더는 출시되지 않는 파일의 판정은 `tools/installer/src/fs/installer/unshipped.rs` → `auto_cleanup_unshipped_files` 다. 업스트림 URL 과 버전은 모델이 읽는다, `src/commands/upstream-pr.md` 와 `src/skills/pull-request/references/upstream-config.md` 가 이 파일에서 읽으라고 지시한다
- **테스트**: `tools/installer/src/fs/manifest.rs`, `tools/installer/src/fs/installer/unshipped.rs`, `tools/installer/src/loading/scan.rs`
- **함정**: MCP 나 플러그인만 설치했는데 install.json 이 그대로임 → 매니페스트는 파일 기반 컴포넌트만 기록하고 그 새로고침에서만 쓴다, 근거: `tools/installer/src/fs/manifest.rs` → `InstallManifest`
- **함정**: 처음 실행하고 아무것도 설치하지 않으면 install.json 이 없음 → 첫 로딩은 `write` 를 부르지 않고 `forget` 은 있는 기록만 고친다, 근거: `tools/installer/src/loading/initial_load.rs` → `start_loading_thread`, `tools/installer/src/fs/manifest.rs` → `forget_in`
- **함정**: 업그레이드한 뒤 빠진 옛 파일이 지워지지 않고 `kept N unverified file(s) no longer shipped` 만 뜸 → 해시가 없는 옛 기록의 파일과 링크를 지나는 경로는 확인할 수 없어 남기고, 판정한 ID 는 기록에서 지워져 다음 실행에도 다시 보지 않는다, 근거: `tools/installer/src/fs/installer/unshipped.rs` → `judge`, `tools/installer/src/fs/installer/unshipped.rs` → `auto_cleanup_unshipped_files`
- **공유 의존**: `tools/installer/src/fs/mod.rs` → `VERSION`, `tools/installer/src/fs/mod.rs` → `normalize_line_endings`, sha2 크레이트
- **관련**: `release-workflow`, `commit-push-gating`, `startup-loading`

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
