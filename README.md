<p><img src="tools/installer/assets/hibi.png" alt="Hibi icon" width="88"></p>

# hibi-ai

Hibi is a desktop and terminal app for managing Claude Code and Codex CLI configurations. Choose the skills and other components you need from the included collection, a Git repository, or a local folder.

**English** · [한국어](README.ko.md) · [日本語](README.ja.md)

[Download](https://github.com/devsepnine/hibi_ai/releases/latest) · [Release notes](https://github.com/devsepnine/hibi_ai/releases) · [Report an issue](https://github.com/devsepnine/hibi_ai/issues)

## Features

- Follow the system appearance or choose an iced palette. The desktop app remembers your choice.
- Use the keyboard to work in a terminal over SSH.
- Search and filter components, compare installed files with their source, and review selected items before removing them.
- Manage skills, configuration, and MCP servers for both tools. Claude Code also supports agents, slash commands, plugins, hooks, and output styles.
- Add Git repositories and local folders as sources. The desktop and terminal interfaces share sources and installation records.

## Install

Download the package for your system from [GitHub Releases](https://github.com/devsepnine/hibi_ai/releases/latest).

| Platform | Desktop install | Other options |
|---|---|---|
| macOS, Apple Silicon or Intel | Open the DMG and drag `Hibi.app` to `Applications`. Launch Hibi from Applications. | Homebrew or `.tar.gz` |
| Linux, x86_64 | Install the `.deb` on Debian or Ubuntu, `.rpm` on Fedora or RHEL, or `.apk` on Alpine. Launch Hibi from your app menu. | Install script, Homebrew on glibc based distros, or `.tar.gz` |
| Windows, x64 | Run the MSI and launch Hibi from the Start menu. | Scoop or portable ZIP |

Linux GNU builds require glibc 2.34 or later. On Alpine, use the `.apk` or the `linux-musl` archive. Check downloaded files against `checksums.txt` from the same release. The Alpine package is unsigned. After verifying it, install it with `apk add --allow-untrusted ./hibi-ai_*.apk` as root.

**Homebrew**

```bash
brew install devsepnine/brew/hibi
```

**Linux install script**

```bash
curl -fsSL https://raw.githubusercontent.com/devsepnine/hibi_ai/main/install.sh | sh
```

The script verifies the release checksum and installs to `~/.local` by default. If needed, add `~/.local/bin` to your `PATH`. Set `HIBI_PREFIX` to change the install location or `HIBI_VERSION` to pin a release.

**Scoop**

```powershell
scoop bucket add hibi-ai https://github.com/devsepnine/scoop-bucket
scoop install hibi-ai
```

For a portable install, extract the full archive and run `./hibi` or `hibi.exe` from the extracted folder. Keep the folder name and bundle contents in place to use updates from the app.

## Quick start

1. Open Hibi or run `hibi` to launch the desktop interface.
2. Select **Claude Code** or **Codex CLI** and check the components you want.
3. Choose **Install** to apply the selection. To remove installed items, choose **Remove** and review the list before confirming.

Over SSH or without a graphical display, run:

```bash
hibi --tui
```

Press `?` in the terminal component list to see the keyboard shortcuts. Use `hibi --help` or `hibi --version` to inspect the command without opening a window.

## Configuration sources

Open **Manage sources** to add HTTPS Git repositories or local folders. A source can provide a directory such as `skills/` or a catalog such as `mcps/mcps.yaml`. You can also edit `~/.hibi/sources.yaml`:

```yaml
sources:
  - type: git
    url: https://github.com/your-org/shared-configs.git
    branch: main
  - type: local
    path: ~/dotfiles/claude-configs
auto_update: true
```

The bundled source has the lowest priority. If two sources contain the same component, the later entry wins. Run `hibi --sync` to refresh Git sources without opening the interface. If a fetch fails, Hibi uses a cached copy when available and reports the failure.

Claude Code sources can also provide custom hooks, rules, and contexts. The included collection uses skills for its workflow guidance. See the [configuration guide](docs/features/config-behaviors.md) for how Hibi handles deprecated bundled hooks.

Hibi tracks installed components and their origins in `~/.hibi/install.json`. When a newer bundle drops a previously installed file, Hibi can clean up the old copy if its recorded content hash still matches. Files you edited are kept.

## Update Hibi

Select the version in the desktop app to check for updates. Hibi verifies signed bundles before replacing eligible direct installations and reopens the app. Supported Homebrew and Scoop installations update through their package manager when the matching package is published. For system packages such as the Windows MSI or Linux packages, install the newer package from [GitHub Releases](https://github.com/devsepnine/hibi_ai/releases).

## Build and contribute

Use the stable Rust toolchain and Python 3. Run the following commands from the repository root:

```bash
cargo run --locked --manifest-path tools/installer/Cargo.toml
cargo run --locked --manifest-path tools/installer/Cargo.toml --no-default-features -- --tui
python3 tools/lint-prose.py
python3 tools/lint-arch.py
cargo fmt --manifest-path tools/installer/Cargo.toml --check
cargo test --locked --manifest-path tools/installer/Cargo.toml
```

On Linux, see the [release workflow](.github/workflows/release.yml) for desktop build dependencies. See the [feature map](docs/FEATURES.md) for the installer, the [architecture guide](docs/ARCHITECTURE.md) for code boundaries, and the [release runbook](docs/RUNBOOK.md) for packaging.

Open an [issue](https://github.com/devsepnine/hibi_ai/issues) for a bug or feature request. Include your OS, Hibi version, and steps to reproduce a bug. For code or documentation changes, submit a [pull request](https://github.com/devsepnine/hibi_ai/pulls).

## License

Hibi is available under the [MIT License](LICENSE).
