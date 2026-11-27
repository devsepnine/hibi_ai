<p><img src="tools/installer/assets/hibi.png" alt="Hibi のアイコン" width="88"></p>

# hibi-ai

Hibi は、Claude Code と Codex CLI の設定を管理するデスクトップおよびターミナルアプリです。同梱の設定集、Git リポジトリ、ローカルフォルダーから、必要なスキルやコンポーネントを選んでインストールできます。

[English](README.md) · [한국어](README.ko.md) · **日本語**

[ダウンロード](https://github.com/devsepnine/hibi_ai/releases/latest) · [リリースノート](https://github.com/devsepnine/hibi_ai/releases) · [Issue を作成](https://github.com/devsepnine/hibi_ai/issues)

## 主な機能

- デスクトップアプリはシステムの外観設定に従います。iced のパレットを選ぶと、次回の起動時にもその設定が使われます。
- SSH ではキーボードでターミナル画面を操作できます。
- コンポーネントを検索し、状態で絞り込めます。インストール済みのファイルをソースと比較し、削除前に対象を確認できます。
- 両方のツールのスキル、設定、MCP サーバーを管理できます。Claude Code では、エージェント、スラッシュコマンド、プラグイン、フック、出力スタイルも管理できます。
- Git リポジトリやローカルフォルダーをソースとして追加できます。ソースとインストール履歴は、デスクトップとターミナルで共有します。

## インストール

[GitHub Releases](https://github.com/devsepnine/hibi_ai/releases/latest) から、お使いの OS に合ったパッケージをダウンロードしてください。

| プラットフォーム | デスクトップへのインストール | その他の方法 |
|---|---|---|
| macOS、Apple Silicon または Intel | DMG を開いて `Hibi.app` を `Applications` にドラッグし、アプリケーションフォルダーから Hibi を起動します。 | Homebrew または `.tar.gz` |
| Linux、x86_64 | Debian や Ubuntu では `.deb`、Fedora や RHEL では `.rpm`、Alpine では `.apk` をインストールし、アプリメニューから Hibi を起動します。 | インストールスクリプト、glibc ベースの環境向けの Homebrew、`.tar.gz` |
| Windows、x64 | MSI を実行し、スタートメニューから Hibi を起動します。 | Scoop またはポータブル版の ZIP |

Linux の GNU ビルドには glibc 2.34 以降が必要です。Alpine では `.apk` または `linux-musl` アーカイブを使ってください。ダウンロードしたファイルは、同じリリースの `checksums.txt` と照合してください。Alpine パッケージは署名なしで配布しています。チェックサムを確認したら、root 権限で `apk add --allow-untrusted ./hibi-ai_*.apk` を実行してください。

**Homebrew**

```bash
brew install devsepnine/brew/hibi
```

**Linux インストールスクリプト**

```bash
curl -fsSL https://raw.githubusercontent.com/devsepnine/hibi_ai/main/install.sh | sh
```

スクリプトはリリースのチェックサムを検証し、既定では `~/.local` にインストールします。必要に応じて `~/.local/bin` を `PATH` に追加してください。`HIBI_PREFIX` でインストール先を変更し、`HIBI_VERSION` でリリースのバージョンを固定できます。

**Scoop**

```powershell
scoop bucket add hibi-ai https://github.com/devsepnine/scoop-bucket
scoop install hibi-ai
```

ポータブル版を使うには、アーカイブ全体を展開し、そのフォルダーから `./hibi` または `hibi.exe` を実行してください。アプリから更新する場合は、展開したフォルダーの名前とファイル構成を維持してください。

## 使い方

1. Hibi を開くか `hibi` を実行すると、デスクトップ画面が開きます。
2. **Claude Code** または **Codex CLI** を選び、必要なコンポーネントにチェックを入れます。
3. **インストール** を選んで適用します。インストール済みの項目を削除するには、**削除** を選び、対象一覧を確認してください。

SSH 接続時やグラフィカルな画面を使えない環境では、次のコマンドを実行してください。

```bash
hibi --tui
```

ターミナルのコンポーネント一覧で `?` を押すと、キーボードショートカットが表示されます。`hibi --help` と `hibi --version` はウィンドウを開かずにヘルプとバージョンを表示します。

## 設定ソース

**ソースを管理** から HTTPS の Git リポジトリやローカルフォルダーを追加できます。ソースには `skills/` のようなフォルダーや、`mcps/mcps.yaml` のような一覧ファイルを置けます。`~/.hibi/sources.yaml` を直接編集することもできます。

```yaml
sources:
  - type: git
    url: https://github.com/your-org/shared-configs.git
    branch: main
  - type: local
    path: ~/dotfiles/claude-configs
auto_update: true
```

同梱ソースの優先度が最も低くなります。同じコンポーネントが複数のソースにある場合は、一覧の最後にあるソースを使います。`hibi --sync` を実行すると、画面を開かずに Git ソースを更新できます。取得に失敗した場合はエラーを報告し、キャッシュがあればそのコピーを使います。

Claude Code 向けのソースには、独自のフック、ルール、コンテキストも追加できます。同梱の設定集では、作業手順をスキルで案内します。廃止された同梱フックの扱いは、[設定ガイド](docs/features/config-behaviors.md)を参照してください。

Hibi は、インストールしたコンポーネントとその出所を `~/.hibi/install.json` に記録します。新しいバンドルから外れたファイルは、記録したハッシュと内容が一致する場合に整理できます。編集済みのファイルは保持します。

## Hibi の更新

デスクトップアプリでバージョン表示を選ぶと、更新を確認できます。直接更新に対応するインストールでは、署名付きバンドルを検証してから置き換え、アプリを開き直します。Homebrew と Scoop では、該当バージョンのパッケージが公開されると、それぞれのパッケージマネージャーで更新します。Windows の MSI や Linux のシステムパッケージは、[GitHub Releases](https://github.com/devsepnine/hibi_ai/releases) から新しいパッケージを入手してインストールしてください。

## ビルドとコントリビュート

安定版の Rust ツールチェーンと Python 3 が必要です。リポジトリのルートで、次のコマンドを実行してください。

```bash
cargo run --locked --manifest-path tools/installer/Cargo.toml
cargo run --locked --manifest-path tools/installer/Cargo.toml --no-default-features -- --tui
python3 tools/lint-prose.py
python3 tools/lint-arch.py
cargo fmt --manifest-path tools/installer/Cargo.toml --check
cargo test --locked --manifest-path tools/installer/Cargo.toml
```

Linux のデスクトップビルドに必要なライブラリは、[リリースワークフロー](.github/workflows/release.yml) を参照してください。インストーラーの動作は [機能マップ](docs/FEATURES.md)、コードの構成は [アーキテクチャガイド](docs/ARCHITECTURE.md)、パッケージ作成は [リリース手順](docs/RUNBOOK.md) に記載しています。

不具合や機能要望は [GitHub Issues](https://github.com/devsepnine/hibi_ai/issues) に登録してください。不具合を報告する場合は、OS、Hibi のバージョン、再現手順を添えてください。コードやドキュメントの変更は、[プルリクエスト](https://github.com/devsepnine/hibi_ai/pulls) で受け付けています。

## ライセンス

Hibi は [MIT ライセンス](LICENSE) で配布しています。
