#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
tmpdir="$(mktemp -d)"
trap 'rm -rf "${tmpdir}"' EXIT

bash "${repo_root}/scripts/qa/control/run-rg09-pack.sh" --dry-run --output-dir "${tmpdir}"

test -f "${tmpdir}/metrics/state-snapshot.json"
test -f "${tmpdir}/notes/RG-09-security-checklist.md"
test -f "${tmpdir}/logs/attention-events.jsonl"
