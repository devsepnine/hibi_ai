class Hibi < Formula
  desc "Graphical and terminal installer for Claude Code and Codex CLI configurations"
  homepage "https://github.com/devsepnine/hibi_ai"
  version "2.2.0"
  license "MIT"

  on_macos do
    url "https://github.com/devsepnine/hibi_ai/releases/download/v2.2.0/hibi-ai-2.2.0-macos.tar.gz"
    sha256 "10eb04e7eed074fd7dc5d21ac4570aee6c43116b2d64858a75e90774601fa62f"
  end

  on_linux do
    url "https://github.com/devsepnine/hibi_ai/releases/download/v2.2.0/hibi-ai-2.2.0-linux.tar.gz"
    sha256 "a2847a856a3d3acd104b3221ef2577379028e5e5b7c75805cb77a27ed3979c1d"

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
