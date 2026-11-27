# 인스톨러 GUI 기능과 검수 기준

기본 실행 화면은 iced GUI다. GUI와 TUI는 `App`을 통해 검색, 선택과 작업을 공유한다. 파일 병합, 설치 기록과 소스 캐시는 같은 저장 경로를 사용한다.

## 실행 경로

| 명령 | 실행 대상 |
|---|---|
| `cargo run --locked --manifest-path tools/installer/Cargo.toml` | GUI |
| `cargo run --locked --manifest-path tools/installer/Cargo.toml -- --tui` | TUI |
| `cargo run --locked --manifest-path tools/installer/Cargo.toml -- --sync` | 창 없이 소스 동기화 |
| `cargo run --locked --manifest-path tools/installer/Cargo.toml -- --help` | 명령행 도움말 |
| `cargo run --locked --manifest-path tools/installer/Cargo.toml -- --version` | 버전 |
| `cargo build --locked --manifest-path tools/installer/Cargo.toml --no-default-features` | GUI 의존성을 제외한 터미널 전용 빌드 |

GUI를 실행하려면 데스크톱 디스플레이가 필요하다. GUI 시작에 실패하면 오류에 TUI 실행 방법이 표시된다.

## 방향키 탐색

GUI의 시작 화면은 좌우 방향키로 대상 카드를 고르고 위아래 방향키로 소스 관리와 대상 카드 사이를 이동한다. GUI와 TUI의 시작 화면에서 Tab과 Shift+Tab은 세 선택지를 순서대로 돌며 Enter는 현재 표시한 선택지를 연다.

왼쪽 Components에서 카테고리를 클릭하면 해당 영역이 키보드 포커스를 유지한다. 위아래 방향키로 카테고리를 이동하고 오른쪽 방향키나 Enter로 목록에 들어간다. 목록 항목을 클릭하면 위아래 방향키로 항목을 이동하고 왼쪽과 오른쪽 방향키로 폴더를 접거나 펼친다. 파일에서 왼쪽 방향키를 누르면 부모 폴더로 이동해 접는다.

`1`은 Components, `2`는 목록에 포커스를 둔다. Tab과 Shift+Tab은 두 영역을 전환한다. 현재 포커스는 해당 행과 영역 제목에 표시한다. 검색창 사용 뒤 탐색 영역이나 작업 버튼을 클릭하면 검색창의 입력 포커스를 해제한다. 검색 중 일반 입력과 좌우 방향키는 검색창에서 처리하며 Enter로 목록에 돌아간다. TUI의 가로 탭 키 규칙은 유지한다.

`tools/installer/src/gui/keyboard_tests.rs`는 GUI 키 라우팅과 실제 위젯 이벤트를 함께 검증한다. 사이드바 이동, 목록 이동, 폴더 접기와 펼치기, 검색창 포커스 해제와 확인창 뒤 입력 차단을 검사한다.

## 폴더 숨김의 검증 기준

숨김은 외부 파일과 관리 대상 파일에 같은 폴더 경로 규칙을 적용한다. 기존 숨김 설정도 파일 상태와 관계없이 적용하며 저장 형식은 유지한다. GUI의 Visibility 영역은 현재 대상과 카테고리에 저장된 숨김 폴더 수를 함께 표시한다.

Hide selected와 Unhide selected는 완전히 체크한 폴더 수를 표시하며, 대상이 없으면 비활성화한다. 혼합 선택에서도 버튼의 목표 상태를 선택 전체에 적용한다. 이미 해당 상태인 폴더를 다시 처리해도 반대로 바꾸지 않는다. Hide selected는 기존 확인창을 사용하고 저장 성공 뒤 체크를 해제한다. Unhide selected는 숨김 설정을 해제하고 체크를 유지한다. 선택한 상위 폴더의 숨김을 해제하면 그 아래에 저장된 숨김 설정도 해제한다. 숨긴 상위 폴더를 선택하지 않고 하위 폴더만 숨김 해제하려 하면 상위 폴더 선택을 안내한다.

