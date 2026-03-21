#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
dry_run_output_dir="$(mktemp -d)"
trap 'rm -rf "${dry_run_output_dir}"' EXIT

run_step() {
  local label="$1"
  shift

  printf '\n==> %s\n' "${label}"
  "$@"
}

run_step "Build macOS core artifacts" bash "${repo_root}/scripts/build-macos-core.sh"
run_step "Run hc-runtime tests" cargo test -p hc-runtime
run_step "Run hc-ffi tests" cargo test -p hc-ffi
run_step "Run macOS Swift tests" swift test --package-path "${repo_root}/apps/macos"
run_step "Run macOS Swift build" swift build --package-path "${repo_root}/apps/macos"
run_step "Prepare RG-03 dry-run evidence" bash "${repo_root}/scripts/qa/terminal/run-rg03-pack.sh" --dry-run --tools "vim,tmux,lazygit" --output-dir "${dry_run_output_dir}"

cat <<'CHECKLIST'

Manual smoke checklist:
1. With a saved terminal restore bundle present, app boots into a live shell prompt in Project Focus; without one, the shell stays on the non-live fallback path.
2. Horizontal and vertical split layouts both render and keep focus visible.
3. Typing `printf 'copy target\n'` then `Cmd+F` can find `copy target`.
4. Paste sends bytes to the live session.
5. A printed `https://example.com` link opens externally.
6. After printing more than one viewport of output, older lines remain reachable via scrollback before and after resize.
7. `vim` or `less` survives alternate screen entry/exit and pane resize.
8. Run `bash scripts/qa/terminal/run-rg03-pack.sh` to populate `evidence/` and complete `evidence/notes/rg03-runbook.md` inside Haneulchi `Project Focus / Terminal Deck` for at least 3 real tools.

CHECKLIST
