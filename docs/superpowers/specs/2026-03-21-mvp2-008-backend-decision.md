# MVP2-008 Backend Decision

## Decision

Chosen backend: `SwiftTerm`

## Why It Was Selected Now

`SwiftTerm` is the narrowest native choice that matches the current MVP2-008 boundary. It keeps the terminal surface inside the macOS app stack, avoids introducing a webview-based renderer, and lets the repository lock the first `TerminalSurface` abstraction before PTY lifecycle and split-deck behavior arrive in later tasks.

The repo is still in a scaffold phase, so the decision needs to support repeatable smoke verification more than feature breadth. `SwiftTerm` best fits that constraint while staying aligned with the terminal-first product direction.

## What MVP2-008 Closes

- The native backend choice is fixed to `SwiftTerm`.
- The first hosted Project Focus terminal surface is locked behind a dedicated `TerminalSurface` abstraction.
- Deterministic transcript replay exists for the first hosted surface, including explicit `ready`, `empty`, `degraded`, and `failed` behavior.
- The Rust-to-Swift runtime info bridge and repeatable smoke path exist for local verification.

## What Remains For Later Tasks

- `MVP2-009`: PTY session spawn, kill, resize, and restore lifecycle.
- `MVP2-010`: split, focus, and resize behavior for the terminal deck.
- `MVP2-012`: copy/paste, find, scrollback, hyperlink, and other terminal UX polish.
- `MVP2-013`: multi-tool TUI compatibility smoke coverage.

## AC-03 / AC-04 Status

`AC-03` and `AC-04` are prerequisite-evidence targets only in MVP2-008. This task does not claim either acceptance criterion is closed.

MVP2-008 records the backend decision, the single-surface host, the deterministic replay path, and the verification handoff so later tasks can finish lifecycle and compatibility work without reopening the renderer choice.
