#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
output_dir="${repo_root}/evidence"
mode="live"
app_package_path="${repo_root}/apps/macos"
metrics_file=""
checklist_file=""
log_file=""

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

bash "${repo_root}/scripts/qa/terminal/write-evidence-manifest.sh" "${output_dir}"

mkdir -p \
  "${output_dir}/metrics" \
  "${output_dir}/scenarios/S-02" \
  "${output_dir}/logs" \
  "${output_dir}/notes"

metrics_file="${output_dir}/metrics/terminal-latency.json"
checklist_file="${output_dir}/scenarios/S-02/checklist.json"
log_file="${output_dir}/logs/RG-02-terminal-pack.log"

run_and_capture() {
  local label="$1"
  shift

  {
    printf '==> %s\n' "${label}"
    "$@"
    printf '\n'
  } >>"${log_file}" 2>&1
}

cat >"${log_file}" <<EOF
# RG-02 automated terminal quality pack

mode: ${mode}
package_path: ${app_package_path}
runtime_suite: hc-runtime session lifecycle
renderer_suite: TerminalSessionControllerTests, TerminalRendererHostTests, ProjectFocusSurfaceTests, TerminalDeckLayoutTests

EOF

run_and_capture "Runtime suite" cargo test -p hc-runtime --test session_lifecycle -- --nocapture
run_and_capture "Renderer suite" swift test --package-path "${app_package_path}" --filter 'TerminalSessionControllerTests|TerminalRendererHostTests|ProjectFocusSurfaceTests|TerminalDeckLayoutTests'

python3 - <<'PY' "${metrics_file}"
import json
import math
import os
import pty
import select
import struct
import subprocess
import termios
import time
import fcntl
import sys

metrics_path = sys.argv[1]

def percentile(values, q):
    ordered = sorted(values)
    if not ordered:
        return 0.0
    index = max(0, min(len(ordered) - 1, math.ceil(len(ordered) * q) - 1))
    return ordered[index]

spawn_samples = []
typing_samples = []
resize_samples = []

master, slave = pty.openpty()

try:
    spawn_start = time.perf_counter()
    process = subprocess.Popen(
        ["/bin/sh", "-lc", "stty -echo; cat"],
        stdin=slave,
        stdout=slave,
        stderr=slave,
        close_fds=True,
    )
    spawn_samples.append((time.perf_counter() - spawn_start) * 1000.0)
finally:
    os.close(slave)

try:
    for index in range(10):
        payload = f"ping-{index}\n".encode()
        started = time.perf_counter()
        os.write(master, payload)
        captured = b""
        deadline = time.perf_counter() + 2.0

        while time.perf_counter() < deadline:
            readable, _, _ = select.select([master], [], [], 0.05)
            if not readable:
                continue
            chunk = os.read(master, 1024)
            if chunk:
                captured += chunk
            if payload.strip() in captured:
                break

        if payload.strip() not in captured:
            raise RuntimeError(f"failed to capture echo for sample {index}")

        typing_samples.append((time.perf_counter() - started) * 1000.0)

    for rows, cols in [(24, 80), (40, 120)] * 5:
        started = time.perf_counter()
        fcntl.ioctl(master, termios.TIOCSWINSZ, struct.pack("HHHH", rows, cols, 0, 0))
        resize_samples.append((time.perf_counter() - started) * 1000.0)
finally:
    try:
        process.terminate()
        process.wait(timeout=2.0)
    except Exception:
        process.kill()
        process.wait(timeout=2.0)
    os.close(master)

metrics = {
    "status": "dry-run",
    "collection": "non_ui_proxy",
    "spawn_proxy_p95_ms": round(percentile(spawn_samples, 0.95), 3),
    "typing_proxy_p95_ms": round(percentile(typing_samples, 0.95), 3),
    "typing_proxy_p99_ms": round(percentile(typing_samples, 0.99), 3),
    "resize_ioctl_p95_ms": round(percentile(resize_samples, 0.95), 3),
    "sample_count": len(typing_samples),
}

with open(metrics_path, "w", encoding="utf-8") as fh:
    json.dump(metrics, fh, indent=2)
    fh.write("\n")
PY

cat >"${checklist_file}" <<JSON
{
  "mode": "${mode}",
  "status": "${mode}",
  "automated_checks": {
    "runtime_suite": "passed",
    "renderer_suite": "passed",
    "proxy_metrics": "captured"
  },
  "manual_artifacts_pending": [
    "scenarios/S-02/terminal-proof.mp4",
    "notes/RG-02-terminal-checklist.md"
  ]
}
JSON

python3 - <<'PY' "${output_dir}/gate-results.json" "${mode}"
import json
import sys

path, mode = sys.argv[1], sys.argv[2]
with open(path, "r", encoding="utf-8") as fh:
    data = json.load(fh)

data["RG-02"] = mode

with open(path, "w", encoding="utf-8") as fh:
    json.dump(data, fh, indent=2)
    fh.write("\n")
PY

printf 'RG-02 %s prepared at %s\n' "${mode}" "${output_dir}"
