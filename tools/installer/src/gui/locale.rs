use super::Language;
mod notices;

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::English => "English",
            Self::Korean => "한국어",
            Self::Japanese => "日本語",
        })
    }
}

macro_rules! catalog {
    ($($english:literal => $korean:literal, $japanese:literal;)*) => {
        impl Language {
            pub(super) fn text(self, english: &str) -> &str {
                if self == Self::English { return english; }
                let (ko, ja) = match english {
                    $($english => ($korean, $japanese),)*
                    _ => return english,
                };
                if self == Self::Korean { ko } else { ja }
            }
        }
    };
}

macro_rules! localized_format {
    ($lang:expr, $en:literal, $ko:literal, $ja:literal $(, $arg:expr)* $(,)?) => {
        match $lang {
            $crate::gui::Language::English => format!($en $(, $arg)*),
            $crate::gui::Language::Korean => format!($ko $(, $arg)*),
            $crate::gui::Language::Japanese => format!($ja $(, $arg)*),
        }
    };
}

impl Language {
    pub(super) fn preferred(locales: impl Iterator<Item = String>) -> Self {
        locales
            .filter_map(|tag| {
                match tag
                    .split(['-', '_', '.', '@'])
                    .next()?
                    .to_ascii_lowercase()
                    .as_str()
                {
                    "en" => Some(Self::English),
                    "ko" => Some(Self::Korean),
                    "ja" => Some(Self::Japanese),
                    _ => None,
                }
            })
            .next()
            .unwrap_or_default()
    }

    pub(super) fn result_count(self, count: usize, searching: bool) -> String {
        match self {
            Self::English => {
                let noun = if searching { "result" } else { "item" };
                format!("{count} {noun}{}", if count == 1 { "" } else { "s" })
            }
            Self::Korean if searching => format!("검색 결과 {count}개"),
            Self::Korean => format!("항목 {count}개"),
            Self::Japanese if searching => format!("検索結果 {count} 件"),
            Self::Japanese => format!("項目 {count} 件"),
        }
    }
}

