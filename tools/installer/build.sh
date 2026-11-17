#!/bin/bash
set -euo pipefail

cd "$(dirname "$0")"
BUILD_DIR="../../dist"
mkdir -p "$BUILD_DIR"

# Build each platform on its native runner. GUI display libraries cannot use
# the old static musl cross build from macOS.
PLATFORM="${1:-native}"
if [ "$PLATFORM" = native ]; then
    case "$(uname -s)" in
        Darwin) PLATFORM=macos ;;
        Linux)
            case "$(rustc -vV | sed -n 's/^host: //p')" in
                *-musl) PLATFORM=linux-musl ;;
                *) PLATFORM=linux ;;
            esac
            ;;
        MINGW*|MSYS*|CYGWIN*) PLATFORM=windows ;;
        *) echo "Unsupported build host" >&2; exit 1 ;;
    esac
fi

case "$PLATFORM" in
    macos)
        rustup target add aarch64-apple-darwin x86_64-apple-darwin
        cargo build --locked --release --target aarch64-apple-darwin
        cargo build --locked --release --target x86_64-apple-darwin
        lipo -create \
            target/aarch64-apple-darwin/release/hibi \
            target/x86_64-apple-darwin/release/hibi \
            -output "$BUILD_DIR/hibi"
        ;;
    windows)
        cargo build --locked --release
        cp target/release/hibi.exe "$BUILD_DIR/hibi.exe"
        ;;
    linux)
        cargo build --locked --release --target x86_64-unknown-linux-gnu
        cp target/x86_64-unknown-linux-gnu/release/hibi "$BUILD_DIR/hibi-linux"
        ;;
    linux-musl)
        cargo build --locked --release --target x86_64-unknown-linux-musl
        cp target/x86_64-unknown-linux-musl/release/hibi "$BUILD_DIR/hibi-linux-musl"
        ;;
    *) echo "Usage: $0 [native|macos|windows|linux|linux-musl]" >&2; exit 1 ;;
esac

echo "Build complete: $PLATFORM"
