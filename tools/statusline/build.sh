#!/bin/bash

set -e

echo "🔨 Building statusline for all platforms..."
echo ""

# Build directory
BUILD_DIR="../../src/statusline"

# Ensure statusline directory exists
mkdir -p "$BUILD_DIR"

# macOS Universal Binary (Apple Silicon + Intel)
echo "📦 Building for macOS (Universal Binary)..."

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
  target/aarch64-apple-darwin/release/statusline \
  target/x86_64-apple-darwin/release/statusline \
  -output "$BUILD_DIR/statusline_macos"

echo "✅ macOS Universal Binary complete: $BUILD_DIR/statusline_macos"
echo "   (supports both Apple Silicon and Intel Macs)"
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