catalog! {
    "Checking for updates…" => "업데이트 확인 중…", "更新を確認中…";
    "Up to date" => "최신 버전입니다", "最新です";
    "Could not check for updates" => "업데이트를 확인할 수 없습니다", "更新を確認できません";
    "Could not open release page" => "릴리스 페이지를 열 수 없습니다", "リリースページを開けません";
    "Update failed" => "업데이트 실패", "更新に失敗しました";
    "Waiting for the package update" => "패키지 배포 대기 중", "パッケージの公開待ちです";
    "Download the package for your installation." => "설치 방식에 맞는 패키지를 다운로드하세요.", "インストール方法に合ったパッケージをダウンロードしてください。";
    "Open release page" => "릴리스 페이지 열기", "リリースページを開く";
    "hibi will restart after updating." => "업데이트 후 hibi를 다시 시작합니다.", "更新後に hibi を再起動します。";
    "Update" => "업데이트", "更新";
    "Wait for the current operation to finish." => "진행 중인 작업이 끝나면 업데이트할 수 있습니다.", "現在の操作が完了すると更新できます。";
    "Updating hibi…" => "hibi 업데이트 중…", "hibi を更新中…";
    "Installation finished" => "설치 완료", "インストール完了";
    "Installation cancelled" => "설치 취소", "インストールをキャンセル";
    "Removal finished" => "제거 완료", "削除完了";
    "Removal cancelled" => "제거 취소", "削除をキャンセル";
    "MCP scope: user" => "MCP 범위: 사용자", "MCP の範囲: ユーザー";
    "Selected Claude Code" => "Claude Code 선택됨", "Claude Code を選択しました";
    "Selected Codex CLI" => "Codex CLI 선택됨", "Codex CLI を選択しました";
    "Selected MCP servers are already registered" => "선택한 MCP 서버는 이미 등록되어 있습니다", "選択した MCP サーバーは登録済みです";
    "Resolved sources" => "소스를 확인했습니다", "ソースを確認しました";
    "Operation finished; status refresh failed. Review the log." => "작업은 끝났지만 상태 갱신에 실패했습니다. 로그를 확인하세요.", "処理は終了しましたが、状態を更新できませんでした。ログを確認してください。";
    "Git URL must not contain credentials" => "Git URL에 인증 정보를 포함할 수 없습니다", "Git URL に認証情報を含めることはできません";
    "The bundled source cannot be removed" => "기본 제공 소스는 제거할 수 없습니다", "同梱のソースは削除できません";
    "Root must be a directory inside the local source" => "루트는 로컬 소스 안의 디렉터리여야 합니다", "ルートにはローカルソース内のディレクトリを指定してください";
    "The source being edited is no longer available" => "편집 중인 소스를 더 이상 사용할 수 없습니다", "編集中のソースは利用できなくなりました";
    "Folder contents changed. Review the folders before hiding." => "폴더 내용이 변경되었습니다. 숨기기 전에 다시 확인하세요.", "フォルダーの内容が変わりました。非表示にする前に確認してください。";
    "Showing hidden folders. Press x on a hidden folder to unhide it." => "숨긴 폴더를 표시합니다. 폴더에서 x를 누르면 숨김이 해제됩니다.", "非表示フォルダーを表示しています。対象のフォルダーで x を押すと再表示できます。";
    "Showing only hidden folders. Unhide selected restores their visibility." => "숨긴 폴더만 표시합니다. 선택한 폴더 표시로 숨김을 해제하세요.", "非表示フォルダーのみ表示しています。選択したフォルダーを再表示できます。";
    "Cancelling; waiting for the source worker to stop..." => "취소 중입니다. 소스 작업이 중지될 때까지 기다려 주세요.", "キャンセル中です。ソース処理が停止するまでお待ちください。";
    "Sync thread crashed" => "동기화가 중단되었습니다", "同期が中断されました";
    "Preflight thread crashed" => "사전 점검이 중단되었습니다", "事前チェックが中断されました";
    "No environment variable is awaiting a value" => "입력할 환경 변수가 없습니다", "入力待ちの環境変数はありません";
    "Minimize" => "최소화", "最小化";
    "Maximize or restore" => "최대화 또는 복원", "最大化または元に戻す";
    "Your AI workspace" => "내 AI 작업 공간", "AI ワークスペース";
    "Your workspace, configured." => "작업 공간 설정", "ワークスペースの設定";
    "Manage the instructions, tools and workflows your AI assistants use." => "AI 도우미가 사용할 지침, 도구, 워크플로를 관리하세요.", "AI アシスタントが使う指示、ツール、ワークフローを管理します。";
    "Agents, skills, hooks, MCP servers and plugins" => "에이전트, 스킬, 훅, MCP 서버, 플러그인", "エージェント、スキル、フック、MCP サーバー、プラグイン";
    "Skills, shared configuration and MCP servers" => "스킬, 공유 설정, MCP 서버", "スキル、共有設定、MCP サーバー";
    "Connected sources" => "연결된 소스", "接続済みのソース";
    "Manage sources" => "소스 관리", "ソースを管理";
    "Workspace" => "작업 공간", "ワークスペース";
    "Components" => "구성 요소", "コンポーネント";
    "Component" => "구성 요소", "コンポーネント";
    "Keyboard shortcuts" => "키보드 단축키", "キーボードショートカット";
    "Switch CLI" => "CLI 전환", "CLI を切り替え";
    "Agents" => "에이전트", "エージェント";
    "Commands" => "명령", "コマンド";
    "Contexts" => "컨텍스트", "コンテキスト";
    "Rules" => "규칙", "ルール";
    "Skills" => "스킬", "スキル";
    "Hooks" => "훅", "フック";
    "Output styles" => "출력 스타일", "出力スタイル";
    "Styles" => "스타일", "スタイル";
    "Statusline" => "상태 표시줄", "ステータスライン";
    "Config" => "설정", "設定";
    "Plugins" => "플러그인", "プラグイン";
    "Expand all" => "모두 펼치기", "すべて展開";
    "Collapse all" => "모두 접기", "すべて折りたたむ";
    "Change scope" => "범위 변경", "適用範囲を変更";
    "Not set" => "미설정", "未設定";
    "Set default" => "기본값 설정", "既定に設定";
    "Clear default" => "기본값 해제", "既定を解除";
    "Status" => "상태", "状態";
    "Clear selection" => "선택 해제", "選択を解除";
    "Select all" => "전체 선택", "すべて選択";
    "Diff" => "차이 보기", "差分";
    "Remove" => "제거", "削除";
    "Install" => "설치", "インストール";
    "Search this category…" => "이 범주에서 검색…", "このカテゴリを検索…";
    "Clear" => "지우기", "クリア";
    "In this view" => "현재 보기", "現在の表示範囲";
    "Specialist instructions for focused work." => "특정 작업을 위한 전문 지침입니다.", "特定の作業に特化した指示です。";
    "Reusable workflows for your everyday tasks." => "반복 작업에 재사용할 워크플로입니다.", "日々の作業で再利用できるワークフローです。";
    "Context that helps your assistant understand your work." => "도우미가 작업을 이해하는 데 필요한 배경 정보입니다.", "アシスタントが作業を理解するための背景情報です。";
    "Consistent conventions across your projects." => "여러 프로젝트에서 공통으로 사용할 규칙입니다.", "プロジェクト間で共有するルールです。";
    "Tools and knowledge for the way you build." => "개발에 필요한 도구와 지식입니다.", "開発に使うツールと知識です。";
    "Automation that runs at specific events." => "지정한 이벤트에서 자동으로 실행됩니다.", "指定したイベントで自動実行されます。";
    "Choose how your assistant communicates." => "도우미의 응답 방식을 선택하세요.", "アシスタントの応答スタイルを選びます。";
    "Useful context below your Claude Code prompt." => "Claude Code 프롬프트 아래에 표시할 정보입니다.", "Claude Code のプロンプトの下に表示する情報です。";
    "Shared settings with your existing preferences preserved." => "기존 설정을 유지하면서 공유 설정을 적용합니다.", "既存の設定を保持しながら共有設定を適用します。";
    "Connect your assistant to external tools and services." => "도우미를 외부 도구와 서비스에 연결합니다.", "アシスタントを外部のツールやサービスに接続します。";
    "Extend Claude Code with marketplace plugins." => "마켓플레이스 플러그인으로 Claude Code의 기능을 확장합니다.", "マーケットプレイスのプラグインで Claude Code の機能を拡張します。";
    "Filters · Default" => "필터 · 기본값", "フィルター · 既定";
    "Default" => "기본값", "既定";
    "Modified" => "수정됨", "変更あり";
    "External" => "외부", "外部";
    "New" => "신규", "新規";
    "Visible only" => "표시 항목만", "表示中のみ";
    "Include hidden" => "숨김 항목 포함", "非表示も含む";
    "Hidden only" => "숨김 항목만", "非表示のみ";
    "hidden" => "숨김", "非表示";
    "partial selection" => "일부 선택", "一部選択";
    "folder" => "폴더", "フォルダー";
    " · deprecated" => " · 지원 종료 예정", " · 非推奨";
    "No matching items" => "검색 결과가 없습니다", "一致する項目がありません";
    "No hidden items in this view" => "숨긴 항목이 없습니다", "非表示の項目はありません";
    "Nothing to show here yet" => "표시할 항목이 없습니다", "表示する項目はありません";
    "Try another search, change the status filter, or choose another visibility mode." => "검색어나 상태 필터, 숨김 표시 방식을 바꿔 보세요.", "検索語、状態フィルター、表示範囲を変更してください。";
    "Clear search" => "검색 지우기", "検索をクリア";
    "new" => "신규", "新規";
    "not installed" => "미설치", "未インストール";
    "modified" => "수정됨", "変更あり";
    "installed" => "설치됨", "インストール済み";
    "managed" => "관리 중", "管理対象";
    "blocked" => "변경 불가", "変更不可";
    "external" => "외부", "外部";
    "unknown" => "확인 불가", "不明";
    "user" => "사용자", "ユーザー";
    "local" => "로컬", "ローカル";
    "Finding your components" => "구성 요소 찾는 중", "コンポーネントを検索中";
    "Scanning connected libraries and checking installed configuration." => "연결된 라이브러리와 설치된 설정을 확인하고 있습니다.", "接続済みのライブラリとインストール済みの設定を確認しています。";
    "This may take a moment." => "잠시 기다려 주세요.", "しばらくお待ちください。";
    "MCP server" => "MCP 서버", "MCP サーバー";
    "Value" => "값", "値";
    "Enter the required value to continue. It is hidden while you type." => "계속하려면 필수 값을 입력하세요. 입력한 값은 화면에 표시되지 않습니다.", "続行するには必須の値を入力してください。入力内容は画面に表示されません。";
    "Enter an absolute file path in a writable directory for persistent storage." => "영구 저장에 사용할 쓰기 가능한 디렉터리의 절대 파일 경로를 입력하세요.", "永続化に使うファイルの絶対パスを、書き込み可能なディレクトリ内で指定してください。";
    "Set the token in the environment of your MCP client." => "MCP 클라이언트의 환경 변수에 토큰을 설정하세요.", "MCP クライアントの環境変数にトークンを設定してください。";
    "Enter value" => "값 입력", "値を入力";
    "Choose a project directory" => "프로젝트 디렉터리 선택", "プロジェクトディレクトリを選択";
    "The MCP server will be available in this project's local configuration." => "이 프로젝트의 로컬 설정에서 MCP 서버를 사용할 수 있습니다.", "このプロジェクトのローカル設定で MCP サーバーを使用できます。";
    "Project directory" => "프로젝트 디렉터리", "プロジェクトディレクトリ";
    "Leave this environment?" => "이 환경에서 나갈까요?", "この環境を離れますか？";
    "Your pending selections in every category will be cleared. Installed components stay in place." => "모든 범주의 선택이 해제됩니다. 설치된 구성 요소는 유지됩니다.", "すべてのカテゴリの選択が解除されます。インストール済みの項目は保持されます。";
    "Discard selections" => "선택 취소 후 나가기", "選択を破棄して戻る";
    "These folders will be hidden from the component list and excluded from install and remove actions. Their files stay in place." => "이 폴더를 목록에서 숨기고 설치 및 제거 대상에서 제외합니다. 파일은 그대로 유지됩니다.", "これらのフォルダーを一覧から隠し、インストールと削除の対象から除外します。ファイルは保持されます。";
    "Hide folders?" => "폴더를 숨길까요?", "フォルダーを非表示にしますか？";
    "Hide folders" => "폴더 숨기기", "フォルダーを非表示";
    "Select an environment to get started." => "시작할 환경을 선택하세요.", "開始する環境を選択してください。";
    "Cancel" => "취소", "キャンセル";
    "Continue" => "계속", "続ける";
    "Keep working" => "계속 작업", "作業を続ける";
    "Stop the current operation?" => "현재 작업을 중지할까요?", "現在の処理を停止しますか？";
    "Ready to close?" => "앱을 종료할까요?", "アプリを終了しますか？";
    "The window closes when the active operation stops." => "진행 중인 작업이 중지되면 창을 닫습니다.", "実行中の処理が停止するとウィンドウが閉じます。";
    "Pending selections will be discarded." => "아직 적용하지 않은 선택은 취소됩니다.", "未適用の選択は破棄されます。";
    "Close application" => "앱 종료", "アプリを終了";
    "This removes the selected installed items in the current category from this environment, including selected external files." => "선택한 외부 파일을 포함하여 현재 범주에서 선택한 설치 항목을 이 환경에서 제거합니다.", "選択した外部ファイルを含め、現在のカテゴリで選択したインストール済み項目をこの環境から削除します。";
    "This action cannot be undone." => "이 작업은 되돌릴 수 없습니다.", "この操作は取り消せません。";
    "Remove items" => "항목 제거", "項目を削除";
    "Search" => "검색", "検索";
    "Focus category search" => "범주 검색창으로 이동", "カテゴリ検索に移動";
    "Select all text in the focused search field" => "검색창의 텍스트 전체 선택", "検索欄の文字をすべて選択";
    "Return from search to the component list" => "검색창에서 구성 요소 목록으로 이동", "検索欄からコンポーネント一覧に戻る";
    "Clear the current search before going back" => "현재 검색을 지운 뒤 뒤로 이동", "戻る前に検索をクリア";
    "Selection" => "선택", "選択";
    "Select or clear the focused item" => "현재 항목 선택 또는 해제", "現在の項目の選択を切り替え";
    "Select or clear shown items" => "표시된 항목 전체 선택 또는 해제", "表示中の全項目の選択を切り替え";
    "Browse & review" => "탐색 및 검토", "閲覧と確認";
    "Move through the focused categories or list" => "활성 범주 또는 목록 안에서 이동", "選択中のカテゴリまたは一覧内を移動";
    "Collapse or expand folders in the list" => "목록의 폴더 접기 또는 펼치기", "一覧のフォルダーを折りたたむ、または展開";
    "Enter the list from categories" => "범주에서 목록으로 이동", "カテゴリから一覧に移動";
    "Focus categories or component list" => "범주 또는 구성 요소 목록으로 이동", "カテゴリまたはコンポーネント一覧に移動";
    "Switch between categories and component list" => "범주와 구성 요소 목록 간 전환", "カテゴリとコンポーネント一覧を切り替え";
    "View changes or open a folder" => "변경 내용 확인 또는 폴더 열기", "差分を表示、またはフォルダーを開く";
    "Cycle the status filter" => "다음 상태 필터로 전환", "状態フィルターを順に切り替え";
    "Expand or collapse all folders" => "모든 폴더 펼치기 또는 접기", "すべてのフォルダーを展開または折りたたむ";
    "Manage components" => "구성 요소 관리", "コンポーネントの管理";
    "Install selected items" => "선택 항목 설치", "選択した項目をインストール";
    "Remove selected items" => "선택 항목 제거", "選択した項目を削除";
    "Set or unset a default style or statusline" => "기본 스타일 또는 상태 표시줄 설정 및 해제", "既定のスタイルまたはステータスラインを設定・解除";
    "Change MCP scope" => "MCP 적용 범위 변경", "MCP の適用範囲を変更";
    "Folder visibility" => "폴더 표시", "フォルダーの表示";
    "Hide or unhide a folder" => "폴더 숨기기 또는 표시", "フォルダーの表示を切り替え";
    "Hide selected folders" => "선택한 폴더 숨기기", "選択したフォルダーを非表示";
    "Show or conceal hidden folders" => "숨긴 폴더 표시 전환", "非表示フォルダーの表示を切り替え";
    "Application" => "앱", "アプリ";
    "Switch light and dark themes" => "라이트·다크 테마 전환", "ライト・ダークテーマを切り替え";
    "Open this reference" => "단축키 도움말 열기", "ショートカット一覧を開く";
    "Go back or cancel" => "뒤로 이동 또는 취소", "戻る、またはキャンセル";
    "Close the application" => "앱 종료", "アプリを終了";
    "Back" => "뒤로", "戻る";
    "Click to focus a row. Use its checkbox to select it. Input fields support selection and paste; shortcuts pause while typing." => "행을 클릭하면 포커스가 이동합니다. 체크박스로 항목을 선택하세요. 입력창에서는 선택과 붙여넣기를 사용할 수 있으며 입력 중에는 단축키가 일시 중지됩니다.", "行をクリックしてフォーカスし、チェックボックスで選択します。入力欄では選択と貼り付けができ、入力中はショートカットが停止します。";
    "Sources" => "소스", "ソース";
    "Your component libraries, all in one place." => "구성 요소를 가져올 라이브러리를 관리하세요.", "コンポーネントの取得元となるライブラリを管理します。";
    "Bundled library" => "기본 제공 라이브러리", "同梱ライブラリ";
    "Included with HIBI AI · Read only" => "HIBI AI에 포함됨 · 읽기 전용", "HIBI AI に同梱 · 読み取り専用";
    "Add source" => "소스 추가", "ソースを追加";
    "Sync all" => "모두 동기화", "すべて同期";
    "Connected libraries" => "연결된 라이브러리", "接続済みライブラリ";
    "Selected sources provide your components" => "선택한 소스에서 구성 요소를 가져옵니다", "選択したソースからコンポーネントを取得します";
    "Edit source" => "소스 편집", "ソースを編集";
    "Remove source" => "소스 제거", "ソースを削除";
    "Cached version · Sync to retry" => "캐시된 버전 · 동기화하여 다시 시도", "キャッシュを使用中 · 同期して再試行";
    "Selected" => "선택됨", "選択中";
    "Local" => "로컬", "ローカル";
    "Where are your components?" => "구성 요소를 어디서 가져올까요?", "どこからコンポーネントを取得しますか？";
    "Step 1" => "1단계", "ステップ 1";
    "Connect a repository or a folder on this computer." => "저장소 또는 컴퓨터의 폴더를 연결하세요.", "リポジトリまたはこのコンピューターのフォルダーを接続します。";
    "Git repository" => "Git 저장소", "Git リポジトリ";
    "Sync components from an HTTPS repository" => "HTTPS 저장소에서 구성 요소 동기화", "HTTPS リポジトリからコンポーネントを同期";
    "Local folder" => "로컬 폴더", "ローカルフォルダー";
    "Use an existing component directory" => "기존 구성 요소 디렉터리 사용", "既存のコンポーネントディレクトリを使用";
    "Syncing sources" => "소스 동기화 중", "ソースを同期中";
    "Resolving your libraries and refreshing the component inventory." => "라이브러리를 확인하고 구성 요소 목록을 갱신합니다.", "ライブラリを確認し、コンポーネント一覧を更新しています。";
    "Changes will appear when synchronization finishes." => "동기화가 끝나면 변경 사항이 표시됩니다.", "同期が完了すると変更が表示されます。";
    "Disconnect this library?" => "이 라이브러리의 연결을 해제할까요?", "このライブラリの接続を解除しますか？";
    "This source is no longer available." => "이 소스는 더 이상 사용할 수 없습니다.", "このソースは利用できなくなりました。";
    "This removes the library from your sources. Installed components can be managed separately." => "이 라이브러리를 소스 목록에서 제거합니다. 설치한 구성 요소는 따로 관리할 수 있습니다.", "ソース一覧からこのライブラリを削除します。インストール済みの項目は個別に管理できます。";
    "Connect a Git repository" => "Git 저장소 연결", "Git リポジトリを接続";
    "Use an HTTPS URL without credentials." => "인증 정보가 없는 HTTPS URL을 사용하세요.", "認証情報を含まない HTTPS URL を使用してください。";
    "Choose a branch" => "브랜치 선택", "ブランチを選択";
    "Default branch" => "기본 브랜치", "既定のブランチ";
    "Leave empty to follow the repository's default branch." => "비워 두면 저장소의 기본 브랜치를 사용합니다.", "空欄の場合はリポジトリの既定ブランチを使用します。";
    "Choose a local folder" => "로컬 폴더 선택", "ローカルフォルダーを選択";
    "Enter a component folder. You can use ~ for your home directory." => "구성 요소 폴더를 입력하세요. 홈 디렉터리는 ~로 표시할 수 있습니다.", "コンポーネントのフォルダーを入力してください。ホームディレクトリには ~ を使用できます。";
    "Set the source root" => "소스 루트 설정", "ソースのルートを設定";
    "Repository root" => "저장소 루트", "リポジトリのルート";
    "Optional: a subdirectory such as config/claude." => "선택 사항입니다. config/claude 같은 하위 디렉터리를 입력하세요.", "必要に応じて config/claude などのサブディレクトリを入力してください。";
    "Source" => "소스", "ソース";
    "Organize your components" => "구성 요소 분류", "コンポーネントの分類";
    "Map every file to one component type, or keep the source's folder structure." => "모든 파일에 같은 유형을 지정하거나 소스의 폴더 구조를 유지하세요.", "全ファイルに同じ種類を指定するか、ソースのフォルダー構成を保持します。";
    "Keep folder structure" => "폴더 구조 유지", "フォルダー構成を保持";
    "No differences available." => "표시할 차이가 없습니다.", "表示できる差分はありません。";
    "Compare configuration" => "설정 비교", "設定を比較";
    "Back to components" => "구성 요소로 돌아가기", "コンポーネント一覧に戻る";
    "Source content compared with the installed version." => "소스와 설치된 버전의 내용을 비교합니다.", "ソースの内容とインストール済みの内容を比較します。";
    "Checking requirements" => "요구 사항 확인 중", "要件を確認中";
    "Cancelling operation" => "작업 취소 중", "処理をキャンセル中";
    "Refreshing installed state" => "설치 상태 갱신 중", "インストール状態を更新中";
    "Operation cancelled" => "작업이 취소되었습니다", "処理をキャンセルしました";
    "Completed with errors" => "오류가 발생한 상태로 완료됨", "エラーが発生して完了しました";
    "Operation complete" => "작업 완료", "処理が完了しました";
    "Removing components" => "구성 요소 제거 중", "コンポーネントを削除中";
    "Installing components" => "구성 요소 설치 중", "コンポーネントをインストール中";
    "Checking that your environment is ready. No components have been changed yet." => "환경을 확인하고 있습니다. 아직 변경된 구성 요소는 없습니다.", "環境を確認しています。コンポーネントはまだ変更されていません。";
    "Cancel operation" => "작업 취소", "処理をキャンセル";
    "Operation log" => "작업 로그", "処理ログ";
    "Finished" => "완료", "完了";
    "Live" => "진행 중", "実行中";
    "Review the log before leaving this screen." => "이 화면을 나가기 전에 로그를 확인하세요.", "この画面を離れる前にログを確認してください。";
    "Cancelling waits for the current operation to stop safely." => "취소하면 현재 작업이 안전하게 중지될 때까지 기다립니다.", "キャンセルすると、現在の処理が安全に停止するまで待機します。";
    "Loading sources and configuration…" => "소스와 설정을 불러오는 중…", "ソースと設定を読み込み中…";
    "Dismiss" => "닫기", "閉じる";
    "Retry" => "다시 시도", "再試行";
    "Light theme" => "라이트 테마", "ライトテーマ";
    "Dark theme" => "다크 테마", "ダークテーマ";
    "Cancelled" => "취소됨", "キャンセルしました";
    "No items selected" => "선택한 항목이 없습니다", "項目が選択されていません";
    "Enter an existing project directory." => "기존 프로젝트 디렉터리를 입력하세요.", "既存のプロジェクトディレクトリを入力してください。";
    "Only HTTPS git URLs are allowed" => "HTTPS Git URL만 사용할 수 있습니다", "HTTPS の Git URL のみ使用できます";
    "Local source path is empty" => "로컬 소스 경로가 비어 있습니다", "ローカルソースのパスが空です";
    "Path traversal (..) not allowed in root" => "루트에는 상위 디렉터리 경로(..)를 사용할 수 없습니다", "ルートには親ディレクトリを示す .. を使用できません";
    "Select items in this category to remove." => "이 범주에서 제거할 항목을 선택하세요.", "このカテゴリで削除する項目を選択してください。";
    "Selection changed. Review your selection before removing items." => "선택이 변경되었습니다. 제거하기 전에 다시 확인하세요.", "選択内容が変わりました。削除する前に確認してください。";
    "Initialization worker stopped unexpectedly. Retry to reload." => "초기화가 중단되었습니다. 다시 시도하세요.", "初期化が中断されました。再試行してください。";
    "Codex MCP uses user scope. Local scope is available for Claude Code." => "Codex MCP는 사용자 범위를 사용합니다. 로컬 범위는 Claude Code에서 사용할 수 있습니다.", "Codex MCP はユーザー範囲を使用します。ローカル範囲は Claude Code で利用できます。";
    "Codex MCP uses user scope." => "Codex MCP는 사용자 범위를 사용합니다.", "Codex MCP はユーザー範囲を使用します。";
    "Loading failed" => "불러오기 실패", "読み込みに失敗しました";
    "Select a folder to hide or unhide" => "숨기거나 표시할 폴더를 선택하세요", "表示を切り替えるフォルダーを選択してください";
    "No fully checked folders to hide" => "모두 선택된 폴더가 없습니다", "すべての項目が選択されたフォルダーはありません";
    "Hidden folders concealed; hidden selections cleared." => "숨긴 폴더의 표시와 선택을 해제했습니다.", "非表示フォルダーを隠し、選択を解除しました。";
    "Unset default output style" => "기본 출력 스타일을 해제했습니다", "既定の出力スタイルを解除しました";
    "Unset statusline" => "상태 표시줄 설정을 해제했습니다", "ステータスラインの設定を解除しました";
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unsupported_locales_are_skipped_in_preference_order() {
        let preferred = |tags: &[&str]| Language::preferred(tags.iter().map(|tag| tag.to_string()));
        assert_eq!(preferred(&["fr-FR", "ja-JP", "ko-KR"]), Language::Japanese);
        assert_eq!(preferred(&["ko_KR.UTF-8", "en-US"]), Language::Korean);
        assert_eq!(preferred(&["en-US", "ko"]), Language::English);
        assert_eq!(preferred(&["fr", "C"]), Language::English);
    }
}
