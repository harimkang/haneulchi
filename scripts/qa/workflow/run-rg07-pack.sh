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
mkdir -p "${output_dir}/scenarios/RG-07" "${output_dir}/notes"

cat >"${output_dir}/scenarios/RG-07/ops-summary.json" <<JSON
{
  "mode": "${mode}",
  "scenario": "ops-summary",
  "status": "incomplete",
  "requires_operator_attention": true
}
JSON

cat >"${output_dir}/notes/RG-07-attention.md" <<'EOF'
# RG-07 Ops Attention Note

- capture one ops strip screenshot
- record one stale target / blocked dispatch note
- do not mark RG-07 pass until attention artifacts are attached
EOF

python3 - <<'PY' "${output_dir}/gate-results.json" "${mode}"
import json, sys
path, mode = sys.argv[1], sys.argv[2]
with open(path, "r", encoding="utf-8") as fh:
    data = json.load(fh)
data["RG-07"] = mode
with open(path, "w", encoding="utf-8") as fh:
    json.dump(data, fh, indent=2)
    fh.write("\n")
PY

printf 'RG-07 %s prepared\n' "${mode}"
