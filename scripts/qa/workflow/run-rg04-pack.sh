#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
output_dir="${repo_root}/evidence"
mode="live"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry-run)
      mode="dry-run"
      shift
      ;;
    --output-dir)
      output_dir="${2:-}"
      shift 2
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

bash "${repo_root}/scripts/qa/terminal/write-evidence-manifest.sh" "${output_dir}"

mkdir -p "${output_dir}/scenarios/RG-04" "${output_dir}/notes"

cat >"${output_dir}/scenarios/RG-04/valid-validate.json" <<JSON
{
  "mode": "${mode}",
  "scenario": "valid-validate",
  "state": "ok",
  "workflow_name": "Basic Demo Workflow",
  "requires_operator_review": true
}
JSON

cat >"${output_dir}/scenarios/RG-04/invalid-reload.json" <<JSON
{
  "mode": "${mode}",
  "scenario": "invalid-reload-kept-last-good",
  "state": "invalid_kept_last_good",
  "last_known_good_retained": true,
  "requires_operator_review": true
}
JSON

cat >"${output_dir}/notes/rg04-summary.md" <<EOF
# RG-04 Summary

- Mode: \`${mode}\`
- Scenario set: valid validate / invalid reload kept last good
- Validation surface: Haneulchi \`Workflow Contract / Runbook Drawer\`
- Status: \`incomplete\`

Use \`notes/RG-04-workflow-checklist.md\` and operator notes to complete hosted validation.
EOF

python3 - <<'PY' "${output_dir}/gate-results.json" "${mode}"
import json
import sys

path, mode = sys.argv[1], sys.argv[2]
with open(path, "r", encoding="utf-8") as fh:
    data = json.load(fh)

data["RG-04"] = mode

with open(path, "w", encoding="utf-8") as fh:
    json.dump(data, fh, indent=2)
    fh.write("\n")
PY

printf 'RG-04 %s prepared\n' "${mode}"
