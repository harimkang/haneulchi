#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
tmpdir="$(mktemp -d)"
trap 'rm -rf "${tmpdir}"' EXIT

bash "${repo_root}/scripts/qa/workflow/run-rg07-pack.sh" --dry-run --output-dir "${tmpdir}"

test -f "${tmpdir}/scenarios/RG-07/ops-summary.json"
test -f "${tmpdir}/notes/RG-07-attention.md"
grep -q "ops strip" "${tmpdir}/notes/RG-07-attention.md"
