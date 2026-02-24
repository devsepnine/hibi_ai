#!/bin/bash

set -e

echo "Building suggest-compact for all platforms..."
echo ""

# Output directory
OUTPUT_DIR="../../../hooks/suggest-compact"
mkdir -p "$OUTPUT_DIR"

# macOS
echo "Building for macOS..."
cargo build --release
cp target/release/suggest-compact "$OUTPUT_DIR/suggest-compact_macos"
echo "macOS build complete"

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
