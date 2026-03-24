#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
trace_file="${SMOKE_TRACE_FILE:-}"
dry_run_output_dir=""

cleanup() {
  if [[ -n "${dry_run_output_dir}" && -d "${dry_run_output_dir}" ]]; then
    rm -rf "${dry_run_output_dir}"
  fi
}

trap cleanup EXIT

run_cmd() {
  if [[ -n "${trace_file}" ]]; then
    printf '%s\n' "$*" >> "${trace_file}"
    return 0
  fi

  "$@"
}

run_step() {
  local label="$1"
  shift

  if [[ -z "${trace_file}" ]]; then
    printf '\n==> %s\n' "${label}"
  fi

  run_cmd "$@"
}

show_usage() {
  cat <<'EOF'
Usage: bash scripts/smoke.sh <target>

Targets:
  shell             MVP2-001 / MVP2-002 / MVP2-003
  readiness         MVP2-004 / MVP2-005 / MVP2-006
  readiness-pack    MVP2-049 / RG-01 automated pack
  launcher          MVP2-007
  terminal-surface  MVP2-008
  terminal-quality  RG-02 automated pack
  terminal-deck     MVP2-009 / MVP2-010 / MVP2-012
  workflow          MVP2-055 / MVP2-056 / MVP2-064
  control           Sprint 4 control / parity / security packs

Aliases:
  mvp2-001-002-003
  mvp2-004-005-006
  mvp2-049
  mvp2-007
  mvp2-008
  mvp2-rg02
  mvp2-009-010-012
  mvp2-055-056-064
  sprint4-control
EOF
}

run_swift_checks() {
  run_step "Run macOS Swift tests" swift test --package-path "${repo_root}/apps/macos"
  run_step "Run macOS Swift build" swift build --package-path "${repo_root}/apps/macos"
}

run_core_checks() {
  run_step "Build macOS core artifacts" bash "${repo_root}/scripts/build-macos-core.sh"
  run_step "Run hc-runtime tests" cargo test -p hc-runtime
  run_step "Run hc-ffi tests" cargo test -p hc-ffi
  run_swift_checks
}

target="${1:-}"

case "${target}" in
  shell|mvp2-001-002-003)
    run_swift_checks
    run_step "Show shell checklist" cat "${repo_root}/evidence/notes/WF-00-WF-10-shell-navigation-checklist.md"
    ;;
  readiness|readiness-pack|mvp2-004-005-006|mvp2-049)
    run_swift_checks
    run_step "Prepare RG-01 dry-run evidence" bash "${repo_root}/scripts/qa/readiness/run-rg01-pack.sh" --dry-run
    ;;
  launcher|mvp2-007)
    run_step "Run MVP2-007 focused Swift tests" swift test --package-path "${repo_root}/apps/macos" --filter 'DemoWorkspaceScaffoldTests|WelcomeReadinessViewModelTests|AppShellBootstrapTests'
    run_step "Run macOS Swift build" swift build --package-path "${repo_root}/apps/macos"
    ;;
  terminal-surface|mvp2-008)
    run_core_checks
    ;;
  terminal-quality|mvp2-rg02)
    run_step "Prepare RG-02 dry-run evidence" bash "${repo_root}/scripts/qa/terminal/run-rg02-pack.sh" --dry-run
    ;;
  terminal-deck|mvp2-009-010-012)
    run_core_checks
    if [[ -n "${trace_file}" ]]; then
      dry_run_output_dir="__SMOKE_TMPDIR__"
    else
      dry_run_output_dir="$(mktemp -d)"
    fi
    run_step "Prepare RG-03 dry-run evidence" bash "${repo_root}/scripts/qa/terminal/run-rg03-pack.sh" --dry-run --tools "vim,tmux,lazygit" --output-dir "${dry_run_output_dir}"
    if [[ -z "${trace_file}" ]]; then
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
    fi
    ;;
  workflow|mvp2-055-056-064)
    run_step "Run workflow bridge tests" cargo test -p hc-ffi --test workflow_bridge -- --nocapture
    run_step "Prepare RG-04 dry-run evidence" bash "${repo_root}/scripts/qa/workflow/run-rg04-pack.sh" --dry-run
    run_step "Prepare RG-05 dry-run evidence" bash "${repo_root}/scripts/qa/workflow/run-rg05-pack.sh" --dry-run
    run_step "Prepare RG-06 dry-run evidence" bash "${repo_root}/scripts/qa/workflow/run-rg06-pack.sh" --dry-run
    run_step "Prepare RG-07 dry-run evidence" bash "${repo_root}/scripts/qa/workflow/run-rg07-pack.sh" --dry-run
    ;;
  control|sprint4-control)
    run_step "Prepare RG-05 control pack" bash "${repo_root}/scripts/qa/control/run-rg05-pack.sh" --dry-run
    run_step "Prepare RG-06 control pack" bash "${repo_root}/scripts/qa/control/run-rg06-pack.sh" --dry-run
    run_step "Prepare RG-07 control pack" bash "${repo_root}/scripts/qa/control/run-rg07-pack.sh" --dry-run
    run_step "Prepare RG-09 control pack" bash "${repo_root}/scripts/qa/control/run-rg09-pack.sh" --dry-run
    run_step "Prepare RG-10 control pack" bash "${repo_root}/scripts/qa/control/run-rg10-pack.sh" --dry-run
    ;;
  ""|-h|--help|help)
    show_usage
    ;;
  *)
    printf 'unknown smoke target: %s\n\n' "${target}" >&2
    show_usage >&2
    exit 1
    ;;
esac
