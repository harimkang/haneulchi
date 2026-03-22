#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
output_dir="${repo_root}/evidence"
mode="dry-run"
app_package_path="${repo_root}/apps/macos"
log_file=""

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

log_file="${output_dir}/logs/S-01-first-run.log"

run_and_capture() {
  local label="$1"
  shift

  {
    printf '==> %s\n' "${label}"
    "$@"
    printf '\n'
  } >>"${log_file}" 2>&1
}

cat >"${output_dir}/scenarios/S-01/runbook.md" <<'EOF'
# RG-01 Runbook

1. Launch the app with no selected project.
2. Add a project folder.
3. Confirm shell, git, preset, keychain, and workflow status are shown.
4. Continue with Generic Shell even if preset readiness is degraded.
5. Run one shell command inside Project Focus and capture the first-run log.
EOF

cat >"${log_file}" <<EOF
# RG-01 automated readiness pack

mode: ${mode}
package_path: ${app_package_path}
manual_first_command_capture: pending
readiness_suites: ReadinessProbeRunnerTests, WelcomeReadinessViewModelTests
launcher_suites: AppShellBootstrapTests, DemoWorkspaceScaffoldTests

EOF

run_and_capture "Readiness suite" swift test --package-path "${app_package_path}" --filter 'ReadinessProbeRunnerTests|WelcomeReadinessViewModelTests'
run_and_capture "Launcher suite" swift test --package-path "${app_package_path}" --filter 'AppShellBootstrapTests|DemoWorkspaceScaffoldTests'

cat >"${output_dir}/scenarios/S-01/checklist.json" <<JSON
{
  "mode": "${mode}",
  "status": "${mode}",
  "automated_checks": {
    "readiness_suite": "passed",
    "launcher_suite": "passed"
  },
  "manual_artifacts_pending": [
    "screens/S-01-welcome.png",
    "logs/S-01-first-run.log"
  ]
}
JSON

cat >"${output_dir}/notes/rg01-summary.md" <<EOF
# RG-01 Summary

- Mode: \`${mode}\`
- Validation surface: Haneulchi \`WF-01 Welcome / Readiness Launcher\`
- Automated suites:
  - \`ReadinessProbeRunnerTests\`
  - \`WelcomeReadinessViewModelTests\`
  - \`AppShellBootstrapTests\`
  - \`DemoWorkspaceScaffoldTests\`
- Save the launcher screenshot to \`screens/S-01-welcome.png\`.
- Append the first terminal command transcript to \`logs/S-01-first-run.log\`.
- Promote \`RG-01\` to \`pass\` only after the manual artifacts are present.
EOF

python3 - <<'PY' "${output_dir}/gate-results.json" "${mode}"
import json
import sys

path, mode = sys.argv[1], sys.argv[2]
with open(path, "r", encoding="utf-8") as fh:
    data = json.load(fh)

data["RG-01"] = mode

with open(path, "w", encoding="utf-8") as fh:
    json.dump(data, fh, indent=2)
    fh.write("\n")
PY

printf 'RG-01 %s prepared at %s\n' "${mode}" "${output_dir}"
