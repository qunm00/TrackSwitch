#!/bin/sh
# Install TrackSwitch from a GitHub release on macOS.
#
# Use: sh install.sh [tag]
#
# The script downloads the release zip with curl. Files that curl downloads
# do not get the macOS quarantine flag, so Gatekeeper does not block the app
# on first launch. The app is still not signed by Apple, so the script adds a
# local ad-hoc signature. That signature is enough for Apple Silicon to run it.
set -eu

REPO="qunm00/TrackSwitch"
APP_NAME="TrackSwitch.app"
ASSET="TrackSwitch.app.zip"
# Where the app goes. The default needs no admin password when the app updates
# itself. Set INSTALL_DIR to use another folder, for example INSTALL_DIR=/Applications.
# The login item follows the app path, so both folders work.
INSTALL_DIR="${INSTALL_DIR:-$HOME/Applications}"

say() {
  printf '%s\n' "$1"
}

die() {
  printf 'install: %s\n' "$1" >&2
  exit 1
}

if [ "$(uname -s)" != "Darwin" ]; then
  die "this script runs on macOS only"
fi

command -v curl >/dev/null 2>&1 || die "curl is missing"
command -v unzip >/dev/null 2>&1 || die "unzip is missing"
command -v codesign >/dev/null 2>&1 || die "codesign is missing"

TAG="${1:-}"
if [ -z "$TAG" ]; then
  say "Finding the latest release"
  TAG=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" \
    | sed -n 's/.*"tag_name": *"\([^"]*\)".*/\1/p' \
    | head -n 1)
  [ -n "$TAG" ] || die "cannot read the latest release tag. Pass a tag as the first argument."
fi
say "Installing $TAG"

BASE="https://github.com/$REPO/releases/download/$TAG"
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT INT TERM

say "Downloading $ASSET"
curl -fsSL -o "$TMP/$ASSET" "$BASE/$ASSET" || die "download failed: $BASE/$ASSET"

if curl -fsSL -o "$TMP/$ASSET.sha256" "$BASE/$ASSET.sha256" 2>/dev/null; then
  say "Checking the checksum"
  ( cd "$TMP" && shasum -a 256 -c "$ASSET.sha256" ) \
    || die "the checksum does not match. Do not install this file."
else
  say "The release has no checksum file. Skipping the check."
fi

say "Unpacking"
( cd "$TMP" && unzip -q "$ASSET" )
[ -d "$TMP/$APP_NAME" ] || die "the zip does not contain $APP_NAME"

TARGET_DIR="$INSTALL_DIR"
if [ ! -d "$TARGET_DIR" ]; then
  mkdir -p "$TARGET_DIR" || die "cannot create $TARGET_DIR"
fi
[ -w "$TARGET_DIR" ] \
  || die "cannot write to $TARGET_DIR. Choose another folder with INSTALL_DIR."

DEST="$TARGET_DIR/$APP_NAME"

if [ -d "$DEST" ]; then
  say "Removing the old copy"
  launchctl bootout "gui/$(id -u)/com.qunm00.trackswitch" 2>/dev/null || true
  pkill -f "$DEST/Contents/MacOS" 2>/dev/null || true
  rm -rf "$DEST"
fi

say "Copying to $DEST"
cp -R "$TMP/$APP_NAME" "$DEST"

say "Signing the app locally"
codesign --force --deep --sign - "$DEST" >/dev/null 2>&1 || die "codesign failed"
codesign --verify --strict "$DEST" || die "the local signature does not verify"
xattr -dr com.apple.quarantine "$DEST" 2>/dev/null || true

EXEC_NAME=$(/usr/libexec/PlistBuddy -c "Print :CFBundleExecutable" "$DEST/Contents/Info.plist" 2>/dev/null \
  || basename "$APP_NAME" .app)
EXEC="$DEST/Contents/MacOS/$EXEC_NAME"
if [ ! -x "$EXEC" ]; then
  say "Warning: the bundle has no executable at $EXEC."
fi

command -v blueutil >/dev/null 2>&1 \
  || say "Warning: blueutil is missing. Run: brew install blueutil"

say "Done. Open TrackSwitch from $TARGET_DIR."
