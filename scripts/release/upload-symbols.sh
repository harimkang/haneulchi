#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SYMBOLS_PATH="${SYMBOLS_PATH:-$ROOT_DIR/src-tauri/target/release/bundle/macos}"
ALLOW_MISSING_SYMBOLS="${ALLOW_MISSING_SYMBOLS:-0}"

fail() {
  echo "error: $*" >&2
  exit 1
}

require_env() {
  if [[ -z "${!1:-}" ]]; then
    fail "$1 is required for crash symbol upload"
  fi
}

require_env SENTRY_AUTH_TOKEN
require_env SENTRY_ORG
require_env SENTRY_PROJECT

if ! find "$SYMBOLS_PATH" -name "*.dSYM" -o -name "*.app.dSYM" | grep -q .; then
  if [[ "$ALLOW_MISSING_SYMBOLS" == "1" ]]; then
    echo "warning: no dSYM bundles found under $SYMBOLS_PATH"
    exit 0
  fi
  fail "no dSYM bundles found under $SYMBOLS_PATH"
fi

command -v sentry-cli >/dev/null 2>&1 || fail "sentry-cli is required for symbol upload"

sentry-cli debug-files upload \
  --org "$SENTRY_ORG" \
  --project "$SENTRY_PROJECT" \
  "$SYMBOLS_PATH"
