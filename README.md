# Haneulchi

A terminal-first workspace for running, reviewing, and operating multi-project agent sessions on macOS.

Haneulchi keeps the terminal at the center, then adds the visibility needed to run multiple projects, sessions, and review loops without losing manual control. It is built for CLI-native builders who want reviewable outcomes instead of opaque automation.

## Why Haneulchi Exists

Your terminal is not the problem. The friction starts when tasks, sessions, worktrees, review, and operational signals get split across too many places.

- From tabs to sessions
- From output to evidence
- From automation to controllable flow

Haneulchi brings those flows back into one workspace without replacing the terminal itself.

## What Makes It Different

- Terminal-first by default
- Reviewable outcomes instead of opaque background automation
- Explicit human control at takeover and review points
- Shared control plane across app surfaces, CLI, and runtime
- First-class multi-project and multi-session operations

## How It Works

1. Plan work in a way that keeps task context attached to the session that is actually doing it.
2. Run generic shells, CLI tools, and isolated agent sessions from a terminal-first workspace.
3. Review summaries, diffs, and test hints as outcomes a human can inspect and accept.
4. Operate with readiness, attention, and worktree visibility so recovery and handoff stay explicit.

Surfaces such as the Readiness Launcher, Review Queue, and Attention Center support the terminal workflow instead of trying to replace it.

## Quick Start

Haneulchi is currently macOS-only and under active development.

```bash
scripts/bootstrap/ensure-runtime-dirs.sh
swift test --package-path apps/macos
cargo test
bash scripts/smoke.sh --help
```

If you use [`justfile`](justfile), `just check` wraps the Swift and Rust test commands above.

## Project Layout

- `apps/macos`: SwiftUI/AppKit desktop shell and terminal-facing product surfaces
- `crates`: Rust workspace for runtime, workflow, storage, FFI, CLI, and control-plane components
- `scripts`: bootstrap, smoke, and QA helpers for local development and release-gate checks
- `fixtures`: sample projects, workflow fixtures, and terminal transcript inputs
- `tests`: integration and contract coverage around workspace behavior

## Repository Guides

- [`apps/macos/README.md`](apps/macos/README.md): macOS app shell entry points and test location
- [`crates/README.md`](crates/README.md): Rust workspace component map
- [`scripts/qa/README.md`](scripts/qa/README.md): release-gate and compatibility helper overview

## Open Source

Haneulchi is being built in the open. Issues and pull requests are welcome.

If you want to contribute, start with the checks above and keep changes aligned with the project's terminal-first, reviewable-outcomes, and human-in-the-loop model.

## Current Status

Haneulchi is a macOS-only project in active development. The current scope is intentionally focused on terminal-centered work, review surfaces, and operational visibility while the app shell, terminal, and control-plane layers continue to evolve.

Browser-heavy surfaces and broader orchestration ambitions are intentionally out of scope for the current public README.

## Notes

- Runtime data lives outside the repository under `~/Library/Application Support/Haneulchi`.
- Secrets belong in Keychain, not committed files.
