#!/usr/bin/env bash
# Build the Linux release in Docker (native arch of the Docker host).
# The private elyra git dependency is satisfied by mounting a local framework
# checkout read-only and patching the git source to that path inside the
# container (no SSH credentials needed in Docker).
set -euo pipefail
cd "$(dirname "$0")/.."

VERSION=$(grep -m1 '^version' Cargo.toml | cut -d'"' -f2)
FRAMEWORK="${ELYRA_FRAMEWORK_DIR:-$HOME/Code/elyra-framework}"
ARCH=$(docker version --format '{{.Server.Arch}}')

echo "==> Building frontend"
(cd app && npm run build)

echo "==> Building Linux ($ARCH) release in Docker"
# The path-patch rewrites Cargo.lock inside the container; keep the host's.
cp Cargo.lock /tmp/blogwriter-Cargo.lock.bak
trap 'mv /tmp/blogwriter-Cargo.lock.bak Cargo.lock' EXIT

docker run --rm \
  -v "$PWD":/work \
  -v "$FRAMEWORK":/framework:ro \
  -e CARGO_TARGET_DIR=/work/target/linux \
  -w /work \
  rust:1-bookworm bash -c '
    set -e
    apt-get update -qq >/dev/null
    apt-get install -y -qq libwebkit2gtk-4.1-dev libgtk-3-dev libxdo-dev \
      libayatana-appindicator3-dev >/dev/null 2>&1
    cargo build --release \
      --config "patch.\"ssh://git@github.com/kwhorne/elyra-framework.git\".elyra.path=\"/framework/framework\""
  '

echo "==> Packaging dist/"
mkdir -p dist
STAGE=$(mktemp -d)
cp target/linux/release/blogwriter LICENSE README.md "$STAGE/"
tar -czf "dist/BlogWriter-$VERSION-linux-$ARCH.tar.gz" -C "$STAGE" .
rm -rf "$STAGE"

echo "==> Done:"
ls -lh "dist/BlogWriter-$VERSION-linux-$ARCH.tar.gz"
echo "Runtime deps on target machines: libwebkit2gtk-4.1, libgtk-3, libayatana-appindicator3"
