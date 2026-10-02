#!/bin/sh
# hibi installer — downloads a release archive, verifies its checksum, and
# installs under a prefix:
#
#   curl -fsSL https://raw.githubusercontent.com/devsepnine/hibi_ai/main/install.sh | sh
#
# Environment overrides:
#   HIBI_VERSION   version to install, with or without "v" (default: latest release)
#   HIBI_PREFIX    install prefix (default: ~/.local — e.g. /usr/local for system-wide)
#
# Layout: <prefix>/lib/hibi-ai/ holds the binary plus bundled configs, and
# <prefix>/bin/hibi symlinks to it. The binary resolves bundled configs
# relative to the real executable path, so the symlink must not be replaced
# with a copy.
set -eu

REPO="devsepnine/hibi_ai"

fail() {
    echo "hibi-install: error: $*" >&2
    exit 1
}

need() {
    command -v "$1" >/dev/null 2>&1 || fail "required tool not found: $1"
}

verify_checksum() {
    dir="$1"
    file="$2"
    # Exact filename match against the "<hash>  <name>" lines of checksums.txt.
    # A missing match feeds empty input to the checker, which exits non-zero.
    if command -v sha256sum >/dev/null 2>&1; then
        (cd "$dir" && awk -v n="$file" '$2 == n { print $1 "  " $2 }' checksums.txt | sha256sum -c - >/dev/null)
    elif command -v shasum >/dev/null 2>&1; then
        (cd "$dir" && awk -v n="$file" '$2 == n { print $1 "  " $2 }' checksums.txt | shasum -a 256 -c - >/dev/null)
    else
        fail "no SHA-256 tool found (need sha256sum or shasum)"
    fi
}

main() {
    os="$(uname -s)"
    arch="$(uname -m)"

    case "$os" in
        Linux) platform="linux" ;;
        Darwin) fail "on macOS use Homebrew instead: brew install devsepnine/brew/hibi" ;;
        MINGW* | MSYS* | CYGWIN*)
            fail "on Windows use Scoop instead: scoop bucket add hibi-ai https://github.com/devsepnine/scoop-bucket && scoop install hibi-ai" ;;
        *) fail "unsupported OS: $os" ;;
    esac

    case "$arch" in
        x86_64 | amd64) ;;
        *) fail "unsupported architecture: $arch (releases currently ship x86_64 only)" ;;
    esac

    need curl
    need tar

    if [ -n "${HIBI_VERSION:-}" ]; then
        version="${HIBI_VERSION#v}"
    else
        version="$(
            curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" |
                sed -n 's/.*"tag_name": *"v\{0,1\}\([^"]*\)".*/\1/p' |
                head -1
        )"
    fi
    case "$version" in
        *[!0-9A-Za-z.+-]* | "") fail "invalid or unresolvable version: '$version'" ;;
    esac

    prefix="${HIBI_PREFIX:-$HOME/.local}"
    name="hibi-ai-${version}-${platform}"
    base_url="https://github.com/$REPO/releases/download/v${version}"

    tmpdir="$(mktemp -d)"
    trap 'rm -rf "$tmpdir"' EXIT

    echo "-> Downloading hibi-ai v${version} (${platform})..."
    curl -fsSL "$base_url/${name}.tar.gz" -o "$tmpdir/${name}.tar.gz"
    curl -fsSL "$base_url/checksums.txt" -o "$tmpdir/checksums.txt"

    echo "-> Verifying checksum..."
    verify_checksum "$tmpdir" "${name}.tar.gz"

    echo "-> Installing to ${prefix}..."
    tar xzf "$tmpdir/${name}.tar.gz" -C "$tmpdir"
    [ -f "$tmpdir/$name/hibi" ] || fail "unexpected archive layout: $name/hibi is missing"

    lib="${prefix}/lib/hibi-ai"
    bin="${prefix}/bin"
    mkdir -p "${prefix}/lib" "$bin"
    # Stage next to the target (same filesystem) so the swap cannot strand the
    # user without a working install if anything before it fails.
    rm -rf "$lib.new"
    mv "$tmpdir/$name" "$lib.new"
    rm -rf "$lib"
    mv "$lib.new" "$lib"
    chmod +x "$lib/hibi"
    ln -sf "$lib/hibi" "$bin/hibi"

    echo "hibi v${version} installed: $bin/hibi"

    case ":$PATH:" in
        *":$bin:"*) ;;
        *)
            echo ""
            echo "NOTE: $bin is not in your PATH. Add it with:"
            echo "  export PATH=\"$bin:\$PATH\""
            ;;
    esac
}

# curl|sh streams the script as it downloads; wrapping the body keeps a dropped
# connection from executing a truncated script.
main "$@"
