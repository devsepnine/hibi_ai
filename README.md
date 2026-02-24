# hibi-ai

TUI installer for Claude Code and Codex CLI configurations.

## Features

- 🎨 Interactive TUI for easy configuration management
- 🔧 Support for both Claude Code and Codex CLI
- 📦 Component-based installation (agents, commands, skills, hooks, MCPs, plugins)
- 🌍 Cross-platform support (macOS, Linux, Windows)
- 🔍 Automatic MCP server detection
- ⚡ Fast and lightweight

## Installation

### Homebrew (macOS/Linux)

```bash
brew install devsepnine/hibi_ai/hibi-ai
```

### Manual Installation

1. Download the latest release for your platform:
   - [macOS](https://github.com/devsepnine/hibi_ai/releases/latest/download/hibi-ai-0.1.0-macos.tar.gz)
   - [Linux](https://github.com/devsepnine/hibi_ai/releases/latest/download/hibi-ai-0.1.0-linux.tar.gz)
   - [Windows](https://github.com/devsepnine/hibi_ai/releases/latest/download/hibi-ai-0.1.0-windows.zip)

2. Extract the archive:
   ```bash
   tar xzf hibi-ai-0.1.0-macos.tar.gz  # macOS/Linux
   # or
   unzip hibi-ai-0.1.0-windows.zip     # Windows
   ```

3. Run the installer:
   ```bash
   ./hibi
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
- `hibi` (macOS)
- `hibi-linux` (Linux)
- `hibi.exe` (Windows)

## License

MIT License - see [LICENSE](LICENSE) for details

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
