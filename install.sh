#!/usr/bin/env bash
set -euo pipefail

PREFIX="${PREFIX:-$HOME/.local/bin}"
VERSION=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --prefix) PREFIX="$2"; shift 2 ;;
    --version) VERSION="$2"; shift 2 ;;
    *) echo "Unknown option: $1"; exit 1 ;;
  esac
done

OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$OS-$ARCH" in
  linux-x86_64)    TARGET="x86_64-unknown-linux-musl" ;;
  darwin-arm64)    TARGET="aarch64-apple-darwin" ;;
  darwin-x86_64)   TARGET="x86_64-apple-darwin" ;;
  *) echo "Unsupported platform: $OS $ARCH"; exit 1 ;;
esac

if [ -z "$VERSION" ]; then
  VERSION=$(curl -fsSL https://api.github.com/repos/just-sultanov/xfeat/releases/latest \
    | grep '"tag_name"' \
    | sed 's/.*"v\([^"]*\)".*/\1/')
fi

URL="https://github.com/just-sultanov/xfeat/releases/download/v${VERSION}/xfeat-${TARGET}.tar.gz"

TMP=$(mktemp -d)
curl -fsSL "$URL" -o "$TMP/xfeat.tar.gz"
tar -xzf "$TMP/xfeat.tar.gz" -C "$TMP"

mkdir -p "$PREFIX"
mv "$TMP/xfeat" "$PREFIX/xfeat"
rm -rf "$TMP"

echo "xfeat v${VERSION} installed to ${PREFIX}/xfeat"
