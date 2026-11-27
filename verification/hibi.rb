class Hibi < Formula
  desc "Graphical and terminal installer for Claude Code and Codex CLI configurations"
  homepage "https://github.com/devsepnine/hibi_ai"
  version "2.3.0"
  license "MIT"

  on_macos do
    url "https://github.com/devsepnine/hibi_ai/releases/download/v2.3.0/hibi-ai-2.3.0-macos.tar.gz"
    sha256 "7714793893b32a8f393dbcfce96c7a64eca11e96d1861ef8a6a88e921dbd3609"
  end

  on_linux do
    url "https://github.com/devsepnine/hibi_ai/releases/download/v2.3.0/hibi-ai-2.3.0-linux.tar.gz"
    sha256 "e4d02c19a9d85eb430d6cf5a361efaacdf8e70c567885e7f07d4858d414926de"

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
