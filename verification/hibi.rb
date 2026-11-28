class Hibi < Formula
  desc "Graphical and terminal installer for Claude Code and Codex CLI configurations"
  homepage "https://github.com/devsepnine/hibi_ai"
  version "2.4.1"
  license "MIT"

  on_macos do
    url "https://github.com/devsepnine/hibi_ai/releases/download/v2.4.1/hibi-ai-2.4.1-macos.tar.gz"
    sha256 "b7c2180821e4345da777fef90a53235c89775d393283739aca63d000fe005c59"
  end

  on_linux do
    url "https://github.com/devsepnine/hibi_ai/releases/download/v2.4.1/hibi-ai-2.4.1-linux.tar.gz"
    sha256 "e603b02236329f59e1c035f68c1fd1a40b523a7ac60a0ec1be3bb4a224510851"

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
