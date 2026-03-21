#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

run_step() {
  local label="$1"
  shift

  printf '\n==> %s\n' "${label}"
  "$@"
}

run_step "Build macOS core artifacts" bash "${repo_root}/scripts/build-macos-core.sh"
run_step "Run hc-runtime tests" cargo test -p hc-runtime
run_step "Run hc-ffi tests" cargo test -p hc-ffi
run_step "Run macOS Swift tests" swift test --package-path "${repo_root}/apps/macos"
run_step "Run macOS Swift build" swift build --package-path "${repo_root}/apps/macos"
