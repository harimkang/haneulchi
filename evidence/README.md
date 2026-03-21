# Evidence

This directory stores release-gate evidence for terminal quality and TUI compatibility.

Expected root files:
- `manifest.json`
- `gate-results.json`
- `environment.json`

Expected subdirectories:
- `scenarios/S-02`
- `scenarios/RG-03`
- `metrics`
- `notes`
- `logs`
- `screens`

`scripts/qa/terminal/write-evidence-manifest.sh` refreshes the root files and creates the required directory structure.