| 요구사항 | 구현 경로 | 검증 경로 |
|---|---|---|
| 모든 파일 상태의 단일 숨김, 재실행과 숨김 해제 | `tools/installer/src/app/visibility.rs`의 `toggle_folder_visibility`, `is_component_hidden` | `tools/installer/src/cli/visibility/managed/tests.rs`의 `each_managed_status_can_be_hidden_persisted_and_unhidden_without_changing_files` |
| 완전히 체크한 혼합 폴더의 일괄 숨김 | `tools/installer/src/app/visibility/folders.rs`의 `folder_states`, `hide_selected_folders` | `tools/installer/src/cli/visibility/tests.rs`의 `batch_skips_partial_folders_and_hides_fully_selected_mixed_parents` |
| 숨긴 항목의 전체 선택과 설치 및 제거 제외 | `tools/installer/src/app/visibility.rs`, `tools/installer/src/app/filter.rs`의 `visible_components`, `tools/installer/src/app/processing.rs` | `tools/installer/src/cli/visibility/managed/tests.rs`의 `hidden_managed_items_are_excluded_from_bulk_selection_and_both_work_queues` |
| 취소와 저장 실패 시 선택과 표시 보존 | `tools/installer/src/app/visibility.rs`의 `confirm_folder_hide`, `cancel_folder_hide` | `tools/installer/src/cli/visibility/managed/tests.rs`의 `cancelling_or_failing_to_save_managed_hide_preserves_checks_and_visibility` |
| 확인 전 상태 변경 허용, 사라진 폴더는 일괄 저장 취소 | `tools/installer/src/app/visibility.rs`의 `confirm_folder_hide` | `tools/installer/src/cli/visibility/managed/tests.rs`의 `status_changes_between_request_and_confirmation_do_not_cancel_batch_hide`, `tools/installer/src/cli/visibility/tests.rs`의 `disappeared_second_batch_folder_cancels_entire_transaction` |
| 혼합 선택의 명시적 목표 상태, 반복 실행과 저장 실패 보존 | `tools/installer/src/app/visibility/folders.rs`, `tools/installer/src/gui/actions.rs`, `tools/installer/src/gui/list.rs` | `tools/installer/src/gui/visibility/tests.rs`의 실제 버튼 클릭과 저장 파일 검증 |

### `gui-filters`: 상태 선택과 숨김 표시

- **위치**: 목록 상단의 필터 메뉴
- **UI 문구**: `"Filters · Default"` · ko `"필터 · 기본값"`, `"Status"`, `"Visibility"`, `"Hidden only"`
- **코드 경로**: `tools/installer/src/gui/filters.rs` → `view`, 키 입력은 `tools/installer/src/gui/actions.rs`
- **테스트**: `tools/installer/src/gui/visibility/filters/tests.rs`, `tools/installer/src/gui/filters/dropdown/tests.rs`, `tools/installer/tests/gui_smoke.py`

GUI의 Filters 버튼은 Status와 Visibility를 하나의 오버레이에 모은다. 기본 표시는 Filters · Default이며, 세부 상태 수와 기본값에서 바꾼 Visibility 한 개를 합쳐 적용 조건 수를 표시한다. Status는 탭 대신 Default, Modified, External과 New를 체크하는 목록을 사용한다. 세부 상태는 OR로 합친다. Default를 체크하면 세부 선택을 초기화하며, 마지막 세부 상태를 해제해도 Default로 돌아간다. Default는 설치됨과 관리 중인 항목도 포함하므로 세부 상태 세 개를 모두 체크한 경우와 다르다.

필터 오버레이는 컴포넌트 목록의 위치와 크기를 바꾸지 않는다. 상태와 Visibility를 고르는 동안 메뉴를 유지한다. 버튼을 다시 누르거나 Esc 또는 메뉴 바깥 클릭으로 닫는다. 바깥 클릭은 메뉴만 닫고 뒤쪽 항목을 선택하지 않는다. `tools/installer/src/gui/filters/dropdown.rs`는 앵커 버튼과 오버레이를 분리하고 화면 가장자리에서 메뉴를 배치한다. 작은 창에서는 메뉴 안을 스크롤할 수 있다.

