#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "${repo_root}"

cargo test -p hc-control-plane --test dispatch_contract
cargo test -p hc-ffi --test dispatch_bridge
