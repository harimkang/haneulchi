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

bash "${repo_root}/scripts/qa/workflow/run-rg07-pack.sh" --dry-run --output-dir "${output_dir}"
mkdir -p "${output_dir}/scenarios/S-05"
touch "${output_dir}/scenarios/S-05/ops-strip.mp4"
printf 'RG-07 %s control pack prepared\n' "${mode}"
