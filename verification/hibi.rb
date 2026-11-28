class Hibi < Formula
  desc "Graphical and terminal installer for Claude Code and Codex CLI configurations"
  homepage "https://github.com/devsepnine/hibi_ai"
  version "2.4.2"
  license "MIT"

  on_macos do
    url "https://github.com/devsepnine/hibi_ai/releases/download/v2.4.2/hibi-ai-2.4.2-macos.tar.gz"
    sha256 "c73e73970ebb2627d76acfccbdf16e4b711a43253eb0d6bfe43ed8584730bee6"
  end

  on_linux do
    url "https://github.com/devsepnine/hibi_ai/releases/download/v2.4.2/hibi-ai-2.4.2-linux.tar.gz"
    sha256 "ff3adb641196d56c74f74c22dd204dc491ae697cb09f29262eb21a3635c9129f"

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
