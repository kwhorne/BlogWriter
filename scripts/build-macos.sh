#!/usr/bin/env bash
# Build, bundle, sign, and package the macOS (arm64) release.
# Requires: rata (Elyra CLI), a "Developer ID Application" identity in the keychain.
set -euo pipefail
cd "$(dirname "$0")/.."

VERSION=$(grep -m1 '^version' Cargo.toml | cut -d'"' -f2)
IDENTITY="${CODESIGN_IDENTITY:-Developer ID Application: GETS AS (7G383N3VY7)}"
APP=target/release/bundle/BlogWriter.app

echo "==> Building frontend"
(cd app && npm run build)

echo "==> Bundling BlogWriter.app"
rata bundle

echo "==> Signing with: $IDENTITY"
codesign --force --deep --options runtime --timestamp -s "$IDENTITY" "$APP"
codesign --verify --deep --strict "$APP"

echo "==> Packaging dist/"
mkdir -p dist
STAGE=$(mktemp -d)
cp -R "$APP" "$STAGE/"
ln -s /Applications "$STAGE/Applications"
hdiutil create -volname "BlogWriter $VERSION" -srcfolder "$STAGE" -ov -quiet \
  -format UDZO "dist/BlogWriter-$VERSION-macos-arm64.dmg"
rm -rf "$STAGE"
codesign --force --timestamp -s "$IDENTITY" "dist/BlogWriter-$VERSION-macos-arm64.dmg"
ditto -c -k --keepParent "$APP" "dist/BlogWriter-$VERSION-macos-arm64.zip"

echo "==> Notarizing (profile: ${NOTARY_PROFILE:-AC_PASSWORD})"
xcrun notarytool submit "dist/BlogWriter-$VERSION-macos-arm64.dmg" \
  --keychain-profile "${NOTARY_PROFILE:-AC_PASSWORD}" --wait
xcrun stapler staple "dist/BlogWriter-$VERSION-macos-arm64.dmg"
xcrun stapler staple "$APP"
# Re-zip so the zipped .app carries the stapled ticket too.
ditto -c -k --keepParent "$APP" "dist/BlogWriter-$VERSION-macos-arm64.zip"
spctl --assess --type open --context context:primary-signature -v "dist/BlogWriter-$VERSION-macos-arm64.dmg"

echo "==> Done:"
ls -lh "dist/BlogWriter-$VERSION-macos-arm64".{dmg,zip}