GUI에서는 `f`로 필터를 열거나 닫는다. 열린 필터에서는 위아래 방향키, Tab과 Shift+Tab으로 이동하고 Enter 또는 Space로 적용한다. Esc는 필터를 닫으며 열린 상태의 키는 목록의 설치와 제거 단축키로 전달되지 않는다.

오버레이의 표시 영역은 생성할 때마다 창에서 받는다. iced는 배치를 캐시하고 그리기 단계에서 오버레이를 다시 생성하므로 배치 단계에서만 영역을 저장하면 메뉴가 보이지 않는다. `tools/installer/src/gui/filters/dropdown/tests.rs`는 이 수명 주기를 재현하고 렌더링한 메뉴 배경의 픽셀 수를 검사한다.

Visibility는 같은 오버레이의 구분선 아래에서 Visible only, Include hidden과 Hidden only 중 하나를 고른다. 기본값인 Visible only는 숨긴 항목을 제외한다. Include hidden은 숨긴 항목과 일반 항목을 함께 표시하며, Hidden only는 숨긴 항목만 표시한다. 세 방식 모두 현재 상태와 검색 조건을 함께 적용한다. External과 Hidden only를 선택하면 숨긴 외부 파일만 표시하며 New나 Modified인 숨김 항목을 추가하지 않는다.

보기 방식을 바꿔도 숨김 설정 자체는 바꾸지 않는다. Hidden only에서 숨김을 해제한 항목은 목록에서 빠진다. 결과가 없으면 현재 보기에서 숨김 항목이 없다는 안내를 표시한다. Visible only로 돌아갈 때는 기존처럼 숨긴 항목의 체크를 해제한다. TUI의 H는 기존 숨김 제외와 함께 보기를 전환한다.

상태 선택의 단일 기준은 `tools/installer/src/app/filter.rs`의 `ComponentFilters`이며, 숨김 보기는 `tools/installer/src/app/visibility.rs`의 `HiddenVisibility`를 사용한다. 목록과 작업 대기열은 같은 선택을 사용한다. TUI의 `f`는 기존 단일 상태 순환을 유지한다. `tools/installer/src/gui/filters.rs`는 선택 목록을 그리며 `tools/installer/src/gui/visibility/filters/tests.rs`는 실제 위젯 클릭, Default 복귀와 숨김 표시의 교집합을 검증한다.

## 설치와 제거 대상

GUI와 TUI는 `tools/installer/src/app/selection.rs`에서 계산한 설치와 제거 대상 수를 표시하고 같은 판정으로 실행 대기열을 구성한다. 검색과 필터로 가려진 항목은 작업 범위에서 제외한다. Codex에 이미 등록한 MCP 서버는 설치 대상에서 제외한다. 미설치 파일과 설치 여부를 알 수 없는 MCP 서버 및 플러그인은 제거 대상에서 제외한다.

제거 확인창은 대상 이름을 표시하고 수정한 파일과 외부 파일을 구분한다. 취소하면 체크를 보존한다. 선택한 항목이나 작업 위치가 바뀌면 제거를 시작하기 전에 다시 확인해야 한다. TUI 흐름은 [목록 기능맵](installer-list.md)을 참고한다. 이 계약은 `tools/installer/src/app/removal.rs`의 대상 교체와 경로 변경 테스트, `tools/installer/src/cli/tests.rs`의 설치와 제거 실행 테스트로 검증한다.

## 기존 기능과 GUI의 대응

이 표는 기능 지원과 검수 범위를 정의한다. 테스트 실행 결과를 뜻하지 않는다. 자동 검증 결과는 실제 실행 로그에서 확인한다.

