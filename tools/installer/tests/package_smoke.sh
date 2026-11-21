#!/bin/bash
set -euo pipefail

packages=$(realpath "$1")
version=$2
engine=${HIBI_CONTAINER_ENGINE:-docker}
previous=$(mktemp -d)
chmod 755 "$previous"
trap 'rm -rf "$previous"' EXIT

download() {
    local tag=$1 asset=$2 type=$3 hash=$4
    curl -fsSL --retry 3 "https://github.com/devsepnine/hibi_ai/releases/download/$tag/$asset" -o "$previous/hibi.$type"
    printf '%s  %s\n' "$hash" "$previous/hibi.$type" | sha256sum -c -
}

download v1.23.1 hibi-ai-1.23.1-1.x86_64.rpm rpm bf3c283243b26938ac624a60b1e6e1012df643ead582cf0e63df13f4e9cb0590
download v1.25.0 hibi-ai_1.25.0_amd64.deb deb 3e22c616a493dd498c5fc4b56ef82d9d18060342f189f2c40b1db19b664c042b
download v1.25.0 hibi-ai_1.25.0_x86_64.apk apk a43be989dc9bf19dd1ce2a73ce861f1b22dd39f3e36c09ecfeef70bcdcdaedb4

for image in registry.fedoraproject.org/fedora:45 docker.io/library/almalinux:9 docker.io/library/debian:13-slim docker.io/library/alpine:3.22; do
    "$engine" run --rm -v "$packages:/packages:ro,z" -v "$previous:/previous:ro,z" "$image" sh -ec '
        version=$1
        if command -v dnf >/dev/null; then
            dnf -y install /packages/*.rpm
            test "$(hibi --version)" = "hibi v$version (Config Installer)"
            dnf -y remove hibi-ai
            dnf -y install /previous/hibi.rpm
            test "$(hibi --version)" = "hibi v1.23.1 (Config Installer)"
            dnf -y upgrade /packages/*.rpm
        elif command -v apt-get >/dev/null; then
            export DEBIAN_FRONTEND=noninteractive
            apt-get update -qq
            apt-get install -y /packages/*.deb
            test "$(hibi --version)" = "hibi v$version (Config Installer)"
            apt-get remove -y hibi-ai
            apt-get install -y /previous/hibi.deb
            test "$(hibi --version)" = "hibi v1.25.0 (Config Installer)"
            apt-get install -y /packages/*.deb
        else
            apk --no-cache add --allow-untrusted /packages/*.apk
            test "$(hibi --version)" = "hibi v$version (Config Installer)"
            apk del hibi-ai
            apk --no-cache add --allow-untrusted /previous/hibi.apk
            test "$(hibi --version)" = "hibi v1.25.0 (Config Installer)"
            apk --no-cache add --allow-untrusted /packages/*.apk
        fi
        hibi --version
        test "$(hibi --version)" = "hibi v$version (Config Installer)"
        test -f /usr/share/hibi/settings.json
    ' package-smoke "$version"
done

msiinfo export "$packages/"*.msi InstallExecuteSequence | awk -F '\t' '
    $1 == "InstallInitialize" { initialize = $3 + 0 }
    $1 == "RemoveExistingProducts" { remove = $3 + 0 }
    $1 == "ProcessComponents" { process = $3 + 0 }
    END { if (!initialize || remove != initialize + 1 || remove >= process) exit 1 }
' || { echo 'MSI must remove the previous version inside the upgrade transaction' >&2; exit 1; }
