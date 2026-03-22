#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
tmpdir="$(mktemp -d)"
trap 'rm -rf "${tmpdir}"' EXIT

bash "${repo_root}/scripts/qa/workflow/run-rg04-pack.sh" --dry-run --output-dir "${tmpdir}"

test -f "${tmpdir}/scenarios/RG-04/valid-validate.json"
test -f "${tmpdir}/scenarios/RG-04/invalid-reload.json"
test -f "${tmpdir}/scenarios/S-05/runbook.md"
test -f "${tmpdir}/notes/rg04-summary.md"
test -f "${tmpdir}/notes/RG-04-workflow-checklist.md"
test -f "${tmpdir}/notes/rg04-valid-operator-note.md"
test -f "${tmpdir}/notes/rg04-invalid-operator-note.md"

grep -q "last-known-good" "${tmpdir}/scenarios/S-05/runbook.md"
grep -q "valid path" "${tmpdir}/notes/rg04-valid-operator-note.md"
