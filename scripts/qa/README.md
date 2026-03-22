# scripts/qa

Release-gate and compatibility helpers live here.

- `readiness/run-rg01-pack.sh`: build the `RG-01` readiness acceptance pack and dry-run evidence.
- `terminal/check-tool-availability.sh`: detect candidate TUI tools for `RG-03`.
- `terminal/run-rg02-pack.sh`: build the `RG-02` terminal quality acceptance pack and proxy metrics.
- `terminal/run-rg03-pack.sh`: build the `RG-03` TUI compatibility dry-run pack.
- `terminal/write-evidence-manifest.sh`: create or refresh the release-evidence root files and directory skeleton.
- `workflow/run-rg04-pack.sh`: build the `RG-04` workflow reload dry-run pack.
