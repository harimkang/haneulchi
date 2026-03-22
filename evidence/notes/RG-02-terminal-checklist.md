# RG-02 Terminal Checklist

Automated regression run on `2026-03-22`:
- `bash scripts/smoke.sh terminal-surface` passed.
- `bash scripts/smoke.sh terminal-deck` passed.
- `bash scripts/smoke.sh terminal-quality` captures `metrics/terminal-latency.json` and `scenarios/S-02/checklist.json` as dry-run evidence.

Validation status:
- Split create action keeps the new pane usable:
Automated: `TerminalDeckLayoutTests`, `ProjectFocusSurfaceTests`, `terminal-deck` smoke passed
Manual: pending live operator validation in app
- Resize after split keeps the PTY alive:
Automated: `hc-runtime` session lifecycle tests, `TerminalSessionControllerTests`, `terminal-deck` smoke passed
Manual: pending live resize exercise in app
- Copy/paste works in the hosted terminal:
Automated: `TerminalRendererHostTests.rendererHostForwardsInputAndResize`
Manual: pending clipboard exercise in live hosted terminal
- Scrollback remains accessible after overflow and resize:
Automated: `TerminalRendererHostTests.rendererHostRetainsScrollbackMarkersAfterOverflowAndResize`
Manual: pending operator confirmation in hosted terminal UI
- URL open works from the hosted terminal:
Automated: not directly exercised by smoke scripts in this run
Manual: pending operator validation in app
- Live bootstrap failures are operator-visible:
Automated: `TerminalSessionControllerTests.liveSessionControllerExposesRestoreFailure`, `TerminalSurfaceStateTests.liveSurfaceRestoreFailureIsVisible`
Manual: pending forced-failure exercise in app
