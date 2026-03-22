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

mkdir -p "${output_dir}/scenarios/RG-04" "${output_dir}/scenarios/S-05" "${output_dir}/notes"

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

cat >"${output_dir}/scenarios/S-05/runbook.md" <<'EOF'
# S-05 Workflow Reload Runbook

Use this runbook to complete the hosted `RG-04` / `AC-18` / `AC-24` validation.

Preflight:
1. Open a project with a known-good `WORKFLOW.md`.
2. Keep one running hosted session open before changing the workflow file.
3. Open the `Workflow Contract / Runbook Drawer`.

Valid-path sequence:
1. Confirm the drawer shows `state: ok`, watched path, last-known-good hash, review checklist, allowed agents, and hooks.
2. Record one screenshot for the valid state.
3. Save one operator note to `notes/rg04-valid-operator-note.md`.

Invalid-reload sequence:
1. Edit `WORKFLOW.md` into an invalid state.
2. Wait for the auto-reload window or use `Reload Workflow` to force evaluation.
3. Confirm `invalid_kept_last_good` is shown.
4. Confirm the previous last-known-good hash remains visible.
5. Confirm the already running session still behaves as before and keeps its prior launch context.
6. Record one screenshot for the invalid state.
7. Save one operator note to `notes/rg04-invalid-operator-note.md`.

Promotion rule:
- Do not promote `RG-04` to `pass` until both valid and invalid paths have screenshots and operator notes.
EOF

cat >"${output_dir}/notes/RG-04-workflow-checklist.md" <<'EOF'
# RG-04 Workflow Checklist

1. Validate a known-good `WORKFLOW.md` and confirm `state: ok`.
2. Open the workflow drawer and confirm watched path, last reload time, last-known-good hash, review checklist, allowed agents, and hooks are visible.
3. Capture the valid-path screenshot and write `notes/rg04-valid-operator-note.md`.
4. Introduce an invalid reload and confirm `invalid_kept_last_good` is shown.
5. Confirm the previous last-known-good hash remains visible after invalid reload.
6. Confirm the running session is not mutated by reload and that only future launch behavior changes.
7. Capture the invalid-path screenshot and write `notes/rg04-invalid-operator-note.md`.
EOF

cat >"${output_dir}/notes/rg04-valid-operator-note.md" <<'EOF'
# RG-04 valid path operator note

- Validation surface:
- Watched path visible:
- Last-known-good hash visible:
- Review checklist visible:
- Allowed agents visible:
- Hook list visible:
- Screenshot path:
- Caveat summary:
EOF

cat >"${output_dir}/notes/rg04-invalid-operator-note.md" <<'EOF'
# RG-04 invalid path operator note

- Validation surface:
- Invalid reload state visible:
- Last-known-good retained:
- Running session unaffected:
- Screenshot path:
- Caveat summary:
EOF

cat >"${output_dir}/notes/rg04-summary.md" <<EOF
# RG-04 Summary

- Mode: \`${mode}\`
- Scenario set: valid validate / invalid reload kept last good
- Validation surface: Haneulchi \`Workflow Contract / Runbook Drawer\`
- Status: \`incomplete\`

Use \`scenarios/S-05/runbook.md\`, \`notes/RG-04-workflow-checklist.md\`, and the operator notes to complete hosted validation.
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
