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

## MVP2-008 Smoke

Use the repeatable MVP2-008 smoke path to refresh the Rust FFI artifacts and run the terminal-core verification set:

```bash
bash scripts/build-macos-core.sh
bash scripts/run-mvp2-008-smoke.sh
```

## MVP2-009 / MVP2-010 / MVP2-012 Smoke

Use the repeatable post-PTY smoke path to verify live session lifecycle, split deck basics, and terminal UX coverage:

```bash
bash scripts/run-mvp2-009-010-012-smoke.sh
```

`apps/macos` builds use the package-local SwiftPM plugin to build `hc_ffi` and generate the transcript catalog from repo-root `fixtures/terminal/`. For local inspection and debugging, the synced vendor artifacts live under:

- `apps/macos/Vendor/lib`
- `apps/macos/Vendor/HCCoreFFI/include`

## Notes

- `reference/` remains read-only.
- Runtime paths under `~/Library/Application Support/Haneulchi` are created outside the repo.
- Secrets should stay in Keychain, not in committed `.env` files.
