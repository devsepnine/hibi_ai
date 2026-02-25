#!/bin/bash
set -e

VERSION="0.1.2"
NAME="hibi-ai"

echo "📦 Packaging ${NAME} v${VERSION}..."

# Create release directory
RELEASE_DIR="release"
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
        cp hibi.exe "$PKG_DIR/"
    elif [ "$PLATFORM" = "macos" ]; then
        cp hibi "$PKG_DIR/"
    else
        cp hibi-linux "$PKG_DIR/hibi"
    fi

    # Copy all config directories (exclude tools/)
    for dir in agents commands skills hooks mcps plugins rules contexts output-styles statusline; do
        if [ -d "$dir" ]; then
            cp -r "$dir" "$PKG_DIR/"
        fi
    done

    # Copy config files
    cp settings.json "$PKG_DIR/" 2>/dev/null || true
    cp CLAUDE.md "$PKG_DIR/" 2>/dev/null || true
    cp AGENTS.md "$PKG_DIR/" 2>/dev/null || true
    cp mcp.md "$PKG_DIR/" 2>/dev/null || true

    # Create archive
    cd "$RELEASE_DIR"
    if [ "$PLATFORM" = "windows" ]; then
        zip -r "${PKG_NAME}.zip" "$PKG_NAME"
        echo "✅ Created: ${PKG_NAME}.zip"
    else
        tar czf "${PKG_NAME}.tar.gz" "$PKG_NAME"
        echo "✅ Created: ${PKG_NAME}.tar.gz"
    fi
    cd ..

    # Cleanup temp directory
    rm -rf "$PKG_DIR"
done

echo ""
echo "🎉 Packaging complete!"
echo ""
ls -lh "$RELEASE_DIR"/*.{tar.gz,zip} 2>/dev/null || true
