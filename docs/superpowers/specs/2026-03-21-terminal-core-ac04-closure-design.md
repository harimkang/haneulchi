# Terminal Core AC-04 Closure Design

## Goal
Close the remaining `MVP2-010` and `MVP2-012` gaps, then complete `MVP2-013` so the repository can claim `AC-03` and `AC-04` with repeatable `RG-02` and `RG-03` evidence.

## Scope

### In Scope
- Interactive split-deck focus polish for `WF-02`.
- Explicit hosted-terminal first-responder handoff after split, focus change, restore, and resize.
- Operator-visible terminal UX closure for copy, paste, find, scrollback, and hyperlink behavior.
- Repeatable `RG-03` compatibility pack for at least 3 validated TUI tools from `yazi`, `lazygit`, `vim`/`nvim`, and `tmux`.
- Release-evidence scaffolding for terminal quality and TUI compatibility.

### Out of Scope
- Session header metadata (`MVP2-011`).
- Session Stack (`MVP2-052`).
- New session sheet, preset registry, shell integration, or takeover state machine (`MVP2-014` and later).
- Multi-window, detached panes, tabs, or browser surfaces.
- New control-plane semantics outside the existing terminal runtime boundary.

## Governing References
- `docs/PRD/Haneulchi_PRD_v8_closure_synced.md`
  - `FR-02`: terminal runtime quality must cover split, focus, resize, copy, paste, find, scrollback, hyperlink, and TUI compatibility.
  - `AC-03`: split terminal deck must behave stably.
  - `AC-04`: `RG-03` must pass with at least 3 validated TUIs.
- `docs/tasks/haneulchi_task_docs_v5/EP-02_Terminal_Core/MVP2-010.md`
- `docs/tasks/haneulchi_task_docs_v5/EP-02_Terminal_Core/MVP2-012.md`
- `docs/tasks/haneulchi_task_docs_v5/EP-02_Terminal_Core/MVP2-013.md`
- `docs/architecture/Haneulchi_LoFi_Wireframe_Spec_v5.md`
  - `WF-02` remains the daily-driver terminal deck.
  - Session Stack and Inspector seams stay reserved outside the central deck.
  - Keyboard-first navigation rules remain authoritative.
- `docs/theme/UI_THEME.md`
  - `No-line rule`.
  - `Tonal separation`.
  - `terminal focus is default`.
- `docs/execution/Haneulchi_Release_Gate_and_Evidence_Spec_v1.md`
  - `RG-02`: split/resize/copy-paste/scrollback/URL behavior.
  - `RG-03`: per-tool TUI compatibility pass criteria and evidence requirements.

## Current State
- `MVP2-009` PTY lifecycle, Rust-owned runtime registry, and Swift bridge are already in place.
- `MVP2-010` already has `TerminalDeckLayout` and recursive Project Focus deck rendering, but task status still calls out interactive pane focus polish as open.
- `MVP2-012` already has live IO forwarding, incremental rendering, scrollback smoke coverage, and a smoke script, but task status still calls out explicit first-responder handoff and discoverability polish as open.
- The existing post-PTY smoke script prints a manual checklist, but it does not yet produce `RG-03` evidence artifacts or a repeatable per-tool compatibility result.

## Problem Statement
The repository already proves that a live terminal can run, split, resize, and stream bytes, but it still lacks a stable operator-facing contract for which pane is active, when keyboard focus returns to the hosted terminal, and how terminal UX actions are invoked after deck mutations. Without that closure, `AC-03` stays open. Separately, `AC-04` cannot be claimed until the repo can execute and record a real compatibility pass against at least 3 TUI tools with the evidence structure expected by `RG-03`.

## Design Summary
Use a `closure-first` terminal slice:
- finish the remaining deck focus and hosted-terminal UX gaps first;
- keep the Rust PTY runtime as the source of truth for session lifecycle;
- add a thin Swift coordinator for pane focus and imperative terminal actions;
- add a repeatable compatibility harness that generates `RG-03` evidence files without inventing new control-plane state.

