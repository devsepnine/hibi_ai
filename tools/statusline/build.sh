#!/bin/bash

set -e

echo "🔨 Building statusline for all platforms..."
echo ""

# Build directory
BUILD_DIR="../../statusline"

# Ensure statusline directory exists
mkdir -p "$BUILD_DIR"

# macOS (current platform)
echo "📦 Building for macOS..."
cargo build --release
cp target/release/statusline "$BUILD_DIR/statusline_macos"
echo "✅ macOS build complete: $BUILD_DIR/statusline_macos"
echo ""

# Windows
echo "📦 Building for Windows..."
cargo build --release --target x86_64-pc-windows-gnu
cp target/x86_64-pc-windows-gnu/release/statusline.exe "$BUILD_DIR/statusline.exe"
echo "✅ Windows build complete: $BUILD_DIR/statusline.exe"
echo ""

# Linux (using musl for static binary)
if rustup target list | grep -q "x86_64-unknown-linux-musl (installed)"; then
    echo "📦 Building for Linux..."
    cargo build --release --target x86_64-unknown-linux-musl
    cp target/x86_64-unknown-linux-musl/release/statusline "$BUILD_DIR/statusline_linux"
    echo "✅ Linux build complete: $BUILD_DIR/statusline_linux"
else
    echo "⚠️  Linux target not installed. Skipping Linux build."
    echo "   To install: rustup target add x86_64-unknown-linux-musl"
    echo "   To install linker: brew install filosottile/musl-cross/musl-cross"
fi

echo ""
echo "🎉 Build complete!"
echo ""
echo "Output files:"
ls -lh "$BUILD_DIR"/statusline*
