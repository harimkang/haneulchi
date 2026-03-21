#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
output_dir="${repo_root}/evidence"
mode="live"
tools_csv=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry-run)
      mode="dry-run"
      shift
      ;;
    --tools)
      tools_csv="${2:-}"
      shift 2
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

supported_tools=(yazi lazygit nvim vim tmux)
selected_tools=()

is_supported_tool() {
  local tool="$1"
  for candidate in "${supported_tools[@]}"; do
    if [[ "${candidate}" == "${tool}" ]]; then
      return 0
    fi
  done
  return 1
}

if [[ -n "${tools_csv}" ]]; then
  IFS=',' read -r -a requested_tools <<< "${tools_csv}"
  for tool in "${requested_tools[@]}"; do
    tool="${tool// /}"
    [[ -z "${tool}" ]] && continue
    if ! is_supported_tool "${tool}"; then
      echo "unsupported tool: ${tool}" >&2
      exit 1
    fi
    selected_tools+=("${tool}")
  done
else
  while IFS=$'\t' read -r tool status _; do
    if [[ "${status}" == "installed" ]]; then
      selected_tools+=("${tool}")
    fi
  done < <(bash "${repo_root}/scripts/qa/terminal/check-tool-availability.sh")
fi

if [[ ${#selected_tools[@]} -lt 3 ]]; then
  echo "insufficient tools installed or selected: need at least 3 for RG-03" >&2
  exit 2
fi

bash "${repo_root}/scripts/qa/terminal/write-evidence-manifest.sh" "${output_dir}"

mkdir -p "${output_dir}/scenarios/RG-03" "${output_dir}/notes"

json_tools=""
for tool in "${selected_tools[@]}"; do
  if [[ -n "${json_tools}" ]]; then
    json_tools+=", "
  fi
  json_tools+="\"${tool}\""
done

cat >"${output_dir}/scenarios/RG-03/results.json" <<JSON
{
  "selected_tools": [${json_tools}],
  "mode": "${mode}",
  "status": "incomplete",
  "requires_hosted_terminal_validation": true
}
JSON

cat >"${output_dir}/notes/rg03-summary.md" <<EOF
# RG-03 Summary

- Mode: \`${mode}\`
- Selected tools: ${selected_tools[*]}
- Validation surface: Haneulchi \`Project Focus / Terminal Deck\`
- Status: \`incomplete\`

Use \`notes/rg03-runbook.md\` to complete operator validation and attach screen captures.
EOF

cat >"${output_dir}/notes/rg03-runbook.md" <<'EOF'
# RG-03 Operator Runbook

Run each selected tool inside a live Haneulchi `Project Focus / Terminal Deck` pane.

For each tool:
1. Launch it from the hosted terminal.
2. Confirm input is not dropped.
3. Confirm alternate screen enter/exit works.
4. Resize the pane and confirm the tool remains usable.
5. Paste clipboard content and confirm it appears correctly.
6. Quit the tool and confirm the shell prompt returns inside the same pane.

Required evidence per tool:
- one screen capture under `evidence/screens/`
- one checklist note under `evidence/notes/`
- one caveat note under `evidence/notes/`
EOF

cat >"${output_dir}/gate-results.json" <<JSON
{
  "RG-02": "not_run",
  "RG-03": "${mode}"
}
JSON

printf 'RG-03 %s prepared for tools: %s\n' "${mode}" "${selected_tools[*]}"
