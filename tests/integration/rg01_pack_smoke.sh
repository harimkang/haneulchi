#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
tmpdir="$(mktemp -d)"
trap 'rm -rf "${tmpdir}"' EXIT

bash "${repo_root}/scripts/qa/readiness/run-rg01-pack.sh" --dry-run --output-dir "${tmpdir}"

test -f "${tmpdir}/manifest.json"
test -f "${tmpdir}/gate-results.json"
test -f "${tmpdir}/environment.json"
test -f "${tmpdir}/scenarios/S-01/runbook.md"
test -f "${tmpdir}/scenarios/S-01/checklist.json"
test -f "${tmpdir}/logs/S-01-first-run.log"
test -f "${tmpdir}/notes/rg01-summary.md"

python3 - <<'PY' "${tmpdir}/scenarios/S-01/checklist.json"
import json
import sys

path = sys.argv[1]
with open(path, "r", encoding="utf-8") as fh:
    data = json.load(fh)

assert data["mode"] == "dry-run", data
assert data["status"] == "dry-run", data
assert data["automated_checks"]["readiness_suite"] == "passed", data
assert data["automated_checks"]["launcher_suite"] == "passed", data
assert data["manual_artifacts_pending"] == [
    "screens/S-01-welcome.png",
    "logs/S-01-first-run.log",
], data
PY

grep -q "RG-01 automated readiness pack" "${tmpdir}/logs/S-01-first-run.log"
grep -q "ReadinessProbeRunnerTests" "${tmpdir}/logs/S-01-first-run.log"
grep -q "AppShellBootstrapTests" "${tmpdir}/logs/S-01-first-run.log"
