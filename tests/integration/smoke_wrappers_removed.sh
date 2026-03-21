#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

for wrapper in \
  "${repo_root}/scripts/run-mvp2-001-002-003-smoke.sh" \
  "${repo_root}/scripts/run-mvp2-004-005-006-smoke.sh" \
  "${repo_root}/scripts/run-mvp2-007-smoke.sh" \
  "${repo_root}/scripts/run-mvp2-008-smoke.sh" \
  "${repo_root}/scripts/run-mvp2-009-010-012-smoke.sh"
do
  if [[ -e "${wrapper}" ]]; then
    printf 'legacy wrapper still exists: %s\n' "${wrapper}" >&2
    exit 1
  fi
done
