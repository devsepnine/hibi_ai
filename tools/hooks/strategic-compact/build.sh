#!/bin/bash

set -e

echo "Building suggest-compact for all platforms..."
echo ""

# Output directory
OUTPUT_DIR="../../../src/hooks/suggest-compact"
mkdir -p "$OUTPUT_DIR"

# macOS Universal Binary (Apple Silicon + Intel)
echo "Building for macOS (Universal Binary)..."

# Check if targets are installed
MISSING_TARGETS=""
if ! rustup target list | grep -q "aarch64-apple-darwin (installed)"; then
    MISSING_TARGETS="$MISSING_TARGETS aarch64-apple-darwin"
fi
if ! rustup target list | grep -q "x86_64-apple-darwin (installed)"; then
    MISSING_TARGETS="$MISSING_TARGETS x86_64-apple-darwin"
fi

if [ -n "$MISSING_TARGETS" ]; then
    echo "⚠️  Missing macOS targets:$MISSING_TARGETS"
    echo "   To install: rustup target add$MISSING_TARGETS"
    exit 1
fi

# Build for Apple Silicon (arm64)
echo "  - Building for Apple Silicon (arm64)..."
cargo build --release --target aarch64-apple-darwin

# Build for Intel (x86_64)
echo "  - Building for Intel (x86_64)..."
cargo build --release --target x86_64-apple-darwin

# Create Universal Binary
echo "  - Creating Universal Binary..."
lipo -create \
  target/aarch64-apple-darwin/release/suggest-compact \
  target/x86_64-apple-darwin/release/suggest-compact \
  -output "$OUTPUT_DIR/suggest-compact_macos"

echo "macOS Universal Binary complete (Intel + Apple Silicon)"

# Windows
echo "Building for Windows..."
cargo build --release --target x86_64-pc-windows-gnu
cp target/x86_64-pc-windows-gnu/release/suggest-compact.exe "$OUTPUT_DIR/suggest-compact.exe"
echo "Windows build complete"

# Linux
if rustup target list | grep -q "x86_64-unknown-linux-musl (installed)"; then
    echo "Building for Linux..."
    if cargo build --release --target x86_64-unknown-linux-musl 2>/dev/null; then
        cp target/x86_64-unknown-linux-musl/release/suggest-compact "$OUTPUT_DIR/suggest-compact_linux"
        echo "Linux build complete"
    else
        echo "Linux build failed (linker issue). Skipping."
    fi
else
    echo "Linux target not installed. Skipping."
fi

echo ""
echo "Build complete!"
ls -lh "$OUTPUT_DIR"/suggest-compact*
