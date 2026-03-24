# scripts/qa

Release-gate and compatibility helpers live here.

- `readiness/run-rg01-pack.sh`: build the `RG-01` readiness acceptance pack and dry-run evidence.
- `terminal/check-tool-availability.sh`: detect candidate TUI tools for `RG-03`.
- `terminal/run-rg02-pack.sh`: build the `RG-02` terminal quality acceptance pack and proxy metrics.
- `terminal/run-rg03-pack.sh`: build the `RG-03` TUI compatibility dry-run pack.
- `terminal/write-evidence-manifest.sh`: create or refresh the release-evidence root files and directory skeleton.
- `workflow/run-rg04-pack.sh`: build the `RG-04` workflow reload dry-run pack.
- `workflow/run-rg05-pack.sh`: build the `RG-05` review flow dry-run pack.
- `workflow/run-rg06-pack.sh`: build the `RG-06` workflow parity dry-run pack.
- `workflow/run-rg07-pack.sh`: build the `RG-07` ops attention dry-run pack.
- `control/run-rg05-pack.sh`: wrap `RG-05` for Sprint 4 control evidence.
- `control/run-rg06-pack.sh`: generate Sprint 4 state/session parity evidence placeholders.
- `control/run-rg07-pack.sh`: wrap `RG-07` for Control Tower / attention artifacts.
- `control/run-rg09-pack.sh`: generate local-only socket security notes and metrics.
- `control/run-rg10-pack.sh`: generate Sprint 4 ship-exit and performance placeholders.
