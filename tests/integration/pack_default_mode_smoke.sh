#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
tmpdir="$(mktemp -d)"
trap 'rm -rf "${tmpdir}"' EXIT

bash "${repo_root}/scripts/qa/workflow/run-rg04-pack.sh" --output-dir "${tmpdir}"

python3 - <<'PY' "${tmpdir}/gate-results.json"
import json
import sys

path = sys.argv[1]
with open(path, "r", encoding="utf-8") as fh:
    data = json.load(fh)

assert data["RG-04"] == "dry-run", data
PY
