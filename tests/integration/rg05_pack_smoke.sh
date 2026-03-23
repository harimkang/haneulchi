#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
tmpdir="$(mktemp -d)"
trap 'rm -rf "${tmpdir}"' EXIT

bash "${repo_root}/scripts/qa/workflow/run-rg05-pack.sh" --dry-run --output-dir "${tmpdir}"

test -f "${tmpdir}/scenarios/RG-05/review-flow.json"
test -f "${tmpdir}/notes/RG-05-review-checklist.md"
grep -q "review-ready task" "${tmpdir}/notes/RG-05-review-checklist.md"
