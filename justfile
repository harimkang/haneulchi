default:
  @just --list

swift-format:
  swiftformat --lint apps/macos

swift-analyze:
  swift build --package-path apps/macos

swift-test:
  swift test --package-path apps/macos

rust-format:
  cargo fmt --all --check

rust-analyze:
  cargo clippy --workspace --all-targets -- -D warnings

rust-test:
  cargo test --workspace

check:
  just swift-format
  just swift-analyze
  just swift-test
  just rust-format
  just rust-analyze
  just rust-test
