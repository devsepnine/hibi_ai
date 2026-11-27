class Hibi < Formula
  desc "Graphical and terminal installer for Claude Code and Codex CLI configurations"
  homepage "https://github.com/devsepnine/hibi_ai"
  version "2.1.3"
  license "MIT"

  on_macos do
    url "https://github.com/devsepnine/hibi_ai/releases/download/v2.1.3/hibi-ai-2.1.3-macos.tar.gz"
    sha256 "16e1a6441ff5d5bab4ef756f7948d64835574d173a63ace8700122a8f6cdc4dc"
  end

  on_linux do
    url "https://github.com/devsepnine/hibi_ai/releases/download/v2.1.3/hibi-ai-2.1.3-linux.tar.gz"
    sha256 "4f03dfce253c460cc9ee6e4c56c3522ad01f7e5dc5a9f3be71b05a7712c58422"

    depends_on "patchelf" => :build
    depends_on "libx11"
    depends_on "libxcursor"
    depends_on "libxi"
    depends_on "libxkbcommon"
    depends_on "libxrandr"
    depends_on "wayland"
  end

  def install
    bin.install "hibi"
    # The prebuilt GUI loads the display libraries at runtime from the brew prefix.
    system "patchelf", "--add-rpath", HOMEBREW_PREFIX/"lib", bin/"hibi" if OS.linux?

    share_dir = share/"hibi"
    %w[agents commands contexts hooks mcps output-styles plugins rules skills statusline].each do |d|
      share_dir.install d if File.exist?(d)
    end
    share_dir.install "settings.json" if File.exist?("settings.json")
    share_dir.install "CLAUDE.md" if File.exist?("CLAUDE.md")
    share_dir.install "AGENTS.md" if File.exist?("AGENTS.md")
    share_dir.install "mcp.md" if File.exist?("mcp.md")
  end

  test do
    assert_match "hibi v#{version} ", shell_output("#{bin}/hibi --version")
    assert_path_exists share/"hibi/skills/technical-writing/SKILL.md"
  end
end
