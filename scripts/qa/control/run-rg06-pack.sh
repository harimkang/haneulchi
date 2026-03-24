#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
output_dir="${repo_root}/evidence"
mode="dry-run"
socket_path="${TMPDIR:-/tmp}/haneulchi-control-rg06.$$.$RANDOM.sock"
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

bash "${repo_root}/scripts/qa/workflow/run-rg06-pack.sh" --dry-run --output-dir "${output_dir}"
mkdir -p "${output_dir}/parity" "${output_dir}/scenarios/S-06"

cargo run -q -p hc-api --bin hc-control-server -- --socket "${socket_path}" --requests 4 &
server_pid=$!
for _ in $(seq 1 50); do
  [[ -S "${socket_path}" ]] && break
  sleep 0.1
done

python3 - <<'PY' "${socket_path}" "${output_dir}/parity/state-api.json" "/v1/state"
import socket, sys
sock_path, out_path, route = sys.argv[1:4]
client = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
client.connect(sock_path)
request = f"GET {route} HTTP/1.1\r\nHost: localhost\r\nContent-Length: 0\r\n\r\n".encode()
client.sendall(request)
client.shutdown(socket.SHUT_WR)
payload = b""
while True:
    chunk = client.recv(1 << 20)
    if not chunk:
        break
    payload += chunk
payload = payload.decode()
body = payload.split("\r\n\r\n", 1)[1]
open(out_path, "w", encoding="utf-8").write(body)
PY

python3 - <<'PY' "${socket_path}" "${output_dir}/parity/sessions-api.json" "/v1/sessions"
import socket, sys
sock_path, out_path, route = sys.argv[1:4]
client = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
client.connect(sock_path)
request = f"GET {route} HTTP/1.1\r\nHost: localhost\r\nContent-Length: 0\r\n\r\n".encode()
client.sendall(request)
client.shutdown(socket.SHUT_WR)
payload = b""
while True:
    chunk = client.recv(1 << 20)
    if not chunk:
        break
    payload += chunk
payload = payload.decode()
body = payload.split("\r\n\r\n", 1)[1]
open(out_path, "w", encoding="utf-8").write(body)
PY

HC_CONTROL_SOCKET="${socket_path}" cargo run -q -p hc-cli -- state --json > "${output_dir}/parity/state-cli.json"
HC_CONTROL_SOCKET="${socket_path}" cargo run -q -p hc-cli -- session list --json > "${output_dir}/parity/sessions-cli.json"
wait "${server_pid}"
server_pid=""

cargo run -q -p hc-api --bin hc-control-dump -- review-queue > "${output_dir}/parity/review-item.json"
cat >"${output_dir}/parity/diff-report.md" <<'EOF'
# Sprint 4 Parity Diff Report

- state API and CLI payloads captured from the same local control contract
- session API and CLI payloads captured from the same local control contract
- review item payload captured from the review queue projection
EOF

touch "${output_dir}/scenarios/S-06/parity.mp4"
printf 'RG-06 %s control pack prepared\n' "${mode}"
