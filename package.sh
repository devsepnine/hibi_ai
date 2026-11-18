#!/bin/bash
set -e

VERSION="2.0.0-beta"
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
for binary in hibi hibi.exe hibi-linux hibi-linux-musl; do
    if [ ! -f "$DIST_DIR/$binary" ]; then
        echo "❌ Error: Installer binaries not found in dist/: $binary"
        echo "   Build macos, windows, linux and linux-musl on their native hosts first."
        exit 1
    fi
done

bundle_dirs=(agents commands skills hooks mcps plugins output-styles statusline)
bundle_files=(settings.json CLAUDE.md AGENTS.md)
echo "  - Copying bundled configuration..."
for dir in "${bundle_dirs[@]}"; do
    # Start with an empty destination so deleted components stay out of the release.
    rm -rf "$DIST_DIR/$dir"
    cp -R "src/$dir" "$DIST_DIR/"
done
for file in "${bundle_files[@]}"; do
    cp "src/$file" "$DIST_DIR/"
done

# Strip Korean reference files (-ko) from the bundle. The installer scanner
# skips them at install time (stem ends with "-ko"), so they are never
# installed; removing them here keeps release archives lean.
echo "  - Stripping -ko reference files..."
find "$DIST_DIR" -type f -name '*-ko.md' -delete

echo "✅ dist directory ready"

# Create release directory with version
RELEASE_DIR="release/v${VERSION}"
echo ""
echo "📦 Creating release packages..."
rm -rf "$RELEASE_DIR"
mkdir -p "$RELEASE_DIR"

# Package for each platform
for PLATFORM in "macos" "linux" "linux-musl" "windows"; do
    echo ""
    echo "📦 Creating package for ${PLATFORM}..."

    PKG_NAME="${NAME}-${VERSION}-${PLATFORM}"
    PKG_DIR="${RELEASE_DIR}/${PKG_NAME}"

    mkdir -p "$PKG_DIR"
    echo "$VERSION" > "$PKG_DIR/.hibi-bundle"

    # Copy binary
    if [ "$PLATFORM" = "windows" ]; then
        cp "$DIST_DIR/hibi.exe" "$PKG_DIR/"
    elif [ "$PLATFORM" = "macos" ]; then
        cp "$DIST_DIR/hibi" "$PKG_DIR/"
    elif [ "$PLATFORM" = "linux-musl" ]; then
        cp "$DIST_DIR/hibi-linux-musl" "$PKG_DIR/hibi"
    else
        cp "$DIST_DIR/hibi-linux" "$PKG_DIR/hibi"
    fi

    for item in "${bundle_dirs[@]}" "${bundle_files[@]}"; do
        cp -R "$DIST_DIR/$item" "$PKG_DIR/"
    done

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

# Linux native packages (deb/rpm/apk). Optional locally — the release workflow
# installs nfpm before this script runs; without it the archives above still
# ship on their own.
if command -v nfpm >/dev/null 2>&1; then
    echo ""
    echo "📦 Creating Linux packages (deb/rpm/apk)..."
    for PACKAGER in deb rpm apk; do
        package_version="$VERSION"
        if [[ "$PACKAGER" == apk && "$VERSION" =~ ^(.+)-beta\.([0-9]+)$ ]]; then
            # Alpine numbers its beta suffix directly, for example 2.0.0_beta1.
            package_version="${BASH_REMATCH[1]}-beta${BASH_REMATCH[2]}"
        fi
        VERSION="$package_version" nfpm pkg --config nfpm.yaml --packager "$PACKAGER" --target "$RELEASE_DIR/"
        echo "✅ Created: ${PACKAGER} package"
    done
else
    echo ""
    echo "⚠️  nfpm not found — skipping deb/rpm/apk packages"
    echo "   To install: brew install nfpm"
fi

echo ""
echo "📝 Generating checksums..."
cd "$PROJECT_ROOT/$RELEASE_DIR"
shopt -s nullglob
ARTIFACTS=(*.tar.gz *.zip *.deb *.rpm *.apk)
shopt -u nullglob
if [ ${#ARTIFACTS[@]} -gt 0 ]; then
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "${ARTIFACTS[@]}" > checksums.txt
    else
        shasum -a 256 "${ARTIFACTS[@]}" > checksums.txt
    fi
    echo "✅ Created: checksums.txt"
fi
cd "$PROJECT_ROOT"

echo ""
echo "🎉 Packaging complete!"
echo ""
ls -lh "$RELEASE_DIR"/*.{tar.gz,zip,deb,rpm,apk} 2>/dev/null || true
echo ""
echo "Checksums:"
cat "$RELEASE_DIR/checksums.txt"
