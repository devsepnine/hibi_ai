# 인스톨러 Sources 화면

첫 화면의 `Manage Sources` 로 들어가는 전체 화면이다. 탭 바와 상태 바 없이 목록과 자체 하단 안내만 그린다. 단계 대화상자는 이 목록 위에 겹쳐 그린다.

### `sources-screen`: 소스 목록과 삭제

- **위치**: Sources 전체 화면, 맨 위 줄은 항상 `bundled  [Built-in]`, 아래에 사용자 소스, 하단에 키 안내와 동기화 결과
- **별칭**: 소스, sources, sources.yaml, 소스 관리, 소스 삭제, q 안 먹힘, 라이트 테마에서 제목이 안 보임
- **UI 문구**: `"Sources"`, `"bundled"`, `"[Built-in]"`, `"[Git]"`, `"[Local]"`, `"(stale)"`, `"[a]Add"`, `"[e]Edit  [r]Remove"`, `"[f]Sync"`, `"[Esc]Back"`, `"Remove this source?"`, `"Source removed, but cache cleanup failed: {}"`, `"Failed to load sources.yaml: {}"`, `"Source dir lacks expected structure, skipping: {}"`, `"Failed to resolve source: {}"`, `"sources.yaml exceeds 64KB size limit"`
- **컨트롤**: `"[a]Add"` → `source_start_add`, `"[e]Edit"` → `source_start_edit`, `"[r]Remove"` → `source_start_remove`, `"[f]Sync"` → `source_start_sync`, `"[Esc]Back"` → `handle_sources_key`, 삭제 확인 y → `handle_source_confirm_key`
- **코드 경로**:
  - UI: `tools/installer/src/ui/sources.rs` → `render_list`, 삭제 확인은 `tools/installer/src/ui/source_wizard.rs` → `render_confirm_remove`
  - 상태 / 핸들러: `tools/installer/src/cli/mod.rs` → `dispatch_key` 가 Sources 계열 뷰를 `tools/installer/src/app/sources.rs` 로 넘긴다
  - 상태 / 핸들러: `tools/installer/src/app/sources.rs` → `handle_sources_key`
  - 데이터: ~/.hibi/sources.yaml, 쓰기 `tools/installer/src/source/config.rs` → `save_config`, 읽기 `tools/installer/src/source/config.rs` → `load_config` 를 `tools/installer/src/app/mod.rs` → `load_init_data` 와 `tools/installer/src/source/mod.rs` → `resolve_all_sources` 가 각각 부른다
  - 데이터: git 소스를 지우면 캐시 ~/.hibi/cache 하위도 `tools/installer/src/source/git.rs` → `remove_cache` 로 지운다
- **테스트**: `tools/installer/src/source/config.rs`, `tools/installer/src/source/git.rs`
- **함정**: Sources 화면에서 q 를 누르면 꺼지지 않고 첫 화면으로 감 → 이 화면의 q 는 Esc 와 같은 뒤로가기다, 근거: `tools/installer/src/app/sources.rs` → `handle_sources_key`
- **함정**: 라이트 테마에서 Sources 와 대화상자 제목이 안 보임 → 제목 색을 테마 색이 아닌 흰색으로 고정했다, 근거: `tools/installer/src/ui/sources.rs` → `render_list`
- **함정**: bundled 줄에서 e · r 이 반응 없음 → 번들 소스는 읽기 전용이라 안내에서도 빠진다, 근거: `tools/installer/src/ui/sources.rs` → `render_footer`
- **함정**: 같은 이름의 컴포넌트가 내 소스 것으로 바뀜 → sources.yaml 의 마지막 항목이 가장 높은 우선순위로 덮는다, 근거: `tools/installer/src/fs/scanner/mod.rs` → `merge_scanned`
- **공유 의존**: `tools/installer/src/source/config.rs` → `SourceEntry`
- **관련**: `source-wizard`, `source-sync`, `cli-picker`

### `source-wizard`: 소스 추가와 수정 단계

- **위치**: Sources 위 대화상자 순서. Git 은 `Add Source` → `Git URL` → `Git Branch (optional)` → `Subdirectory (optional)` → `Map To`, Local 은 `Local Path` 다음부터 같다
- **별칭**: 소스 추가, git 소스, 로컬 소스, 브랜치, root, map_to, 소스 수정, 숫자 키
- **UI 문구**: `"Add Source"`, `"Git repository"`, `"Local directory"`, `"Git URL"`, `"Git Branch (optional)"`, `"Local Path"`, `"Subdirectory (optional)"`, `"Map To"`, `"Map all files to (optional):"`, `"output-styles"`, `"[Enter] Skip  [Esc] Cancel"`, `"[Enter] Confirm  [Esc] Cancel"`, `"Git URL must not contain credentials"`, `"Only HTTPS git URLs are allowed"`, `"Branch name must not start with '-': {}"`, `"Invalid branch name: {}"`, `"Path traversal (..) not allowed in root"`, `"Path traversal (..) not allowed in source path: {}"`, `"Source path resolves to inside ~/.claude/: {}"`, `"Local source path does not exist: {}"`
- **컨트롤**: `"Git repository"` → `handle_source_type_key`, `"Local directory"` → `handle_source_type_key`, 입력 Enter → `source_input_submit`, Esc → `source_cancel`, Map To 의 숫자와 Enter → `finish_with_map_to`
- **코드 경로**:
  - UI: `tools/installer/src/ui/source_wizard.rs` → `render_text_input`, `tools/installer/src/ui/source_wizard.rs` → `render_map_to_select`
  - 상태 / 핸들러: `tools/installer/src/app/sources.rs` → `handle_source_input_key`
  - 상태 / 핸들러: `tools/installer/src/app/source_wizard.rs` → `submit_url`, `tools/installer/src/app/source_wizard.rs` → `submit_root`, `tools/installer/src/app/source_wizard.rs` → `save_entry`
  - 데이터: 검증 `tools/installer/src/source/config.rs` → `validate_git_url`, 저장 `tools/installer/src/source/config.rs` → `save_config`, 저장 뒤 `tools/installer/src/app/sources.rs` → `source_start_resolve`
