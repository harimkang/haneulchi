#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DMG_PATH="${DMG_PATH:-}"

fail() {
  echo "error: $*" >&2
  exit 1
}

require_env() {
  if [[ -z "${!1:-}" ]]; then
    fail "$1 is required for Apple notarization"
  fi
}

if [[ -z "$DMG_PATH" ]]; then
  DMG_PATH="$(find "$ROOT_DIR/src-tauri/target/release/bundle/dmg" -maxdepth 1 -type f -name "*.dmg" 2>/dev/null | sort | head -n 1 || true)"
fi

if [[ -z "$DMG_PATH" || ! -f "$DMG_PATH" ]]; then
  fail "DMG not found; run npm run release:macos:dmg first or set DMG_PATH"
fi

require_env APPLE_ID
require_env APPLE_PASSWORD
require_env APPLE_TEAM_ID

command -v xcrun >/dev/null 2>&1 || fail "xcrun is required for notarization"

echo "Submitting DMG for notarization: $DMG_PATH"
xcrun notarytool submit "$DMG_PATH" \
  --apple-id "$APPLE_ID" \
  --password "$APPLE_PASSWORD" \
  --team-id "$APPLE_TEAM_ID" \
  --wait

echo "Stapling notarization ticket: $DMG_PATH"
xcrun stapler staple "$DMG_PATH"

echo "Validating stapled notarization ticket: $DMG_PATH"
xcrun stapler validate "$DMG_PATH"
