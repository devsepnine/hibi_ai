#!/bin/bash

set -e

echo "🔨 Building memory-persistence hooks for all platforms..."
echo ""

# Build directories
LOAD_CONTEXT_DIR="../../../hooks/load-context"
PRESERVE_CONTEXT_DIR="../../../hooks/preserve-context"
PERSIST_SESSION_DIR="../../../hooks/persist-session"

# Ensure hooks directories exist
mkdir -p "$LOAD_CONTEXT_DIR"
mkdir -p "$PRESERVE_CONTEXT_DIR"
mkdir -p "$PERSIST_SESSION_DIR"

# macOS (current platform)
echo "📦 Building for macOS..."
cargo build --release

cp target/release/load-context "$LOAD_CONTEXT_DIR/load-context_macos"
cp target/release/preserve-context "$PRESERVE_CONTEXT_DIR/preserve-context_macos"
cp target/release/persist-session "$PERSIST_SESSION_DIR/persist-session_macos"

echo "✅ macOS builds complete"
echo ""

# Windows
echo "📦 Building for Windows..."
cargo build --release --target x86_64-pc-windows-gnu

cp target/x86_64-pc-windows-gnu/release/load-context.exe "$LOAD_CONTEXT_DIR/load-context.exe"
cp target/x86_64-pc-windows-gnu/release/preserve-context.exe "$PRESERVE_CONTEXT_DIR/preserve-context.exe"
cp target/x86_64-pc-windows-gnu/release/persist-session.exe "$PERSIST_SESSION_DIR/persist-session.exe"

echo "✅ Windows builds complete"
echo ""

# Linux (using musl for static binary)
if rustup target list | grep -q "x86_64-unknown-linux-musl (installed)"; then
    echo "📦 Building for Linux..."
    if cargo build --release --target x86_64-unknown-linux-musl 2>/dev/null; then
        cp target/x86_64-unknown-linux-musl/release/load-context "$LOAD_CONTEXT_DIR/load-context_linux"
        cp target/x86_64-unknown-linux-musl/release/preserve-context "$PRESERVE_CONTEXT_DIR/preserve-context_linux"
        cp target/x86_64-unknown-linux-musl/release/persist-session "$PERSIST_SESSION_DIR/persist-session_linux"

        echo "✅ Linux builds complete"
    else
        echo "⚠️  Linux build failed (linker issue). Skipping Linux build."
        echo "   To install musl linker: brew install filosottile/musl-cross/musl-cross"
    fi
else
    echo "⚠️  Linux target not installed. Skipping Linux build."
    echo "   To install: rustup target add x86_64-unknown-linux-musl"
    echo "   To install linker: brew install filosottile/musl-cross/musl-cross"
fi

echo ""
echo "🎉 Build complete!"
echo ""
echo "Output files:"
echo "=== load-context ==="
ls -lh "$LOAD_CONTEXT_DIR"/load-context*
echo ""
echo "=== preserve-context ==="
ls -lh "$PRESERVE_CONTEXT_DIR"/preserve-context*
echo ""
echo "=== persist-session ==="
ls -lh "$PERSIST_SESSION_DIR"/persist-session*