- **테스트**: `tools/installer/src/source/config.rs`, `tools/installer/src/cli/tests.rs`
- **함정**: `git@github.com:...` 주소를 넣으면 credentials 오류 → `@` 가 들어간 URL 을 자격 증명으로 보고 거부하며 https 만 받는다, 근거: `tools/installer/src/source/config.rs` → `validate_git_url`
- **함정**: 소스를 수정하면 기존 map_to 가 사라짐 → Map To 단계는 기존 값을 미리 채우지 않고 Enter 는 map_to 없음으로 저장한다, 근거: `tools/installer/src/app/source_wizard.rs` → `finish_with_map_to`
- **함정**: map_to 에 rules · contexts 가 아닌 값을 손으로 쓰면 그 소스가 비어 보임 → 모르는 map_to 는 조용히 건너뛴다, 근거: `tools/installer/src/fs/scanner/mod.rs` → `parse_map_to`
- **함정**: 로컬 경로를 ~/.claude 안으로 주면 거부됨 → 설치 대상 안의 소스는 막는다, 근거: `tools/installer/src/source/config.rs` → `validate_local_path`
- **공유 의존**: `tools/installer/src/source/config.rs`
- **관련**: `sources-screen`, `source-sync`

### `source-sync`: 소스 동기화와 git 캐시

- **위치**: Sources 화면에서 `f`, 가운데 `Syncing sources...` 스피너 뒤 하단에 소스별 결과 줄
- **별칭**: 동기화, sync, 업데이트 받기, stale, 캐시, 오프라인, fetch 실패, 취소
- **UI 문구**: `"Syncing sources..."`, `"Resolved sources"`, `"Sync thread crashed"`, `"Cancelled"`, `"bundled: local"`, `": stale"`, `": updated"`, `"Git fetch failed, using stale cache: {}"`, `"git is not installed or not in PATH"`, `"timed out after {}s"`
- **컨트롤**: `"[f]Sync"` → `source_start_sync`, 동기화 중 q → `handle_source_syncing` 가 취소 신호를 보낸다
- **코드 경로**:
  - UI: `tools/installer/src/ui/source_wizard.rs` → `render_syncing`, 결과 줄은 `tools/installer/src/ui/sources.rs` → `render_footer`
  - 상태 / 핸들러: `tools/installer/src/app/sources.rs` → `source_start_sync` 가 스레드에서 `tools/installer/src/source/mod.rs` → `sync_all_sources` 를 돌리고 재스캔한다
  - 상태 / 핸들러: `tools/installer/src/app/sources.rs` → `check_source_sync`
  - 데이터: 캐시는 ~/.hibi/cache 하위, 경로 `tools/installer/src/source/git.rs` → `cache_path_for`, clone 과 fetch 는 `tools/installer/src/source/git.rs` → `clone_or_update`, 마지막 fetch 시각은 캐시 안 `.hibi_last_fetch`
- **테스트**: `tools/installer/src/source/mod.rs`, `tools/installer/src/source/git.rs`
- **함정**: 동기화 뒤에도 목록이 예전 상태 → 동기화가 소스 목록만 바꾸고 컴포넌트는 그대로 뒀다, 지금은 재스캔하지만 CLI 를 고르기 전 첫 화면에서 들어온 경우엔 여전히 건너뛴다, 근거: `c09cd8b`
- **함정**: 오프라인에서 동기화하면 `(stale)` 이 붙음 → fetch 실패 때 이전 clone 이 있으면 그것을 stale 로 쓴다, 근거: `tools/installer/src/source/mod.rs` → `resolve_git`
- **함정**: upstream 이 history 를 다시 쓴 뒤 동기화가 실패 → shallow 캐시라 ff merge 가 불가능했다, 근거: `f9b7b31`
- **함정**: 캐시 디렉터리 안에서 고친 파일이 동기화 때 사라짐 → 매번 `fetch --depth 1` 뒤 `reset --hard FETCH_HEAD` 를 한다, 근거: `tools/installer/src/source/git.rs` → `update_repo`
- **함정**: 동기화 중 q 를 눌러도 이미 시작한 git 명령은 끝까지 돈다 → 취소는 해석 시작 전에 한 번만 확인한다, 근거: `tools/installer/src/source/mod.rs` → `sync_all_sources`
- **공유 의존**: git 실행 파일, `tools/installer/src/source/git.rs` → `run_git_command`
- **관련**: `cli-flags`, `sources-screen`, `component-list`