| 기존 기능 ID | GUI에서 유지할 동작 | 실패와 경계 검수 |
|---|---|---|
| `cli-picker` | Claude Code와 Codex 대상 선택 | 대상 전환 후 이전 목록 인덱스와 선택이 남지 않음 |
| `startup-loading` | 초기화와 스캔 중 대기 상태 | 소스 누락과 작업 스레드 종료를 오류로 표시하고 재시도 가능 |
| `cli-flags` | 도움말, 버전, 소스 동기화 명령 유지 | 디스플레이 없는 환경에서 명령행 경로 실행 |
| `tab-bar` | Claude의 모든 탭과 Codex의 Skills, Config, MCP 제공 | 작은 창에서 마지막 탭까지 접근 가능 |
| `component-list` | 체크, 전체 선택과 해제, 폴더 펼침과 접힘 | 빈 목록과 필터 밖의 항목 처리, 선택 후 필터 변경 |
| `component-list` | 상태 필터와 설치 상태 표시 | 수정됨, 외부 파일, 미설치 파일 구분 |
| `gui-search` | 현재 카테고리의 이름, 경로와 출처 검색 | 결과 없음, 한글 정규화, 상태 필터 조합, 가려진 선택의 작업 제외 |
| `component-list` | 파일 상태와 관계없는 폴더 숨김과 숨김 해제 | 관리 대상과 혼합 폴더, 저장 실패 시 체크와 표시 유지 |
| `status-bar` | 대상, 상태 메시지와 버전 표시 | 긴 경로와 여러 줄 오류가 잘리지 않음 |
| `app-update` | 오른쪽 아래 버전을 눌러 업데이트를 확인하고 설치를 승인 | 패키지 배포 대기, 작업 중 설치 차단, 서명 오류, 번들 교체 실패 후 복구 |
| `help-overlay` | 도움말 제공 | 긴 도움말 스크롤과 닫기 |
| `exit-confirm` | 선택을 버리기 전 확인 | 취소 후 체크 유지, 처리 중 창 닫기 보호 |
| `diff-view` | 설치본과 원본 차이 표시 | 새 파일, 외부 파일, 긴 파일, 읽기 오류 |
| `default-style-statusline` | 기본 스타일과 상태줄 설정 및 해제 | 미설치 항목과 설정 저장 실패 표시 |
| `mcp-list` | 서버 목록과 user, local 범위 | CLI 누락, 잘못된 프로젝트 경로, 빈 목록 |
| `mcp-env-input` | 필요한 환경변수 입력 | 빈 값 제출, 한글 붙여넣기, 여러 서버 입력, 취소 후 재시작 |
| `plugin-list` | 플러그인 선택과 설치 및 제거 | 마켓플레이스 등록 실패와 비활성 플러그인 |
| `removal-confirm` | 대상 이름, 수정·외부 표시, 제거 위치 확인 | 선택 변경 시 재확인, 취소 시 체크 보존 |
| `install-progress` | 사전 검사, 진행률과 로그, 설치 및 제거 | 첫 항목 실패, 중간 취소, 작업 스레드 종료, 새로고침 실패 |
| `settings-merge` | 설정 항목 선택, 값 비교, 설치와 제거 | 사용자 설정 보존과 권한 오류 |
| `install-manifest` | 설치 출처와 해시 기록 유지 | 기록 저장 실패, 수정된 옛 파일과 해시 없는 파일 보존 |
| `gui-appearance` | 다크 모드와 라이트 모드 전환, 언어 선택과 저장 | 모든 화면의 제목, 선택, 비활성 버튼과 오류 대비 |
| `sources-screen` | 번들과 사용자 소스 목록, 선택, 수정 및 제거 | 번들 편집과 삭제 금지, 목록 변경 뒤 오래된 인덱스 |
| `source-wizard` | Git과 로컬 소스 추가 및 수정 | HTTPS 외 URL, 자격증명, 잘못된 브랜치와 경로 거부 |
| `source-wizard` | 브랜치, 하위 루트, 컴포넌트 매핑 | 빈 선택 기본값, 기존 값 표시, 각 단계 취소, 저장 실패 재시도 |
| `source-sync` | 동기화 결과와 오래된 캐시 표시 | 오프라인, 취소, 재시도, 종료한 스레드, 동기화 후 목록 갱신 |
| `statusline-render` | 별도 상태줄 바이너리와 출력 유지 | 인스톨러 GUI 의존성 없이 기존 바이너리 실행 |
| `packaging` | 같은 `hibi` 실행 파일 배포 | GUI 기본 빌드와 터미널 전용 빌드 구분 |
| `component-lookup` | 기존 배포 파일과 한국어 미러 제외 유지 | 미러가 사용자 설치 대상으로 들어가지 않음 |

