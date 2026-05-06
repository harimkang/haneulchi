#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
PACKAGE_JSON="$ROOT_DIR/package.json"
TEMPLATE_PATH="${TEMPLATE_PATH:-$ROOT_DIR/packaging/homebrew/haneulchi.rb.template}"
OUTPUT_PATH="${OUTPUT_PATH:-$ROOT_DIR/dist/release/homebrew/haneulchi.rb}"
DMG_PATH="${DMG_PATH:-}"
DMG_URL="${DMG_URL:-}"
HOMEPAGE_URL="${HOMEPAGE_URL:-https://github.com/haneulchi/haneulchi}"
HOMEBREW_TAP_REPOSITORY="${HOMEBREW_TAP_REPOSITORY:-}"

fail() {
  echo "error: $*" >&2
  exit 1
}

if [[ -z "$DMG_PATH" ]]; then
  DMG_PATH="$(find "$ROOT_DIR/src-tauri/target/release/bundle/dmg" -maxdepth 1 -type f -name "*.dmg" 2>/dev/null | sort | head -n 1 || true)"
fi

[[ -f "$DMG_PATH" ]] || fail "DMG_PATH must point to a built DMG"
[[ -n "$DMG_URL" ]] || fail "DMG_URL is required for Homebrew cask publishing"
[[ -n "$HOMEBREW_TAP_REPOSITORY" ]] || fail "HOMEBREW_TAP_REPOSITORY is required for cask publishing metadata"

VERSION="$(node -e 'console.log(JSON.parse(require("fs").readFileSync(process.argv[1], "utf8")).version)' "$PACKAGE_JSON")"
# Homebrew casks require a sha256 checksum for the published DMG.
DMG_SHA256="$(shasum -a 256 "$DMG_PATH" | awk '{print $1}')"

mkdir -p "$(dirname "$OUTPUT_PATH")"
sed \
  -e "s|__VERSION__|$VERSION|g" \
  -e "s|__DMG_SHA256__|$DMG_SHA256|g" \
  -e "s|__DMG_URL__|$DMG_URL|g" \
  -e "s|__HOMEPAGE_URL__|$HOMEPAGE_URL|g" \
  "$TEMPLATE_PATH" > "$OUTPUT_PATH"

echo "Rendered Homebrew cask for $HOMEBREW_TAP_REPOSITORY: $OUTPUT_PATH"
