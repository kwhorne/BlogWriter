#!/usr/bin/env bash
# Sign the release artifacts and write dist/latest.json for the auto-updater.
# Run AFTER build-macos.sh and build-linux.sh, BEFORE gh release create.
#
# The private key lives outside the repo (default ~/.config/blogwriter/update_key,
# override with $UPDATE_KEY). The matching public key is embedded in src/updater.rs.
set -euo pipefail
cd "$(dirname "$0")/.."

VERSION=$(grep -m1 '^version' Cargo.toml | cut -d'"' -f2)
KEY="${UPDATE_KEY:-$HOME/.config/blogwriter/update_key}"
REPO="kwhorne/BlogWriter"
BASE="https://github.com/$REPO/releases/download/v$VERSION"

sign() { cargo run --quiet --bin blogwriter-sign -- sign "$KEY" "$1"; }

MAC_ZIP="dist/BlogWriter-$VERSION-macos-arm64.zip"
LINUX_TGZ="dist/BlogWriter-$VERSION-linux-arm64.tar.gz"

[ -f "$MAC_ZIP" ] || { echo "missing $MAC_ZIP — run scripts/build-macos.sh"; exit 1; }

{
  echo "{"
  echo "  \"version\": \"$VERSION\","
  echo "  \"notes\": \"See https://github.com/$REPO/releases/tag/v$VERSION\","
  echo "  \"platforms\": {"
  echo "    \"macos-aarch64\": {"
  echo "      \"url\": \"$BASE/$(basename "$MAC_ZIP")\","
  echo "      \"signature\": \"$(sign "$MAC_ZIP")\""
  if [ -f "$LINUX_TGZ" ]; then
    echo "    },"
    echo "    \"linux-aarch64\": {"
    echo "      \"url\": \"$BASE/$(basename "$LINUX_TGZ")\","
    echo "      \"signature\": \"$(sign "$LINUX_TGZ")\""
  fi
  echo "    }"
  echo "  }"
  echo "}"
} > dist/latest.json

echo "==> dist/latest.json:"
cat dist/latest.json