### `gui-search`: 목록 검색

- **위치**: 목록 상단의 현재 카테고리 검색창
- **UI 문구**: `"Search this category…"` · ko `"이 범주에서 검색…"`, `"Clear search"`
- **코드 경로**: `tools/installer/src/gui/list.rs`와 `tools/installer/src/gui/text_input.rs`에서 입력을 받아 `tools/installer/src/app/search.rs`에서 비교한다.
- **테스트**: `tools/installer/src/gui/keyboard_tests.rs`, `tools/installer/src/app/search.rs`, `tools/installer/tests/gui_smoke.py`

GUI 목록 위 검색창은 현재 카테고리에 적용한다. 이름, 원본과 설치 경로, 출처 이름으로 찾는다. MCP는 설명, 분류, 명령, URL과 필요한 환경변수 이름도 검색한다. 플러그인은 마켓플레이스와 설명, 저장소 주소를 포함한다. 입력한 자격증명 값은 검색하지 않는다.

공백으로 나눈 단어는 모두 일치해야 한다. 영문 대소문자를 구분하지 않으며 한글과 악센트 문자의 조합형 및 분해형을 같은 문자로 비교한다. 표시 이름과 실제 파일 경로는 변경하지 않는다. 상태 필터와 폴더 숨김 규칙도 함께 적용한다.

검색에 가려진 항목의 선택은 보존하지만 설치와 제거는 현재 보이는 검색 결과만 처리한다. 전체 선택과 해제도 검색 중에는 결과 안에서만 적용한다. 결과가 없으면 실행 버튼을 비활성화하고 이전 항목의 상세 정보도 표시하지 않는다.

검색창은 클릭하거나 Linux와 Windows에서 `Ctrl+F`, macOS에서 `Cmd+F`로 연다. `Enter`는 목록 키보드 탐색으로 돌아간다. `Esc`와 `Clear`는 검색을 비운다. 검색창은 입력한 문자를 받아 검색에만 사용한다. TUI는 현재 탭에서 `/`로 검색한다.

검색창 안에서는 `Ctrl+A`와 `Cmd+A`로 검색어 전체를 선택한다. 복사, 잘라내기와 붙여넣기도 두 보조키와 `C`, `X`, `V` 조합으로 사용한다. Linux에 연결한 Mac 키보드에서도 Command 입력을 지원한다. 이 단축키는 검색 결과의 체크 상태를 바꾸지 않는다.

폴더 펼침 버튼은 글꼴 문자를 대신해 두 방향의 SVG 선을 사용한다. 버튼과 아이콘의 가로와 세로 중심을 고정하고 두 테마에서 펼침과 접힘 상태를 검수한다.

## Sources 화면의 구현 경로

`tools/installer/src/gui/sources.rs`와 `source_forms.rs`는 `App`을 읽고 화면을 만든다. 진행 화면과 diff는 `activity.rs`, 입력과 확인 대화상자는 `dialogs.rs`에 둔다. 버튼은 `gui::actions::Action` 메시지를 보낸다. 소스 검증과 저장은 기존 `tools/installer/src/app/sources.rs`, `tools/installer/src/app/source_wizard.rs`, `tools/installer/src/source/config.rs`가 처리한다.

