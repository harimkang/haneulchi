#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
APP_BUNDLE_PATH="${APP_BUNDLE_PATH:-$ROOT_DIR/src-tauri/target/release/bundle/macos/Haneulchi.app}"
DMG_PATH="${DMG_PATH:-}"
ALLOW_UNSIGNED_APP="${ALLOW_UNSIGNED_APP:-0}"
ALLOW_MISSING_DMG="${ALLOW_MISSING_DMG:-0}"
SKIP_GATEKEEPER="${SKIP_GATEKEEPER:-0}"

fail() {
  echo "error: $*" >&2
  exit 1
}

need_command() {
  command -v "$1" >/dev/null 2>&1 || fail "$1 is required on macOS release verification hosts"
}

verify_app_signature() {
  local app_path="$1"

  echo "Verifying app code signature: $app_path"
  if ! codesign --verify --deep --strict --verbose=2 "$app_path"; then
    if [[ "$ALLOW_UNSIGNED_APP" == "1" ]]; then
      echo "warning: app signature failed, continuing because ALLOW_UNSIGNED_APP=1"
    else
      fail "codesign verification failed; set ALLOW_UNSIGNED_APP=1 only for local unsigned smoke checks"
    fi
  fi

  if [[ "$SKIP_GATEKEEPER" != "1" ]]; then
    echo "Assessing app with Gatekeeper: $app_path"
    if ! spctl --assess --type execute --verbose "$app_path"; then
      if [[ "$ALLOW_UNSIGNED_APP" == "1" ]]; then
        echo "warning: Gatekeeper app assessment failed, continuing because ALLOW_UNSIGNED_APP=1"
      else
        fail "Gatekeeper app assessment failed"
      fi
    fi
  fi
}

need_command codesign
need_command spctl
need_command hdiutil

if [[ -z "$DMG_PATH" ]]; then
  DMG_PATH="$(find "$ROOT_DIR/src-tauri/target/release/bundle/dmg" -maxdepth 1 -type f -name "*.dmg" 2>/dev/null | sort | head -n 1 || true)"
fi

STANDALONE_APP_VERIFIED=0
if [[ -d "$APP_BUNDLE_PATH" ]]; then
  verify_app_signature "$APP_BUNDLE_PATH"
  STANDALONE_APP_VERIFIED=1
elif [[ -z "$DMG_PATH" || ! -f "$DMG_PATH" ]]; then
  fail "app bundle not found: $APP_BUNDLE_PATH"
else
  echo "Standalone app bundle not found; verifying app from mounted DMG instead"
fi

if [[ -z "$DMG_PATH" || ! -f "$DMG_PATH" ]]; then
  if [[ "$ALLOW_MISSING_DMG" == "1" ]]; then
    echo "warning: DMG not found, continuing because ALLOW_MISSING_DMG=1"
    exit 0
  fi
  fail "DMG not found; run npm run release:macos:dmg before release verification"
fi

echo "Mounting DMG read-only: $DMG_PATH"
ATTACH_OUTPUT="$(hdiutil attach -nobrowse -readonly "$DMG_PATH")"
MOUNT_POINT="$(printf "%s\n" "$ATTACH_OUTPUT" | awk '/\/Volumes\// {print substr($0, index($0, "/Volumes/")); exit}')"
if [[ -z "$MOUNT_POINT" ]]; then
  fail "unable to determine DMG mount point"
fi

cleanup() {
  hdiutil detach "$MOUNT_POINT" >/dev/null 2>&1 || true
}
trap cleanup EXIT

if [[ ! -d "$MOUNT_POINT/Haneulchi.app" ]]; then
  fail "mounted DMG does not contain Haneulchi.app"
fi
MOUNTED_APP_PATH="$MOUNT_POINT/Haneulchi.app"

if [[ "$STANDALONE_APP_VERIFIED" != "1" ]]; then
  verify_app_signature "$MOUNTED_APP_PATH"
fi

echo "Assessing DMG primary signature: $DMG_PATH"
if ! spctl --assess --type open --context context:primary-signature --verbose "$DMG_PATH"; then
  if [[ "$ALLOW_UNSIGNED_APP" == "1" ]]; then
    echo "warning: DMG signature assessment failed, continuing because ALLOW_UNSIGNED_APP=1"
  else
    fail "DMG signature assessment failed"
  fi
fi

if command -v xcrun >/dev/null 2>&1; then
  echo "Validating notarization staple: $DMG_PATH"
  if ! xcrun stapler validate "$DMG_PATH"; then
    if [[ "$ALLOW_UNSIGNED_APP" == "1" ]]; then
      echo "warning: notarization staple validation failed, continuing because ALLOW_UNSIGNED_APP=1"
    else
      fail "notarization staple validation failed"
    fi
  fi
fi

echo "macOS release artifacts verified"
