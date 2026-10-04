# hibi-ai

TUI installer for Claude Code and Codex CLI configurations.

## Features

- Interactive TUI for easy configuration management
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

The TUI will guide you through:
1. Selecting the target CLI, either Claude Code or Codex
2. Choosing components to install
3. Reviewing changes before installation
4. Installing configurations

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
  "components": ["agents/architect", "commands/qa-handoff", "skills/qa-handoff"]
}
```

`components` lists only what came from the bundled source; if you configured
extra sources, their labels appear under `other_sources` so `source` is never
read as the origin of a component that came from somewhere else.

This exists so an installed config can name its own origin. The version maps to
a release tag, so the exact source tree is recoverable, and the upstream is where
improvements go back. The `pull-request` skill reads this file to find the
repository without needing a clone. Only hibi's own directory is written; the
agent-owned `~/.claude` tree is left alone.

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
