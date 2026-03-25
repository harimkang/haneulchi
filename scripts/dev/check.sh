#!/usr/bin/env bash
set -euo pipefail

swiftformat --lint apps/macos
swift build --package-path apps/macos
swift test --package-path apps/macos
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