This keeps the current architecture intact: Rust owns PTY sessions, SwiftTerm remains the renderer/input surface, and SwiftUI/AppKit owns focus/UI polish. The compatibility pack stays above the runtime layer as release verification, not as runtime behavior.

## Architecture Decisions

### 1. Separate split/focus state from imperative terminal actions
`TerminalDeckLayout` should remain a pure split/focus model. It should know pane IDs, split ratios, and focus ordering, but it should not know about AppKit first responder or SwiftTerm actions.

Introduce a small Swift-side deck coordinator that:
- tracks the active pane ID;
- registers and unregisters live `TerminalView` handles by pane ID;
- routes imperative actions such as `focus`, `find`, `copy`, `paste`, and `select all` to the active pane;
- re-applies focus after split, restore, and resize operations.

This preserves testability. Layout logic stays deterministic and value-based, while UI side effects are isolated behind a narrow coordinator boundary.

### 2. Keep terminal focus terminal-owned
The theme document explicitly requires terminal focus to remain the default and says supporting surfaces must not steal focus without a reason. The hosted terminal view should therefore explicitly call SwiftTerm's first-responder hook whenever the active pane changes or a split operation creates a new focused pane.

The design does not add a modal workflow for search or actions. Instead:
- `Cmd-F` and explicit “Find” affordances should open SwiftTerm's built-in macOS find bar or invoke its public search helpers;
- copy, paste, and select-all should go through the hosted terminal surface, not through a custom text mirror;
- hyperlink handling should keep using SwiftTerm delegate callbacks plus `NSWorkspace`.

### 3. Treat discoverability as light chrome, not a dominant surface
`MVP2-012` still needs discoverability polish, but the theme and wireframe docs forbid turning the central deck into a button-heavy management surface. The design therefore uses a compact pane header action strip that is only visible on the focused pane or on hover and does not replace keyboard-first flows.

The pane chrome should:
- emphasize active pane via tone and a thin accent rather than heavy borders;
- expose split, focus, and find entry points without expanding beyond the reserved seams;
- avoid floating overlays unless a transient action truly requires one.

### 4. Make `RG-03` a compatibility harness, not a unit-test fiction
`AC-04` requires real tool execution, not only mocks. The harness should:
- detect which candidate tools are installed;
- require at least 3 validated tools before claiming pass;
- run a scenario template for each selected tool covering launch, key input, alternate screen, resize, paste, and quit-return;
- emit result summaries, checklists, and evidence skeleton files under an `evidence/` tree.

The harness may automate environment capture, result JSON generation, and note templates, but screen capture and final operator checklists remain explicit human evidence because the release-gate spec requires real-tested proof. The committed repository should therefore carry a stable evidence skeleton that matches the release-gate root contract, while each run refreshes the generated result files inside that structure.

## Proposed File Boundaries

### Swift UI / AppKit
- Modify: `apps/macos/Sources/HaneulchiApp/Terminal/TerminalDeckLayout.swift`
  - Add deterministic focus-direction helpers and focused-pane selection helpers needed by keyboard-first deck navigation.
- Create: `apps/macos/Sources/HaneulchiApp/Terminal/TerminalDeckCoordinator.swift`
  - Own active pane registration, imperative action routing, and focus re-application.
- Modify: `apps/macos/Sources/HaneulchiApp/ProjectFocus/TerminalDeckView.swift`
  - Bind layout focus state to the coordinator, expose focused-pane chrome, and keep reserved seams intact.
- Modify: `apps/macos/Sources/HaneulchiApp/Terminal/TerminalRendererHost.swift`
  - Publish a small handle for first responder, find, copy, paste, and selection commands against SwiftTerm.
- Modify: `apps/macos/Sources/HaneulchiApp/Terminal/TerminalSurfaceView.swift`
  - Attach the active/focused pane affordances and optional compact action strip without obscuring the terminal.
- Modify: `apps/macos/Sources/HaneulchiApp/Terminal/TerminalSessionController.swift`
  - Expose any small state needed for host readiness or resize/focus timing without broadening ownership.

