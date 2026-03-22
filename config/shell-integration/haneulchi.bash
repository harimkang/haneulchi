# Haneulchi bash shell integration

_hc_emit() {
  printf '\037HC|%s|%s\n' "$1" "$2"
}

_hc_prompt_command() {
  _hc_emit cwd "$PWD"
  local branch
  branch="$(command git rev-parse --abbrev-ref HEAD 2>/dev/null || true)"
  [ -n "$branch" ] && _hc_emit branch "$branch"
  _hc_emit exit "$?"
}

trap '_hc_emit command "$BASH_COMMAND"' DEBUG
PROMPT_COMMAND="_hc_prompt_command${PROMPT_COMMAND:+; $PROMPT_COMMAND}"
