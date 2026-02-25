# hibi-ai

TUI installer for Claude Code and Codex CLI configurations.

## Features

- 🎨 Interactive TUI for easy configuration management
- 🔧 Support for both Claude Code and Codex CLI
- 📦 Component-based installation (agents, commands, skills, hooks, MCPs, plugins)
- 🌍 Cross-platform support (macOS Universal Binary [Intel + Apple Silicon], Linux, Windows)
- 🔍 Automatic MCP server detection
- ⚡ Fast and lightweight

## Installation

### Homebrew (macOS/Linux)

```bash
brew tap devsepnine/brew
brew install hibi
```

### Scoop (Windows)

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
1. Selecting target CLI (Claude Code or Codex)
2. Choosing components to install
3. Reviewing changes before installation
4. Installing configurations

## Components

- **Agents**: Specialized AI agents for different tasks
- **Commands**: Custom slash commands
- **Skills**: Domain-specific skills and knowledge
- **Hooks**: Lifecycle hooks for automation
- **MCPs**: Model Context Protocol servers
- **Plugins**: Additional functionality plugins
- **Rules**: Code style and workflow rules
- **Contexts**: Context presets
- **Output Styles**: Custom output formatting

## Building from Source

Requirements:
- Rust 2024 edition

```bash
cd tools/installer
./build.sh
```

This will create binaries for all platforms:
- `hibi` (macOS Universal Binary - supports both Intel and Apple Silicon Macs)
- `hibi-linux` (Linux x86_64)
- `hibi.exe` (Windows x86_64)

## License

MIT License - see [LICENSE](LICENSE) for details

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
