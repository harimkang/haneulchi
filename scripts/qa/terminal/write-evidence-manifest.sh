#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
output_dir="${1:-${repo_root}/evidence}"

mkdir -p \
  "${output_dir}/scenarios/S-02" \
  "${output_dir}/scenarios/RG-03" \
  "${output_dir}/metrics" \
  "${output_dir}/notes" \
  "${output_dir}/logs" \
  "${output_dir}/screens"

timestamp="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
build_id="$(date -u +"%Y.%m.%d-%H%M%S")"
commit="$(git -C "${repo_root}" rev-parse HEAD 2>/dev/null || echo unknown)"
operator_name="${USER:-unknown}"
shell_name="${SHELL:-unknown}"
macos_version="$(sw_vers -productVersion 2>/dev/null || echo unknown)"
device_model="$(sysctl -n hw.model 2>/dev/null || echo unknown)"
cpu_brand="$(sysctl -n machdep.cpu.brand_string 2>/dev/null || echo unknown)"
ram_bytes="$(sysctl -n hw.memsize 2>/dev/null || echo 0)"

cat >"${output_dir}/manifest.json" <<JSON
{
  "product": "Haneulchi",
  "release_target": "MVP",
  "build_id": "${build_id}",
  "commit": "${commit}",
  "api_version": "1",
  "workflow_contract_version": "1",
  "snapshot_schema_version": "1",
  "operator": "${operator_name}",
  "executed_at": "${timestamp}"
}
JSON

cat >"${output_dir}/gate-results.json" <<'JSON'
{
  "RG-02": "not_run",
  "RG-03": "not_run"
}
JSON

cat >"${output_dir}/environment.json" <<JSON
{
  "macos_version": "${macos_version}",
  "device_model": "${device_model}",
  "cpu": "${cpu_brand}",
  "ram_bytes": ${ram_bytes},
  "shell": "${shell_name}"
}
JSON
