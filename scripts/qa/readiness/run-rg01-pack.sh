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

mkdir -p \
  "${output_dir}/scenarios/S-01" \
  "${output_dir}/screens" \
  "${output_dir}/logs" \
  "${output_dir}/notes"

cat >"${output_dir}/scenarios/S-01/runbook.md" <<'EOF'
# RG-01 Runbook

1. Launch the app with no selected project.
2. Add a project folder.
3. Confirm shell, git, preset, keychain, and workflow status are shown.
4. Continue with Generic Shell even if preset readiness is degraded.
5. Run one shell command inside Project Focus and capture the first-run log.
EOF

cat >"${output_dir}/scenarios/S-01/checklist.json" <<JSON
{
  "mode": "${mode}",
  "status": "incomplete",
  "checks": {
    "first_launch_success": false,
    "project_added": false,
    "readiness_status_visible": false,
    "generic_shell_fallback_used": false,
    "first_command_run": false
  },
  "required_manual_artifacts": [
    "screens/S-01-welcome.png",
    "logs/S-01-first-run.log"
  ]
}
JSON

touch "${output_dir}/logs/S-01-first-run.log"

cat >"${output_dir}/notes/rg01-summary.md" <<EOF
# RG-01 Summary

- Mode: \`${mode}\`
- Validation surface: Haneulchi \`WF-01 Welcome / Readiness Launcher\`
- Save the launcher screenshot to \`screens/S-01-welcome.png\`.
- Capture the first terminal command transcript in \`logs/S-01-first-run.log\`.
- Update \`scenarios/S-01/checklist.json\` after the operator run.
EOF

cat >"${output_dir}/gate-results.json" <<JSON
{
  "RG-01": "${mode}"
}
JSON

printf 'RG-01 %s prepared at %s\n' "${mode}" "${output_dir}"
