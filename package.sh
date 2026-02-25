#!/bin/bash
set -e

VERSION="0.1.3"
NAME="hibi-ai"

echo "📦 Packaging ${NAME} v${VERSION}..."

# Store project root
PROJECT_ROOT="$(pwd)"

# Prepare dist directory
DIST_DIR="dist"
echo ""
echo "🔨 Preparing dist directory..."
mkdir -p "$DIST_DIR"

# Check if installer binaries exist
if [ ! -f "$DIST_DIR/hibi" ]; then
    echo "❌ Error: Installer binaries not found in dist/"
    echo "   Please run: cd tools/installer && ./build.sh"
    exit 1
fi

# Copy statusline from src
echo "  - Copying statusline..."
rm -rf "$DIST_DIR/statusline"
cp -r src/statusline "$DIST_DIR/"

# Copy hooks from src
echo "  - Copying hooks..."
rm -rf "$DIST_DIR/hooks"
cp -r src/hooks "$DIST_DIR/"

# Copy config directories from src
echo "  - Copying configuration directories..."
for dir in agents commands skills rules contexts mcps plugins output-styles; do
    if [ -d "src/$dir" ]; then
        cp -r "src/$dir" "$DIST_DIR/"
    fi
done

# Copy config files from src
echo "  - Copying configuration files..."
cp src/settings.json "$DIST_DIR/" 2>/dev/null || true
cp src/CLAUDE.md "$DIST_DIR/" 2>/dev/null || true
cp src/AGENTS.md "$DIST_DIR/" 2>/dev/null || true
cp src/mcp.md "$DIST_DIR/" 2>/dev/null || true

echo "✅ dist directory ready"

# Create release directory with version
RELEASE_DIR="release/v${VERSION}"
echo ""
echo "📦 Creating release packages..."
rm -rf "$RELEASE_DIR"
mkdir -p "$RELEASE_DIR"

# Package for each platform
for PLATFORM in "macos" "linux" "windows"; do
    echo ""
    echo "📦 Creating package for ${PLATFORM}..."

    PKG_NAME="${NAME}-${VERSION}-${PLATFORM}"
    PKG_DIR="${RELEASE_DIR}/${PKG_NAME}"

    mkdir -p "$PKG_DIR"

    # Copy binary
    if [ "$PLATFORM" = "windows" ]; then
        cp "$DIST_DIR/hibi.exe" "$PKG_DIR/"
    elif [ "$PLATFORM" = "macos" ]; then
        cp "$DIST_DIR/hibi" "$PKG_DIR/"
    else
        cp "$DIST_DIR/hibi-linux" "$PKG_DIR/hibi"
    fi

    # Copy all config directories from dist
    for dir in agents commands skills hooks mcps plugins rules contexts output-styles statusline; do
        if [ -d "$DIST_DIR/$dir" ]; then
            cp -r "$DIST_DIR/$dir" "$PKG_DIR/"
        fi
    done

    # Copy config files from dist
    cp "$DIST_DIR/settings.json" "$PKG_DIR/" 2>/dev/null || true
    cp "$DIST_DIR/CLAUDE.md" "$PKG_DIR/" 2>/dev/null || true
    cp "$DIST_DIR/AGENTS.md" "$PKG_DIR/" 2>/dev/null || true
    cp "$DIST_DIR/mcp.md" "$PKG_DIR/" 2>/dev/null || true

    # Create archive
    cd "$PROJECT_ROOT/$RELEASE_DIR"
    if [ "$PLATFORM" = "windows" ]; then
        zip -q -r "${PKG_NAME}.zip" "$PKG_NAME"
        echo "✅ Created: ${PKG_NAME}.zip"
    else
        tar czf "${PKG_NAME}.tar.gz" "$PKG_NAME"
        echo "✅ Created: ${PKG_NAME}.tar.gz"
    fi
    cd "$PROJECT_ROOT"

    # Cleanup temp directory
    rm -rf "$PKG_DIR"
done

echo ""
echo "📝 Generating checksums..."
cd "$PROJECT_ROOT/$RELEASE_DIR"
shasum -a 256 *.tar.gz *.zip > checksums.txt 2>/dev/null || true
echo "✅ Created: checksums.txt"
cd "$PROJECT_ROOT"

echo ""
echo "🎉 Packaging complete!"
echo ""
ls -lh "$RELEASE_DIR"/*.{tar.gz,zip} 2>/dev/null || true
echo ""
echo "Checksums:"
cat "$RELEASE_DIR/checksums.txt"
