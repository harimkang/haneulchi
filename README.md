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

## Notes

- `reference/` remains read-only.
- Runtime paths under `~/Library/Application Support/Haneulchi` are created outside the repo.
- Secrets should stay in Keychain, not in committed `.env` files.
