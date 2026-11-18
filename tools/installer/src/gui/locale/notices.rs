use super::Language;
use std::borrow::Cow;

impl Language {
    /// The shared installer reports status in English. Translate its message frames
    /// at the GUI boundary while keeping paths, CLI output and error details intact.
    pub(in crate::gui) fn notice(self, message: &str) -> Cow<'_, str> {
        if self == Self::English {
            return message.into();
        }
        let translated = self.text(message);
        if translated != message {
            return translated.into();
        }

        if let Some((action, counts)) = message.split_once(": ")
            && let Some((attempted, rest)) = counts.split_once(" of ")
            && let Some((total, errors)) = rest.split_once(" items attempted; errors: ")
            && [attempted, total, errors]
                .iter()
                .all(|s| s.parse::<usize>().is_ok())
            && matches!(
                action,
                "Installation finished"
                    | "Installation cancelled"
                    | "Removal finished"
                    | "Removal cancelled"
            )
        {
            let action = self.text(action);
            return match self {
                Self::Korean => {
                    format!("{action}: 전체 {total}개 중 {attempted}개 처리 · 오류 {errors}개")
                }
                _ => format!("{action}: 全{total}件中{attempted}件を処理 · エラー{errors}件"),
            }
            .into();
        }
        for (english, korean, japanese) in NOTICES {
            let Some((prefix, suffix)) = english.split_once("{}") else {
                continue;
            };
            if let Some(value) = message
                .strip_prefix(prefix)
                .and_then(|rest| rest.strip_suffix(suffix))
            {
                let template = if self == Self::Korean {
                    korean
                } else {
                    japanese
                };
                return template.replacen("{}", value, 1).into();
            }
        }
        message.into()
    }
}

const NOTICES: &[(&str, &str, &str)] = &[
    (
        "Invalid branch name: {}",
        "잘못된 브랜치 이름: {}",
        "無効なブランチ名: {}",
    ),
    (
        "Branch name must not start with '-': {}",
        "브랜치 이름은 '-'로 시작할 수 없습니다: {}",
        "ブランチ名は '-' で開始できません: {}",
    ),
    (
        "Local source directory does not exist or cannot be accessed: {}",
        "로컬 소스 디렉터리가 없거나 접근할 수 없습니다: {}",
        "ローカルソースのディレクトリが存在しないか、アクセスできません: {}",
    ),
    (
        "Local source path is not a directory: {}",
        "로컬 소스 경로가 디렉터리가 아닙니다: {}",
        "ローカルソースのパスがディレクトリではありません: {}",
    ),
    (
        "Path traversal (..) not allowed in source path: {}",
        "소스 경로에는 ..를 사용할 수 없습니다: {}",
        "ソースのパスに .. は使用できません: {}",
    ),
    (
        "Source path resolves to inside ~/.claude/: {}",
        "~/.claude/ 내부 경로는 소스로 사용할 수 없습니다: {}",
        "~/.claude/ 内のパスはソースに指定できません: {}",
    ),
    (
        "Could not save source: {}",
        "소스를 저장할 수 없습니다: {}",
        "ソースを保存できません: {}",
    ),
    (
        "Failed to resolve sources: {}",
        "소스를 확인할 수 없습니다: {}",
        "ソースを解決できません: {}",
    ),
    (
        "Source removed, but cache cleanup failed: {}",
        "소스를 제거했지만 캐시 정리에 실패했습니다: {}",
        "ソースは削除されましたが、キャッシュを消去できませんでした: {}",
    ),
    (
        "Could not load display settings: {}",
        "화면 설정을 불러올 수 없습니다: {}",
        "表示設定を読み込めません: {}",
    ),
    (
        "Could not save display settings: {}",
        "화면 설정을 저장할 수 없습니다: {}",
        "表示設定を保存できません: {}",
    ),
    (
        "Cannot open project directory: {}. Choose an existing directory.",
        "프로젝트 디렉터리를 열 수 없습니다: {}. 기존 디렉터리를 선택하세요.",
        "プロジェクトディレクトリを開けません: {}。既存のディレクトリを選択してください。",
    ),
    (
        "Hidden by {}. Unhide that parent folder first.",
        "상위 폴더 {}의 숨김을 먼저 해제하세요.",
        "先に親フォルダー {} を再表示してください。",
    ),
    (
        "Hidden by {}. Select that parent folder to unhide it.",
        "상위 폴더 {}를 선택해 숨김을 해제하세요.",
        "親フォルダー {} を選択して再表示してください。",
    ),
    (
        "Hidden {} folder(s); selections cleared. H shows hidden folders.",
        "폴더 {}개를 숨기고 선택을 해제했습니다. H로 숨긴 폴더를 표시합니다.",
        "{} 件のフォルダーを隠して選択を解除しました。H で非表示フォルダーを表示します。",
    ),
    (
        "Unhidden {}. Existing selections are unchanged.",
        "{}의 숨김을 해제했습니다. 기존 선택은 유지됩니다.",
        "{} を再表示しました。既存の選択は保持されます。",
    ),
    (
        "{} selected folder(s) are visible. Existing selections are unchanged.",
        "선택한 폴더 {}개를 표시합니다. 기존 선택은 유지됩니다.",
        "選択した {} 件のフォルダーを表示します。既存の選択は保持されます。",
    ),
    (
        "Failed to load folder visibility: {}",
        "폴더 표시 설정을 불러올 수 없습니다: {}",
        "フォルダーの表示設定を読み込めません: {}",
    ),
    (
        "Failed to save folder visibility: {}",
        "폴더 표시 설정을 저장할 수 없습니다: {}",
        "フォルダーの表示設定を保存できません: {}",
    ),
    (
        "Set default output style: {}",
        "기본 출력 스타일 설정: {}",
        "既定の出力スタイル: {}",
    ),
    (
        "Set statusline: {}",
        "상태 표시줄 설정: {}",
        "ステータスライン: {}",
    ),
    (
        "MCP scope: local ({})",
        "MCP 범위: 로컬 ({})",
        "MCP の範囲: ローカル ({})",
    ),
    ("Theme: {}", "테마: {}", "テーマ: {}"),
    (
        "Error loading: {}",
        "불러오기 오류: {}",
        "読み込みエラー: {}",
    ),
];

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn status_translation_keeps_values_and_unknown_diagnostics_verbatim() {
        assert_eq!(
            Language::Korean.notice("Invalid branch name: 状態/{}"),
            "잘못된 브랜치 이름: 状態/{}"
        );
        assert_eq!(
            Language::Japanese.notice("Installation finished: 2 of 3 items attempted; errors: 1"),
            "インストール完了: 全3件中2件を処理 · エラー1件"
        );
        assert_eq!(
            Language::Japanese.notice("fatal: unknown revision Install"),
            "fatal: unknown revision Install"
        );
    }
}
