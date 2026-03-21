#!/usr/bin/env bash
set -euo pipefail

swift test --package-path apps/macos
cargo test
