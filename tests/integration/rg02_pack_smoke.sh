#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
tmpdir="$(mktemp -d)"
trap 'rm -rf "${tmpdir}"' EXIT

bash "${repo_root}/scripts/qa/terminal/run-rg02-pack.sh" --dry-run --output-dir "${tmpdir}"

test -f "${tmpdir}/manifest.json"
test -f "${tmpdir}/gate-results.json"
test -f "${tmpdir}/environment.json"
test -f "${tmpdir}/metrics/terminal-latency.json"
test -f "${tmpdir}/scenarios/S-02/checklist.json"

python3 - <<'PY' "${tmpdir}/metrics/terminal-latency.json" "${tmpdir}/scenarios/S-02/checklist.json" "${tmpdir}/gate-results.json"
import json
import sys

metrics_path, checklist_path, gate_results_path = sys.argv[1:4]

with open(metrics_path, "r", encoding="utf-8") as fh:
    metrics = json.load(fh)

assert metrics["status"] == "dry-run", metrics
assert isinstance(metrics["typing_proxy_p95_ms"], (int, float)), metrics
assert isinstance(metrics["typing_proxy_p99_ms"], (int, float)), metrics
assert metrics["collection"] == "non_ui_proxy", metrics

with open(checklist_path, "r", encoding="utf-8") as fh:
    checklist = json.load(fh)

assert checklist["status"] == "dry-run", checklist
assert checklist["automated_checks"]["runtime_suite"] == "passed", checklist
assert checklist["automated_checks"]["renderer_suite"] == "passed", checklist
assert checklist["manual_artifacts_pending"] == [
    "scenarios/S-02/terminal-proof.mp4",
    "notes/RG-02-terminal-checklist.md",
], checklist

with open(gate_results_path, "r", encoding="utf-8") as fh:
    gate_results = json.load(fh)

assert gate_results["RG-02"] == "dry-run", gate_results
PY
