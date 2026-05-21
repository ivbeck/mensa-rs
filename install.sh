#!/bin/bash
set -e

OWNER="ivbeck"
REPO="mensa-rs"

LATEST_URL="https://api.github.com/repos/${OWNER}/${REPO}/releases/latest"

download() {
    local url="$1"
    local dest="$2"
    echo "Downloading $url..."
    curl -sSL "$url" -o "$dest"
}

get_asset_url() {
    local pattern="$1"
    curl -sSL "$LATEST_URL" | grep -o "\"${pattern}\":\s*\"[^\"]*\"" | sed 's/.*"browser_download_url":\s*"\([^"]*\)"/\1/'
}

ARCH=$(uname -m)
case "$ARCH" in
    x86_64) ARCH_STR="x86_64-unknown-linux-gnu" ;;
    aarch64) ARCH_STR="aarch64-unknown-linux-gnu" ;;
    *) echo "Unsupported architecture: $ARCH" >&2; exit 1 ;;
esac

ASSET_NAME="mensa-${ARCH_STR}.tar.gz"
TARBALL="/tmp/mensa.tar.gz"
EXTRACTED="/tmp/mensa"

echo "Installing mensa for ${ARCH_STR}..."

URL=$(get_asset_url "$ASSET_NAME")
if [ -z "$URL" ]; then
    echo "No release found. Building from source with cargo..."
    cargo install --git "https://github.com/${OWNER}/${REPO}.git"
    exit 0
fi

download "$URL" "$TARBALL"

mkdir -p "$EXTRACTED"
tar xzf "$TARBALL" -C "$EXTRACTED"

INSTALL_DIR="${HOME}/.local/bin"
mkdir -p "$INSTALL_DIR"
mv "${EXTRACTED}/mensa" "${INSTALL_DIR}/mensa"

rm -rf "$TARBALL" "$EXTRACTED"

echo "Installed mensa to ${INSTALL_DIR}/mensa"