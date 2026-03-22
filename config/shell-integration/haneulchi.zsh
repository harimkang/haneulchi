# Haneulchi zsh shell integration

function _hc_emit() {
  printf '\037HC|%s|%s\n' "$1" "$2"
}

autoload -Uz add-zsh-hook

function _hc_precmd() {
  _hc_emit cwd "$PWD"
  local branch
  branch="$(command git rev-parse --abbrev-ref HEAD 2>/dev/null || true)"
  [[ -n "$branch" ]] && _hc_emit branch "$branch"
}

function _hc_preexec() {
  _hc_emit command "$1"
}

function _hc_precmd_last_exit() {
  _hc_emit exit "$?"
}

add-zsh-hook precmd _hc_precmd
add-zsh-hook preexec _hc_preexec
add-zsh-hook precmd _hc_precmd_last_exit