### Swift tests
- Modify: `apps/macos/Tests/HaneulchiAppTests/TerminalDeckLayoutTests.swift`
  - Cover focus-direction and split-follow-focus behavior.
- Modify: `apps/macos/Tests/HaneulchiAppTests/ProjectFocusSurfaceTests.swift`
  - Cover focused pane contract after restore and split.
- Modify: `apps/macos/Tests/HaneulchiAppTests/TerminalRendererHostTests.swift`
  - Cover first-responder handoff and imperative action routing.
- Create: `apps/macos/Tests/HaneulchiAppTests/TerminalDeckCoordinatorTests.swift`
  - Cover pane registration, active-pane routing, and fallback behavior.

### Verification / release evidence
- Create: `scripts/qa/terminal/run-rg03-pack.sh`
  - Execute the compatibility pack, choose candidate tools, and write result summaries.
- Create: `scripts/qa/terminal/check-tool-availability.sh`
  - Detect installed TUI candidates and emit normalized tool metadata.
- Create: `scripts/qa/terminal/write-evidence-manifest.sh`
  - Prepare the release-gate `evidence/` directory structure and shared environment files.
- Modify: `scripts/run-mvp2-009-010-012-smoke.sh`
  - Delegate to the compatibility harness and stop acting as the only manual checklist.
- Create: `evidence/README.md`
  - Describe expected generated and manually attached artifacts for `RG-02` and `RG-03`.
- Create: `evidence/manifest.json`
- Create: `evidence/gate-results.json`
- Create: `evidence/environment.json`
- Create: `evidence/scenarios/S-02/.gitkeep`
- Create: `evidence/scenarios/RG-03/.gitkeep`
- Create: `evidence/metrics/.gitkeep`
- Create: `evidence/notes/RG-02-terminal-checklist.md`
- Create: `evidence/notes/RG-03-template-checklist.md`
- Create: `evidence/logs/.gitkeep`
- Create: `evidence/screens/.gitkeep`

## UX Rules For This Slice
- Keep the terminal deck visually dominant.
- Use tone and spacing for hierarchy before borders.
- Only the active pane gets the strongest surface tone and accent.
- Do not create a custom full-width toolbar for terminal actions.
- Search and paste should return focus to the active terminal after the command is issued.
- Split actions must keep a clear next-active pane so the operator does not lose keyboard control.

## Verification Strategy

### Automated
- Swift tests for layout focus logic, coordinator routing, and hosted-terminal action hooks.
- Existing Rust PTY lifecycle tests stay as regression coverage.
- Script-level validation for candidate-tool detection and evidence file emission.

### Manual / evidence-backed
- `RG-02` checklist refresh for split, resize, copy/paste, scrollback, IME, and URL open.
- `RG-03` validation for at least 3 installed tools, each with:
  - one screen capture;
  - one completed operator checklist;
  - one caveat note, even if it only says “none observed”.

## Risks And Constraints
- Tool availability on the machine may limit which 3 TUIs are validated; the harness must surface this explicitly instead of silently downgrading the gate.
- SwiftUI focus and AppKit first-responder behavior can regress during split-tree updates; the coordinator must remain narrow and well tested.
- The evidence tree may be partially manual by design; the harness must mark incomplete artifacts clearly so `AC-04` is not claimed prematurely.

## Recommended Execution Order
1. Add coordinator and focus-routing tests first.
2. Close first-responder and focused-pane UX gaps in the deck.
3. Add compact discoverability affordances consistent with the theme docs.
4. Build the `RG-03` harness and evidence scaffolding.
5. Run the compatibility pack against at least 3 installed tools and complete manual artifacts.

## Success Criteria
- Deck split/focus/resize flows keep a deterministic active pane and return keyboard control to the hosted terminal.
- Copy, paste, find, scrollback, and hyperlink behavior are operator-visible and verifiable in `WF-02`.
- The repo can produce `RG-02`/`RG-03` evidence artifacts in a repeatable structure.
- At least 3 real TUI tools satisfy the `RG-03` criteria, allowing `AC-04` to be claimed.
