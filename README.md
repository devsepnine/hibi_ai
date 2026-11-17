# hibi-ai

Rust iced GUI installer for Claude Code and Codex CLI configurations.

## Features

- Native GUI with dark and light themes, plus terminal mode
- Support for both Claude Code and Codex CLI
- Component-based installation of agents, commands, skills, hooks, MCPs, and plugins
- Cross-platform support: macOS Universal Binary for Intel and Apple Silicon, Linux, Windows
- Automatic MCP server detection
- Fast and lightweight

## Installation

### Quick Install, Linux

```bash
curl -fsSL https://raw.githubusercontent.com/devsepnine/hibi_ai/main/install.sh | sh
```

Installs to `~/.local` after verifying the release checksum. Set `HIBI_PREFIX=/usr/local` for a system-wide install, or `HIBI_VERSION=x.y.z` to pin a version.

### Homebrew, macOS/Linux

```bash
brew tap devsepnine/brew
brew install hibi
```

### Linux Packages: deb, rpm, apk

Download the package for your distro from [Releases](https://github.com/devsepnine/hibi_ai/releases/latest), then:

```bash
sudo apt install ./hibi-ai_*_amd64.deb       # Debian/Ubuntu
sudo dnf install ./hibi-ai-*-1.x86_64.rpm    # Fedora/RHEL
apk add --allow-untrusted hibi-ai_*.apk      # Alpine (unsigned package)
```

The apk is unsigned; verify any package against `checksums.txt` from the same release before installing.

The packages install the binary to `/usr/bin/hibi` and bundled configs to `/usr/share/hibi`.

### Scoop, Windows

```bash
scoop bucket add hibi-ai https://github.com/devsepnine/scoop-bucket
scoop install hibi-ai
```

### Manual Installation

1. Download the latest release for your platform from [Releases](https://github.com/devsepnine/hibi_ai/releases/latest)

2. Extract the archive and run the installer:
   ```bash
   # macOS/Linux
   tar xzf hibi-ai-*-macos.tar.gz    # or *-linux.tar.gz
   ./hibi

   # Windows
   # Extract the zip file and run:
   hibi.exe
   ```

## Usage

Simply run `hibi` to launch the interactive installer:

```bash
hibi
```

The installer will guide you through:
1. Selecting the target CLI, either Claude Code or Codex
2. Choosing components to install
3. Reviewing changes before installation
4. Installing configurations

### GUI와 터미널 실행

`v2`에서 `hibi`를 실행하면 iced GUI가 열립니다. Light / Dark 버튼으로 테마를 전환합니다.
기존 기능과 설치 기록, 소스 설정을 공유합니다. 화면과 검증 항목은 [GUI 기능맵](docs/features/installer-gui.md)에 정리했습니다.

SSH나 그래픽 환경이 없는 터미널에서는 다음 명령을 사용합니다.

```bash
hibi --tui
```

GUI를 포함한 개발 빌드는 다음 명령을 사용합니다.

```bash
cargo run --manifest-path tools/installer/Cargo.toml
```

GUI 의존성 없이 터미널 전용 바이너리를 빌드할 수도 있습니다.

```bash
cargo build --manifest-path tools/installer/Cargo.toml --no-default-features
```

`--help`, `--version`, `--sync`는 창을 열지 않고 실행합니다.
Linux GNU 릴리즈는 glibc 2.35 이상이 필요합니다. Alpine은 별도의 musl 바이너리를 사용합니다.
deb, rpm, apk 패키지는 GUI 실행에 필요한 라이브러리와 기본 글꼴을 함께 설치합니다.

### Multi-Source Support

By default, hibi uses bundled configurations from the release package. You can add additional sources, meaning git repos or local directories, via `~/.hibi/sources.yaml`:

```yaml
sources:
  # Git source: pulls from a remote repository
  - type: git
    url: "https://github.com/your-org/shared-configs.git"
    branch: main

  # Local source: uses a local directory
  - type: local
    path: "~/dotfiles/claude-configs"

# Optional: disable auto-update for git sources (default: true)
auto_update: true
```

**Priority**: Bundled, the lowest → first source → ... → last source, the highest. When the same file exists in multiple sources, the last one wins.

**Update git sources** without launching the TUI:

```bash
hibi --sync
```

**Offline behavior**: If a git fetch fails but a cached copy exists, hibi uses the stale cache. Bundled source always works offline.

**Source requirements**: Each source directory must contain at least one of `agents/`, `commands/`, `contexts/`, `rules/`, `skills/`, `hooks/`, `output-styles/`, `statusline/`, `mcps/mcps.yaml`, `plugins/plugins.yaml`, `settings.json`, `CLAUDE.md`, or `AGENTS.md`. The bundled source is checked more strictly: it needs both `agents/` and `settings.json`.

### Install Provenance

After every install or removal, hibi records where your configuration came from
in `~/.hibi/install.json`:

```json
{
  "source": "https://github.com/devsepnine/hibi_ai",
  "version": "v1.16.0",
  "target": ".claude",
  "updated_at": "2026-08-06T05:41:00Z",
  "components": ["agents/architect.md", "commands/qa-handoff.md", "skills/qa-handoff/SKILL.md"]
}
```

`components` lists only what came from the bundled source; if you configured
extra sources, their labels appear under `other_sources` so `source` is never
read as the origin of a component that came from somewhere else.

This exists so an installed config can name its own origin. The version maps to
a release tag, so the exact source tree is recoverable, and the upstream is where
improvements go back. The `pull-request` skill reads this file to find the
repository without needing a clone. The record lives in hibi's own directory,
so no bookkeeping file is added to the agent-owned `~/.claude` tree.

Since v1.22.0, the record also holds `hashes`: the SHA-256 of each bundled agent,
command, context, rule, skill, and output-style file as hibi wrote it, line
endings normalized, keyed by the same IDs. A file you edited gets no hash, and
an older record without `hashes` still reads. At startup hibi uses them to
remove recorded files the bundled source no longer ships, but only when a file
still matches its hash. A file you edited, one without a hash, and anything the
record does not list stay where they are, and the status bar says what was
removed or kept.

## Components

- **Agents**: Specialized AI agents for different tasks
- **Commands**: Custom slash commands
- **Skills**: Domain-specific skills and knowledge
- **MCPs**: Model Context Protocol servers
- **Plugins**: Additional functionality plugins
- **Output Styles**: Custom output formatting
- **Hooks**: Lifecycle hooks. All bundled hooks are deprecated and removed from existing installs; the native Skill system replaces them
- **Rules** / **Contexts**: still supported as component types for your own sources, but the bundled configuration no longer ships either, because policies moved into skills

## Building from Source

Requirements:
- Rust 2024 edition

```bash
cd tools/installer
./build.sh
```

This will create binaries for all platforms:
- `hibi`: macOS Universal Binary that supports both Intel and Apple Silicon Macs
- `hibi-linux`: Linux x86_64
- `hibi.exe`: Windows x86_64

## License

MIT License, see [LICENSE](LICENSE) for details

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
