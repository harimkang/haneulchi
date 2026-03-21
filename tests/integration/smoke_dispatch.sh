#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
tmpdir="$(mktemp -d)"
trace_file="${tmpdir}/trace.log"
trap 'rm -rf "${tmpdir}"' EXIT

SMOKE_TRACE_FILE="${trace_file}" bash "${repo_root}/scripts/smoke.sh" shell
grep -F "swift test --package-path ${repo_root}/apps/macos" "${trace_file}"
grep -F "swift build --package-path ${repo_root}/apps/macos" "${trace_file}"
grep -F "cat ${repo_root}/evidence/notes/WF-00-WF-10-shell-navigation-checklist.md" "${trace_file}"

: > "${trace_file}"
SMOKE_TRACE_FILE="${trace_file}" bash "${repo_root}/scripts/smoke.sh" readiness
grep -F "bash ${repo_root}/scripts/qa/readiness/run-rg01-pack.sh --dry-run" "${trace_file}"

: > "${trace_file}"
SMOKE_TRACE_FILE="${trace_file}" bash "${repo_root}/scripts/smoke.sh" terminal-deck
grep -F "bash ${repo_root}/scripts/qa/terminal/run-rg03-pack.sh --dry-run --tools vim,tmux,lazygit --output-dir __SMOKE_TMPDIR__" "${trace_file}"