| 화면 | GUI 동작 |
|---|---|
| 소스 목록 | 번들, Git, 로컬 소스를 클릭해 선택하고 추가, 수정, 제거, 전체 동기화 실행 |
| 소스 종류 | Git 저장소나 로컬 디렉터리 선택 |
| Git URL | URL 입력과 기존 검증 오류 표시 |
| 브랜치 | 브랜치 입력, 비우면 기본 브랜치 사용 |
| 로컬 경로 | 경로 입력과 기존 검증 오류 표시 |
| 하위 루트 | 선택적 하위 디렉터리 입력 |
| 매핑 | 기존 일곱 컴포넌트 종류나 디렉터리 구조 유지 선택 |
| 삭제 확인 | 선택한 사용자 소스를 표시하고 제거와 취소 제공 |
| 동기화 | 대기 상태와 취소 제공, 완료 뒤 성공이나 오류 표시 |

`sources.rs`의 단위 테스트는 번들 선택과 범위를 벗어난 선택에서 삭제 대상을 만들지 않는지 확인한다. 빈 입력과 여러 줄 오류, 잘못된 수정 인덱스로 각 소스 화면을 생성하는 경로도 검사한다.

`tools/installer/src/app/source_wizard.rs`는 저장 성공 뒤에만 목록과 위저드 상태를 바꾼다. 저장 실패 뒤 다시 시도해도 URL, 브랜치와 하위 루트를 유지한다. `tools/installer/src/app/sources.rs`는 소스 삭제를 설정에 저장한 뒤 Git 캐시를 정리한다. 삭제 저장 실패 시 목록, 커서와 캐시를 유지한다. 삭제 후 소스 해석은 백그라운드에서 실행하고 해석 실패와 캐시 정리 실패를 표시한다.

소스 저장과 제거 테스트는 사용자 홈을 바꾸지 않는다. 임시 설정 경로에서 실제 파일 쓰기 실패를 만들고 상태와 캐시 보존을 확인한다.

## 배포 대상

| 대상 | 바이너리와 패키지 |
|---|---|
| macOS | Universal 바이너리, 앱 DMG와 업데이트용 ZIP |
| Windows | x64 MSVC 바이너리, MSI와 휴대용 ZIP |
| 일반 Linux | GNU 바이너리, deb와 rpm |
| Alpine Linux | 동적 musl 바이너리와 apk |

`tools/installer/build.sh`는 `native`, `macos`, `windows`, `linux`, `linux-musl` 대상을 받는다. Linux GUI 패키지는 X11, Wayland와 xkbcommon 실행 의존성 및 데스크톱 실행기를 포함한다. 루트 설치 스크립트는 Alpine을 감지해 musl 아카이브를 선택한다. 배포 설정만으로 다른 운영체제에서의 실행 검증을 대신하지 않는다.

## 검증 명령

```bash
python3 tools/lint-prose.py
python3 tools/lint-arch.py
cargo fmt --manifest-path tools/installer/Cargo.toml --check
cargo test --locked --manifest-path tools/installer/Cargo.toml
```

화면 검수는 다크 모드와 라이트 모드에서 각각 수행한다. 창을 줄이고 긴 URL과 한글 경로를 입력한다. 설치와 제거에는 임시 사용자 디렉터리를 사용한다. 취소와 실패 뒤에는 다시 실행해 디스크 상태와 목록을 비교한다.

## 화면 설계 기준

탐색은 왼쪽 사이드바에 모은다. 현재 대상과 카테고리를 유지해 사용자가 화면 위치를 확인할 수 있다. 목록은 이름과 출처, 상태를 구분한다. 현재 필터에 포함된 선택 수와 설치·제거 가능 수를 계산하고, 실행할 항목이 없는 버튼은 비활성화한다.

설치는 기본 작업 버튼으로 강조한다. 제거는 보조 작업으로 표시한다. 확인창에서 수량, 이름, 작업 위치를 먼저 검토한다. 취소는 선택을 유지하며 Enter 키만으로 제거를 확정하지 않는다.

색상과 간격, 카드와 버튼 상태는 `tools/installer/src/gui/styles.rs`에서 관리한다. 다크 모드와 라이트 모드는 같은 정보 위계를 사용한다. 본문과 보조 문구, 주요 동작과 오류의 대비를 두 테마에서 자동 검증한다.

