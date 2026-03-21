#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
tmpdir="$(mktemp -d)"
trap 'rm -rf "${tmpdir}"' EXIT

cat >"${tmpdir}/gate-results.json" <<'JSON'
{
  "RG-01": "not_run"
}
JSON

bash "${repo_root}/scripts/qa/readiness/run-rg01-pack.sh" --dry-run --output-dir "${tmpdir}"
python3 - <<'PY' "${tmpdir}/gate-results.json"
import json, sys
path = sys.argv[1]
with open(path, "r", encoding="utf-8") as fh:
    data = json.load(fh)
assert data["RG-01"] == "dry-run", data
assert data["RG-02"] == "not_run", data
assert data["RG-10"] == "not_run", data
PY

bash "${repo_root}/scripts/qa/terminal/run-rg03-pack.sh" --dry-run --tools "vim,tmux,lazygit" --output-dir "${tmpdir}"
python3 - <<'PY' "${tmpdir}/gate-results.json"
import json, sys
path = sys.argv[1]
with open(path, "r", encoding="utf-8") as fh:
    data = json.load(fh)
assert data["RG-01"] == "dry-run", data
assert data["RG-02"] == "not_run", data
assert data["RG-03"] == "dry-run", data
assert data["RG-10"] == "not_run", data
PY
