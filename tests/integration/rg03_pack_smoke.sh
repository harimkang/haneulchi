#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
tmpdir="$(mktemp -d)"
trap 'rm -rf "${tmpdir}"' EXIT

bash "${repo_root}/scripts/qa/terminal/write-evidence-manifest.sh" "${tmpdir}"
test -f "${tmpdir}/manifest.json"
test -f "${tmpdir}/gate-results.json"
test -f "${tmpdir}/environment.json"

bash "${repo_root}/scripts/qa/terminal/run-rg03-pack.sh" \
  --dry-run \
  --tools "vim,tmux,lazygit" \
  --output-dir "${tmpdir}"
test -f "${tmpdir}/notes/rg03-summary.md"
test -f "${tmpdir}/notes/rg03-runbook.md"
test -f "${tmpdir}/notes/RG-03-template-checklist.md"
test -f "${tmpdir}/scenarios/RG-03/results.json"
grep -q "Selected tools: vim tmux lazygit" "${tmpdir}/notes/rg03-runbook.md"
grep -q "Screen capture path" "${tmpdir}/notes/RG-03-template-checklist.md"
