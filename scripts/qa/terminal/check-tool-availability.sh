#!/usr/bin/env bash
set -euo pipefail

tools=(yazi lazygit nvim vim tmux)

for tool in "${tools[@]}"; do
  if command -v "${tool}" >/dev/null 2>&1; then
    printf '%s\tinstalled\t%s\n' "${tool}" "$(command -v "${tool}")"
  else
    printf '%s\tmissing\t-\n' "${tool}"
  fi
done
