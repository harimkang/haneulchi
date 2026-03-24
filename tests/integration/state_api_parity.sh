#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
tmpdir="$(mktemp -d)"
trap 'rm -rf "${tmpdir}"' EXIT

bash "${repo_root}/scripts/qa/control/run-rg06-pack.sh" --dry-run --output-dir "${tmpdir}"

test -f "${tmpdir}/parity/state-api.json"
test -f "${tmpdir}/parity/state-cli.json"
test -f "${tmpdir}/parity/sessions-api.json"
test -f "${tmpdir}/parity/sessions-cli.json"
test -f "${tmpdir}/parity/review-item.json"
test -f "${tmpdir}/parity/diff-report.md"
grep -q '"task_id"' "${tmpdir}/parity/review-item.json"
