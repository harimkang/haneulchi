#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
output_dir="${repo_root}/evidence"
mode="dry-run"
socket_path="${TMPDIR:-/tmp}/haneulchi-control-rg09.$$.$RANDOM.sock"
server_pid=""

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

cleanup() {
  if [[ -n "${server_pid}" ]]; then
    wait "${server_pid}" 2>/dev/null || true
  fi
  rm -f "${socket_path}"
}
trap cleanup EXIT

bash "${repo_root}/scripts/qa/terminal/write-evidence-manifest.sh" "${output_dir}"
mkdir -p "${output_dir}/metrics" "${output_dir}/notes" "${output_dir}/logs"

cargo run -q -p hc-api --bin hc-control-server -- --socket "${socket_path}" --requests 1 &
server_pid=$!
for _ in $(seq 1 50); do
  [[ -S "${socket_path}" ]] && break
  sleep 0.1
done
socket_mode="$(stat -f '%Lp' "${socket_path}" 2>/dev/null || stat -c '%a' "${socket_path}")"
python3 - <<'PY' "${socket_path}"
import socket, sys
sock_path = sys.argv[1]
client = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
client.connect(sock_path)
request = b"GET /v1/state HTTP/1.1\r\nHost: localhost\r\nContent-Length: 0\r\n\r\n"
client.sendall(request)
client.shutdown(socket.SHUT_WR)
while client.recv(1 << 20):
    pass
PY
wait "${server_pid}"
server_pid=""

cat >"${output_dir}/metrics/state-snapshot.json" <<JSON
{
  "mode": "${mode}",
  "socket_policy": "local_only",
  "socket_mode": "${socket_mode}",
  "parity_status": "captured"
}
JSON

cat >"${output_dir}/logs/attention-events.jsonl" <<'EOF'
{"kind":"session_error","summary":"placeholder attention event"}
EOF

cat >"${output_dir}/notes/RG-09-security-checklist.md" <<EOF
# RG-09 Security Checklist

- verify user-scoped socket permissions
- confirm no secret values appear in \`/v1/state\`, \`/v1/sessions\`, CLI, or evidence
- confirm local-only transport policy remains enforced
- observed socket mode: ${socket_mode}
EOF

python3 - <<'PY' "${output_dir}/gate-results.json" "${mode}"
import json, sys
path, mode = sys.argv[1], sys.argv[2]
with open(path, "r", encoding="utf-8") as fh:
    data = json.load(fh)
data["RG-09"] = mode
with open(path, "w", encoding="utf-8") as fh:
    json.dump(data, fh, indent=2)
    fh.write("\n")
PY

printf 'RG-09 %s control pack prepared\n' "${mode}"
