#!/usr/bin/env bash
set -euo pipefail

# Package binaries for GitHub release
# Usage: ./scripts/gh-package.sh <version>

VERSION="${1:-}"

if [[ -z "$VERSION" ]]; then
    VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')
fi

echo "Packaging version: $VERSION"

# Create dist directory
DIST_DIR="dist"
rm -rf "$DIST_DIR"
mkdir -p "$DIST_DIR"

# Package macOS app bundle
if [[ -d "target/release/Dockside.app" ]]; then
    echo "Packaging macOS App Bundle..."
    (cd target/release && zip -r "../../$DIST_DIR/Dockside-v$VERSION-macos-arm64.zip" Dockside.app)
fi

echo ""
echo "Packages created in $DIST_DIR/:"
ls -la "$DIST_DIR/"