### `gui-appearance`: 창 헤더, 테마와 언어

- **위치**: 헤더 오른쪽의 테마 토글, 하단 버전 옆의 언어 선택
- **UI 문구**: `"Light theme"`, `"Dark theme"`, `"Could not save display settings"`
- **코드 경로**: `tools/installer/src/gui/window_bar.rs` → `view`, `tools/installer/src/gui/styles.rs`, `tools/installer/src/gui/runtime.rs` → `start_initialization`, `tools/installer/src/gui/preferences.rs` → `Writer`. `tools/installer/src/fs/installer/preferences.rs`의 `load`와 `save`가 파일을 다룬다.
- **테스트**: `tools/installer/src/gui/window_bar.rs`, `tools/installer/src/gui/keyboard_tests.rs`, `tools/installer/src/gui/locale.rs`, `tools/installer/src/fs/installer/preferences.rs`

헤더는 테마 버튼과 창 드래그 영역을 나눈다. 유리 스타일의 배경과 카드, 입력과 클릭 상태는 `tools/installer/src/gui/styles.rs`에서 관리한다. 현재 설정 언어는 영어, 한국어, 일본어 중 하나다. 저장한 값이 없으면 운영체제의 선호 언어 목록에서 지원하는 첫 언어를 고르고, 목록에 해당 언어가 없으면 영어를 사용한다. 경로와 외부 도구 로그는 원문을 표시한다.

GUI는 언어와 테마를 `~/.hibi/preferences.json`에 저장한다. 저장할 키만 바꿔 다른 설정을 보존한다. 파일이 손상되었거나 읽기 전용이면 원인을 표시하고 파일을 보존한다. 창 닫기는 진행 중인 환경설정 쓰기가 끝날 때까지 기다린다.

디자인 참고 자료는 탐색, 표, 대화상자와 테마 원칙을 설명한다. [GNOME 탐색](https://developer.gnome.org/hig/guidelines/navigation.html) · [GNOME 대화상자](https://developer.gnome.org/hig/patterns/feedback/dialogs.html) · [Material 표](https://m1.material.io/components/data-tables.html) · [Material 다크 테마](https://m2.material.io/design/color/dark-theme.html) · [Mobbin 공개 사례](https://mobbin.com/blog/how-to-use-mobbin-mcp) · [Raycast 설정](https://manual.raycast.com/settings)

Claude Code의 MCP는 user와 local 범위를 제공한다. 기존 Codex MCP 설치 경로는 user 범위만 지원하므로 GUI는 local 선택을 비활성화하고 이유를 표시한다.

자동 GUI 검수는 `tools/installer/tests/gui_smoke.py`에 있다. 실제 X11 입력과 화면 문구, 설치 파일과 매니페스트를 확인한다. 테스트마다 임시 홈과 CLI 대역을 사용한다. Python Xlib, Pillow, Tesseract 영어 데이터와 Xvfb가 필요하다. CLI와 터미널 복원 검수는 표준 Python 라이브러리만 사용하는 `tools/installer/tests/cli_smoke.py`에 있다.

```bash
cargo build --locked --manifest-path tools/installer/Cargo.toml
python3 tools/installer/tests/cli_smoke.py --binary tools/installer/target/debug/hibi
xvfb-run -a python3 tools/installer/tests/gui_smoke.py --binary tools/installer/target/debug/hibi
```

릴리즈 검증 작업은 두 스모크 검사를 실행하고 GUI 화면과 로그를 아티팩트로 남긴다.

## 검수 결과를 확인하는 곳

CI의 `gui-acceptance`에서 실제 X11 입력 스모크의 화면과 로그를 확인한다. 로컬에서 검수할 때는 `--artifacts`로 출력 위치를 지정한다. 확인한 커밋, 플랫폼, 테마, 창 크기를 결과와 함께 기록한다. macOS 활성 창, Wayland, Windows의 창 조작은 해당 환경에서 따로 확인한다. [운영 가이드](../RUNBOOK.md)
