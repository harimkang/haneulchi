#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
output_dir="${repo_root}/evidence"
mode="dry-run"

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
mkdir -p "${output_dir}/scenarios/RG-05" "${output_dir}/notes"

cat >"${output_dir}/scenarios/RG-05/review-flow.json" <<JSON
{
  "mode": "${mode}",
  "scenario": "review-flow",
  "status": "incomplete",
  "requires_operator_review": true
}
JSON

cat >"${output_dir}/notes/RG-05-review-checklist.md" <<'EOF'
# RG-05 Review Checklist

1. Confirm a review-ready task appears in Review Queue.
2. Capture evidence summary, touched files, tests, and warnings.
3. Record one positive-path screenshot.
4. Record one negative-path note for broken-link or degraded evidence.
5. Do not mark RG-05 as pass until operator notes and screenshots exist.
EOF

python3 - <<'PY' "${output_dir}/gate-results.json" "${mode}"
import json, sys
path, mode = sys.argv[1], sys.argv[2]
with open(path, "r", encoding="utf-8") as fh:
    data = json.load(fh)
data["RG-05"] = mode
with open(path, "w", encoding="utf-8") as fh:
    json.dump(data, fh, indent=2)
    fh.write("\n")
PY

printf 'RG-05 %s prepared\n' "${mode}"
