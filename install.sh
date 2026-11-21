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

path_exists() {
    [ -e "$1" ] || [ -L "$1" ]
}

finish() {
    status=$?
    trap - EXIT
    trap '' HUP INT TERM
    if [ "$activating" = yes ] && [ ! -L "$staged_link/hibi" ] &&
        [ "$(readlink "$bin/hibi" 2>/dev/null)" = "$lib/hibi" ]; then
        committed=yes
    fi
    if [ "$committed" = no ] && [ -n "$stage" ]; then
        if [ "$activating" = yes ] && [ ! -d "$stage/next" ] && path_exists "$lib"; then
            mv -T "$lib" "$stage/failed" || status=1
        fi
        if path_exists "$previous"; then
            if path_exists "$lib" || ! mv -T "$previous" "$lib"; then
                echo "hibi-install: error: recover the previous installation from $previous" >&2
                status=1
            fi
        fi
    fi
    [ "$committed" = no ] || rm -rf "$previous"
    [ -z "$staged_link" ] || rm -rf "$staged_link"
    [ -z "$stage" ] || rm -rf "$stage"
    rm -rf "$tmpdir"
    exit "$status"
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
        Linux)
            platform="linux"
            # GNU and Alpine executables use different dynamic loaders.
            if [ -f /etc/alpine-release ]; then
                platform="linux-musl"
            fi
            ;;
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
    need flock

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
    stage=
    staged_link=
    committed=no
    activating=no
    trap finish EXIT
    trap 'exit 129' HUP
    trap 'exit 130' INT
    trap 'exit 143' TERM

    curl -fsSL "$base_url/checksums.txt" -o "$tmpdir/checksums.txt"
    if [ "$platform" = "linux-musl" ]; then
        # Releases before the desktop installer used a static musl binary in
        # the ordinary Linux archive. Keep explicit older versions usable.
        if ! awk -v n="${name}.tar.gz" '$2 == n { found = 1 } END { exit !found }' "$tmpdir/checksums.txt"; then
            platform="linux"
            name="hibi-ai-${version}-${platform}"
        fi
    fi
    echo "-> Downloading hibi-ai v${version} (${platform})..."
    curl -fsSL "$base_url/${name}.tar.gz" -o "$tmpdir/${name}.tar.gz"

    echo "-> Verifying checksum..."
    verify_checksum "$tmpdir" "${name}.tar.gz"

    echo "-> Installing to ${prefix}..."
    tar xzf "$tmpdir/${name}.tar.gz" -C "$tmpdir"
    [ -f "$tmpdir/$name/hibi" ] || fail "unexpected archive layout: $name/hibi is missing"

    mkdir -p -- "$prefix"
    prefix="$(CDPATH= cd -- "$prefix" && pwd -P)"
    lib="${prefix}/lib/hibi-ai"
    previous="$prefix/lib/.hibi-install-previous"
    bin="${prefix}/bin"
    mkdir -p "${prefix}/lib" "$bin"
    exec 9>>"${prefix}/lib/.hibi-update.lock"
    flock -n 9 || fail "another installation is using $prefix"
    if path_exists "$previous"; then
        path_exists "$lib" && fail "previous installation saved at $previous; move it aside before retrying"
        mv -T "$previous" "$lib"
    fi
    stage="$(mktemp -d "$prefix/lib/.hibi-install.XXXXXXXX")"
    staged_link="$(mktemp -d "$bin/.hibi-install.XXXXXXXX")"
    mv -T "$tmpdir/$name" "$stage/next"
    chmod +x "$stage/next/hibi"
    ln -s "$lib/hibi" "$staged_link/hibi"
    if path_exists "$lib"; then
        mv -T "$lib" "$previous"
    fi
    activating=yes
    mv -T "$stage/next" "$lib"
    mv -fT "$staged_link/hibi" "$bin/hibi"
    committed=yes

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
