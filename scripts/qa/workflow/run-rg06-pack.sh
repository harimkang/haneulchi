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
mkdir -p "${output_dir}/parity" "${output_dir}/notes"

cat >"${output_dir}/parity/state-api.json" <<JSON
{ "mode": "${mode}", "source": "api", "status": "placeholder" }
JSON
cat >"${output_dir}/parity/state-cli.json" <<JSON
{ "mode": "${mode}", "source": "cli", "status": "placeholder" }
JSON
cat >"${output_dir}/notes/rg06-summary.md" <<'EOF'
# RG-06 Snapshot Parity Summary

- compare API, CLI, and UI snapshot semantics
- do not promote RG-06 to pass until parity artifacts are replaced with hosted captures
EOF

python3 - <<'PY' "${output_dir}/gate-results.json" "${mode}"
import json, sys
path, mode = sys.argv[1], sys.argv[2]
with open(path, "r", encoding="utf-8") as fh:
    data = json.load(fh)
data["RG-06"] = mode
with open(path, "w", encoding="utf-8") as fh:
    json.dump(data, fh, indent=2)
    fh.write("\n")
PY

printf 'RG-06 %s prepared\n' "${mode}"
