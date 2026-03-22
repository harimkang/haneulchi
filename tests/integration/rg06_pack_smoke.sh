#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
tmpdir="$(mktemp -d)"
trap 'rm -rf "${tmpdir}"' EXIT

bash "${repo_root}/scripts/qa/workflow/run-rg06-pack.sh" --dry-run --output-dir "${tmpdir}"

test -f "${tmpdir}/parity/state-api.json"
test -f "${tmpdir}/parity/state-cli.json"
test -f "${tmpdir}/notes/rg06-summary.md"
grep -q "parity" "${tmpdir}/notes/rg06-summary.md"
