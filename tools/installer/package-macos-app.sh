#!/bin/bash
set -euo pipefail

version=$1
dist=$2
shift 2
assets="$(dirname "$0")/assets"
[[ "$("$dist/hibi" --version)" == "hibi v$version (Config Installer)" ]] || {
    echo "The macOS installer binary does not match version $version" >&2
    exit 1
}
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
app="$work/Hibi.app"
contents="$app/Contents"
resources="$contents/Resources"
mkdir -p "$contents/MacOS" "$resources" "$work/Hibi.iconset"
cp "$dist/hibi" "$contents/MacOS/hibi"
for item in "$@"; do
    cp -R "$dist/$item" "$resources/"
done
printf '%s\n' "$version" > "$resources/.hibi-bundle"

for size in 16 32 128 256 512; do
    for scale in 1 2; do
        pixels=$((size * scale))
        suffix=""
        if [[ "$scale" == 2 ]]; then suffix="@2x"; fi
        sips -z "$pixels" "$pixels" "$assets/hibi.png" \
            --out "$work/Hibi.iconset/icon_${size}x${size}${suffix}.png" >/dev/null
    done
done
iconutil -c icns "$work/Hibi.iconset" -o "$resources/Hibi.icns"
short_version=${version%%-*}
build_version=$short_version
if [[ "$version" =~ -beta\.([1-9][0-9]*)$ ]]; then
    build_version="${short_version}b${BASH_REMATCH[1]}"
fi
cat > "$contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
    <key>CFBundleIdentifier</key><string>com.github.devsepnine.hibi</string>
    <key>CFBundlePackageType</key><string>APPL</string>
    <key>CFBundleName</key><string>Hibi</string>
    <key>CFBundleDisplayName</key><string>Hibi</string>
    <key>CFBundleExecutable</key><string>hibi</string>
    <key>CFBundleIconFile</key><string>Hibi.icns</string>
    <key>CFBundleShortVersionString</key><string>$short_version</string>
    <key>CFBundleVersion</key><string>$build_version</string>
</dict></plist>
EOF
codesign --force --sign - "$app"
codesign --verify --strict "$app"
(cd "$work" && zip -q -r Hibi.zip Hibi.app)
mkdir "$work/image"
mv "$app" "$work/image/"
ln -s /Applications "$work/image/Applications"
hdiutil create -volname Hibi -fs HFS+ -srcfolder "$work/image" -format UDZO "$work/Hibi.dmg"
mv "$work/Hibi.zip" "$dist/Hibi-$version-macos.zip"
mv "$work/Hibi.dmg" "$dist/Hibi-$version-macos.dmg"
