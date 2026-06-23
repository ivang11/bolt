#!/usr/bin/env bash
set -e

REPO="ivang11/bolt"
DEST="/usr/local/bin/bolt"
TMP="$(mktemp)"

echo "Installing bolt..."

OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS:$ARCH" in
  Linux:x86_64 | Linux:amd64)
    ASSET="bolt-linux-x64"
    ;;
  Darwin:x86_64)
    ASSET="bolt-macos-x64"
    ;;
  Darwin:arm64 | Darwin:aarch64)
    ASSET="bolt-macos-arm64"
    ;;
  *)
    echo "Unsupported platform: $OS $ARCH" >&2
    exit 1
    ;;
esac

trap 'rm -f "$TMP"' EXIT

curl -fsSL "https://github.com/$REPO/releases/latest/download/$ASSET" -o "$TMP"
mkdir -p "$(dirname "$DEST")"
install -m755 "$TMP" "$DEST"

echo "bolt installed at $DEST"
