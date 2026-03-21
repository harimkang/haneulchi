#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

swift test --package-path "${repo_root}/apps/macos"
swift build --package-path "${repo_root}/apps/macos"
bash "${repo_root}/scripts/qa/readiness/run-rg01-pack.sh" --dry-run
