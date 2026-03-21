# Haneulchi

Implementation scaffold for the macOS terminal-first workspace described in `docs/`.

## Repository Layout

- `apps/macos`: SwiftUI/AppKit shell package.
- `crates/`: Rust control-plane workspace.
- `fixtures/`: sample projects, workflow contracts, and snapshot fixtures.
- `tests/`: cross-layer contract, integration, and end-to-end notes.
- `scripts/`: local bootstrap and verification helpers.
- `config/`: preset and shell-integration placeholders.

## Quick Start

```bash
just check
```

```bash
scripts/bootstrap/ensure-runtime-dirs.sh
```

## Smoke Commands

Task-specific smoke runners are unified behind one entrypoint:

```bash
bash scripts/smoke.sh <target>
```

Available targets:

- `shell`: global shell chrome, route shortcuts, unread jump, command palette
- `readiness`: Welcome / Readiness Launcher and `RG-01` dry-run scaffolding
- `launcher`: `MVP2-007` demo workspace + no-project launcher checks
- `terminal-surface`: `MVP2-008` Rust FFI + terminal surface verification
- `terminal-deck`: `MVP2-009` / `MVP2-010` / `MVP2-012` live PTY, split deck, terminal UX, `RG-03` dry-run scaffolding

Examples:

```bash
bash scripts/smoke.sh shell
bash scripts/smoke.sh readiness
bash scripts/smoke.sh launcher
bash scripts/smoke.sh terminal-surface
bash scripts/smoke.sh terminal-deck
```

Use `scripts/smoke.sh` as the only supported smoke entrypoint.

`apps/macos` builds use the package-local SwiftPM plugin to build `hc_ffi` and generate the transcript catalog from repo-root `fixtures/terminal/`. For local inspection and debugging, the synced vendor artifacts live under:

- `apps/macos/Vendor/lib`
- `apps/macos/Vendor/HCCoreFFI/include`

## Notes

- `reference/` remains read-only.
- Runtime paths under `~/Library/Application Support/Haneulchi` are created outside the repo.
- Secrets should stay in Keychain, not in committed `.env` files.
