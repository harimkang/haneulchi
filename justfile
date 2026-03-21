default:
  @just --list

swift-test:
  cd apps/macos && swift test

rust-test:
  cargo test

check:
  just swift-test
  just rust-test
