# Terminal Core AC-04 Closure Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close the remaining `MVP2-010` and `MVP2-012` polish, then complete `MVP2-013` so the repo can claim `AC-03` and `AC-04` with repeatable `RG-02` and `RG-03` evidence.

**Architecture:** Keep Rust as the PTY lifecycle owner and keep SwiftTerm as the native renderer/input surface. Add a thin Swift-side deck coordinator for active-pane focus and imperative terminal actions, then add a release-gate compatibility harness that produces the evidence structure required by `RG-03` without introducing new runtime truth sources.

**Tech Stack:** Swift 6.2, SwiftUI, AppKit, Swift Testing, SwiftTerm 1.12.0, shell scripts, Cargo for regression checks, release-evidence files under `evidence/`.

---

## References

- Spec: `docs/superpowers/specs/2026-03-21-terminal-core-ac04-closure-design.md`
- PRD: `docs/PRD/Haneulchi_PRD_v8_closure_synced.md`
- Architecture: `docs/architecture/Haneulchi_Architecture_Document_v2.md`
- FFI Boundary: `docs/architecture/Haneulchi_FFI_Interface_Spec_v3.md`
- Wireframe: `docs/architecture/Haneulchi_LoFi_Wireframe_Spec_v5.md`
- UI Theme: `docs/theme/UI_THEME.md`
- Release Gate: `docs/execution/Haneulchi_Release_Gate_and_Evidence_Spec_v1.md`
- Tasks:
  - `docs/tasks/haneulchi_task_docs_v5/EP-02_Terminal_Core/MVP2-010.md`
  - `docs/tasks/haneulchi_task_docs_v5/EP-02_Terminal_Core/MVP2-012.md`
  - `docs/tasks/haneulchi_task_docs_v5/EP-02_Terminal_Core/MVP2-013.md`

## Scope Locks

- Do not add Session Stack, session header metadata, shell integration, or takeover state in this slice.
- Do not move PTY lifecycle ownership out of Rust.
- Keep the deck visually terminal-first and preserve the reserved Session Stack and Inspector seams from `WF-02`.
- `AC-04` only closes when at least 3 real TUI tools satisfy `RG-03` and the required evidence exists.
- The compatibility harness may automate structure and summaries, but it must not fake real TUI execution.

## File Structure Map

### Swift terminal focus and action routing

- Create: `apps/macos/Sources/HaneulchiApp/Terminal/TerminalHostHandle.swift`
  - Small AppKit-facing abstraction over imperative terminal actions.
- Create: `apps/macos/Sources/HaneulchiApp/Terminal/TerminalDeckCoordinator.swift`
  - Owns active pane registration, active-pane action routing, and focus re-application.
- Modify: `apps/macos/Sources/HaneulchiApp/Terminal/TerminalDeckLayout.swift`
  - Add deterministic focus helpers used by the coordinator and deck view.
- Modify: `apps/macos/Sources/HaneulchiApp/ProjectFocus/TerminalDeckView.swift`
  - Bind pane selection and split flows to the coordinator.
- Modify: `apps/macos/Sources/HaneulchiApp/Terminal/TerminalRendererHost.swift`
  - Construct `TerminalHostHandle` instances from `TerminalView`.
- Modify: `apps/macos/Sources/HaneulchiApp/Terminal/TerminalSurfaceView.swift`
  - Register pane handles, expose compact terminal actions, and keep the focused pane terminal-owned.

### Swift tests

- Create: `apps/macos/Tests/HaneulchiAppTests/TerminalDeckCoordinatorTests.swift`
  - TDD for active-pane action routing and fallback behavior.
- Modify: `apps/macos/Tests/HaneulchiAppTests/TerminalDeckLayoutTests.swift`
  - Add tests for backward focus movement and explicit focused-pane reassignment.
- Modify: `apps/macos/Tests/HaneulchiAppTests/ProjectFocusSurfaceTests.swift`
  - Add tests for focused live pane behavior after split/restore.
- Modify: `apps/macos/Tests/HaneulchiAppTests/TerminalRendererHostTests.swift`
  - Add tests for host handle creation and action invocation.

### Compatibility harness and evidence

- Create: `scripts/qa/terminal/check-tool-availability.sh`
  - Detect installed TUI candidates and emit a normalized tool list.
- Create: `scripts/qa/terminal/write-evidence-manifest.sh`
  - Create or refresh the release-gate evidence root files and directories.
- Create: `scripts/qa/terminal/run-rg03-pack.sh`
  - Run the compatibility pack and write `RG-03` summaries.
- Create: `tests/integration/rg03_pack_smoke.sh`
  - Script-level smoke check for harness structure and emitted files.
- Modify: `scripts/run-mvp2-009-010-012-smoke.sh`
  - Delegate `RG-03` work to the new harness.
- Modify: `scripts/qa/README.md`
  - Document the release-gate helper layout.
- Modify: `README.md`
  - Document the AC-04 closure flow and compatibility harness entry point.

### Evidence skeleton

- Create: `evidence/README.md`
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

## Chunk 1: Deck Focus and Hosted-Terminal UX Closure

### Task 1: Add active-pane action routing primitives

**Files:**
- Create: `apps/macos/Sources/HaneulchiApp/Terminal/TerminalHostHandle.swift`
- Create: `apps/macos/Sources/HaneulchiApp/Terminal/TerminalDeckCoordinator.swift`
- Create: `apps/macos/Tests/HaneulchiAppTests/TerminalDeckCoordinatorTests.swift`
- Modify: `apps/macos/Tests/HaneulchiAppTests/TerminalDeckLayoutTests.swift`

- [ ] **Step 1: Write the failing coordinator tests**

```swift
import Testing
@testable import HaneulchiApp

@MainActor
@Test("deck coordinator routes find and paste to the active pane")
func deckCoordinatorRoutesActionsToActivePane() {
    let first = RecordingHostHandle()
    let second = RecordingHostHandle()
    let coordinator = TerminalDeckCoordinator()

    coordinator.register(first, for: "pane-1")
    coordinator.register(second, for: "pane-2")
    coordinator.setActivePane("pane-2")

    coordinator.showFind()
    coordinator.pasteClipboard()

    #expect(first.findCalls == 0)
    #expect(second.findCalls == 1)
    #expect(second.pasteCalls == 1)
}
```

- [ ] **Step 2: Add one failing layout regression test**

```swift
@Test("deck layout can move focus backward in presentation order")
func deckMovesFocusBackwardInPresentationOrder() {
    var layout = TerminalDeckLayout.singleDemo
    layout.splitFocusedPane(axis: .horizontal)
    let newestPane = layout.focusedPaneID

    layout.moveFocusBackward()

    #expect(layout.focusedPaneID != newestPane)
}
```

- [ ] **Step 3: Run Swift tests to confirm missing types and methods**

Run: `swift test --package-path apps/macos`

Expected: FAIL with missing `TerminalDeckCoordinator`, `RecordingHostHandle`, or `moveFocusBackward`.

- [ ] **Step 4: Implement the minimal host-handle and coordinator boundary**

```swift
@MainActor
protocol TerminalHostHandle: AnyObject {
    func focusTerminal()
    func showFind()
    func pasteClipboard()
    func copySelection()
    func selectAllText()
}

@MainActor
final class TerminalDeckCoordinator: ObservableObject {
    @Published private(set) var activePaneID: String?
    private var handles: [String: TerminalHostHandle] = [:]

    func register(_ handle: TerminalHostHandle, for paneID: String) {
        handles[paneID] = handle
    }

    func setActivePane(_ paneID: String) {
        activePaneID = paneID
        focusActivePane()
    }
}
```

- [ ] **Step 5: Implement the layout helper**

```swift
mutating func moveFocusBackward() {
    let ids = paneIDs
    guard let currentIndex = ids.firstIndex(of: focusedPaneID), !ids.isEmpty else {
        return
    }

    let previousIndex = currentIndex == ids.startIndex ? ids.index(before: ids.endIndex) : ids.index(before: currentIndex)
    focusedPaneID = ids[previousIndex]
}
```

- [ ] **Step 6: Re-run Swift tests**

Run: `swift test --package-path apps/macos`

Expected: PASS for the new coordinator and layout tests.

- [ ] **Step 7: Commit the focus-routing primitives**

```bash
git add apps/macos/Sources/HaneulchiApp/Terminal/TerminalHostHandle.swift \
  apps/macos/Sources/HaneulchiApp/Terminal/TerminalDeckCoordinator.swift \
  apps/macos/Sources/HaneulchiApp/Terminal/TerminalDeckLayout.swift \
  apps/macos/Tests/HaneulchiAppTests/TerminalDeckCoordinatorTests.swift \
  apps/macos/Tests/HaneulchiAppTests/TerminalDeckLayoutTests.swift
git commit -m "feat: add terminal deck focus coordinator"
```

### Task 2: Wire split/focus behavior into `WF-02`

**Files:**
- Modify: `apps/macos/Sources/HaneulchiApp/ProjectFocus/TerminalDeckView.swift`
- Modify: `apps/macos/Sources/HaneulchiApp/Terminal/TerminalDeckLayout.swift`
- Modify: `apps/macos/Tests/HaneulchiAppTests/ProjectFocusSurfaceTests.swift`

- [ ] **Step 1: Write the failing Project Focus test**

```swift
@MainActor
@Test("splitting a live layout keeps the focused pane live")
func splittingLiveLayoutKeepsFocusedPaneLive() {
    var layout = TerminalDeckLayout.singleLiveDemo
    layout.splitFocusedPane(axis: .vertical)

    #expect(layout.paneIDs.count == 2)
    #expect(layout.focusedSurface?.isLive == true)
}
```

- [ ] **Step 2: Run Swift tests to confirm the UX gap**

Run: `swift test --package-path apps/macos`

Expected: FAIL if focused-pane propagation or test-only mutability is missing.

- [ ] **Step 3: Bind deck selection to the coordinator and keep focus deterministic**

```swift
@StateObject private var deckCoordinator = TerminalDeckCoordinator()
@State private var layout: TerminalDeckLayout

init(model: Model) {
    self.model = model
    _layout = State(initialValue: model.layout)
}

private func paneView(_ pane: TerminalPaneModel) -> some View {
    let isFocused = pane.id == layout.focusedPaneID

    return VStack {
        paneHeader(for: pane, isFocused: isFocused)
        TerminalSurfaceView(
            configuration: pane.surface,
            paneID: pane.id,
            deckCoordinator: deckCoordinator,
            isFocused: isFocused
        )
    }
    .contentShape(Rectangle())
    .onTapGesture {
        layout.focusedPaneID = pane.id
        deckCoordinator.setActivePane(pane.id)
    }
}
```

- [ ] **Step 4: Re-run Swift tests**

Run: `swift test --package-path apps/macos`

Expected: PASS with stable focused-pane behavior after split and restore.

- [ ] **Step 5: Commit the deck wiring**

```bash
git add apps/macos/Sources/HaneulchiApp/ProjectFocus/TerminalDeckView.swift \
  apps/macos/Sources/HaneulchiApp/Terminal/TerminalDeckLayout.swift \
  apps/macos/Tests/HaneulchiAppTests/ProjectFocusSurfaceTests.swift
git commit -m "feat: stabilize project focus pane selection"
```

### Task 3: Close first-responder handoff and compact terminal actions

**Files:**
- Modify: `apps/macos/Sources/HaneulchiApp/Terminal/TerminalRendererHost.swift`
- Modify: `apps/macos/Sources/HaneulchiApp/Terminal/TerminalSurfaceView.swift`
- Modify: `apps/macos/Sources/HaneulchiApp/ProjectFocus/TerminalDeckView.swift`
- Modify: `apps/macos/Tests/HaneulchiAppTests/TerminalRendererHostTests.swift`

- [ ] **Step 1: Write the failing host-action test**

```swift
@MainActor
@Test("host handle forwards actions to the wrapped command target")
func hostHandleForwardsActions() {
    let target = RecordingTerminalCommandTarget()
    let handle = SwiftTermTerminalHostHandle(commandTarget: target)

    handle.focusTerminal()
    handle.showFind()
    handle.pasteClipboard()

    #expect(target.focusCalls == 1)
    #expect(target.findCalls == 1)
    #expect(target.pasteCalls == 1)
}
```

- [ ] **Step 2: Run Swift tests to confirm the host-handle adapter is missing**

Run: `swift test --package-path apps/macos`

Expected: FAIL with missing `RecordingTerminalCommandTarget`, `TerminalCommandTarget`, `SwiftTermTerminalHostHandle`, or action-forwarding helpers.

- [ ] **Step 3: Implement the handle adapter and register it from `TerminalRendererHost`**

```swift
@MainActor
protocol TerminalCommandTarget: AnyObject {
    func focusTerminal()
    func showFind()
    func pasteClipboard()
    func copySelection()
    func selectAllText()
}

@MainActor
final class SwiftTermTerminalHostHandle: TerminalHostHandle {
    private let commandTarget: TerminalCommandTarget

    init(commandTarget: TerminalCommandTarget) {
        self.commandTarget = commandTarget
    }

    func focusTerminal() {
        commandTarget.focusTerminal()
    }

    func showFind() {
        commandTarget.showFind()
    }
}
```

```swift
@MainActor
final class SwiftTermTerminalCommandTarget: TerminalCommandTarget {
    private weak var terminalView: TerminalView?

    init(terminalView: TerminalView) {
        self.terminalView = terminalView
    }

    func focusTerminal() {
        terminalView?.window?.makeFirstResponder(terminalView)
    }

    func showFind() {
        let item = NSMenuItem()
        item.tag = NSTextFinder.Action.showFindInterface.rawValue
        terminalView?.performTextFinderAction(item)
    }
}
```

- [ ] **Step 4: Add a compact action strip that respects the theme rules**

```swift
if isFocused {
    HStack(spacing: 8) {
        Button("Find") { deckCoordinator.showFind() }
        Button("Paste") { deckCoordinator.pasteClipboard() }
        Button("Split H") { split(.horizontal) }
        Button("Split V") { split(.vertical) }
    }
    .font(.caption.weight(.semibold))
    .buttonStyle(.borderless)
}
```

- [ ] **Step 5: Re-run Swift tests**

Run: `swift test --package-path apps/macos`

Expected: PASS, with no regressions to the existing renderer and surface tests.

- [ ] **Step 6: Run Rust regression tests**

Run: `cargo test`

Expected: PASS, proving the Swift-side closure did not regress the PTY/runtime layer.

- [ ] **Step 7: Commit the hosted-terminal UX closure**

```bash
git add apps/macos/Sources/HaneulchiApp/Terminal/TerminalRendererHost.swift \
  apps/macos/Sources/HaneulchiApp/Terminal/TerminalSurfaceView.swift \
  apps/macos/Sources/HaneulchiApp/ProjectFocus/TerminalDeckView.swift \
  apps/macos/Tests/HaneulchiAppTests/TerminalRendererHostTests.swift
git commit -m "feat: close hosted terminal focus and action polish"
```

## Chunk 2: RG-03 Compatibility Harness and Evidence Structure

### Task 4: Add the release-evidence skeleton and availability detection

**Files:**
- Create: `scripts/qa/terminal/check-tool-availability.sh`
- Create: `scripts/qa/terminal/write-evidence-manifest.sh`
- Create: `evidence/README.md`
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
- Modify: `scripts/qa/README.md`

- [ ] **Step 1: Write the failing integration smoke check**

```bash
#!/usr/bin/env bash
set -euo pipefail

tmpdir="$(mktemp -d)"
bash scripts/qa/terminal/write-evidence-manifest.sh "$tmpdir"
test -f "$tmpdir/manifest.json"
test -f "$tmpdir/gate-results.json"
test -f "$tmpdir/environment.json"
```

- [ ] **Step 2: Run the integration smoke check to confirm the scripts are missing**

Run: `bash tests/integration/rg03_pack_smoke.sh`

Expected: FAIL because the evidence scripts and files do not exist yet.

- [ ] **Step 3: Implement tool detection and evidence-root generation**

```bash
available_tools=()
for tool in yazi lazygit nvim vim tmux; do
  if command -v "$tool" >/dev/null 2>&1; then
    available_tools+=("$tool")
  fi
done

cat >"$output_dir/gate-results.json" <<'JSON'
{
  "RG-02": "not_run",
  "RG-03": "not_run"
}
JSON
```

- [ ] **Step 4: Re-run the integration smoke check**

Run: `bash tests/integration/rg03_pack_smoke.sh`

Expected: PASS, proving the evidence skeleton and detection scripts work.

- [ ] **Step 5: Commit the evidence skeleton**

```bash
git add scripts/qa/terminal/check-tool-availability.sh \
  scripts/qa/terminal/write-evidence-manifest.sh \
  tests/integration/rg03_pack_smoke.sh \
  scripts/qa/README.md \
  evidence
git commit -m "chore: add terminal compatibility evidence skeleton"
```

### Task 5: Build the `RG-03` pack runner and wire it into the existing smoke path

**Files:**
- Create: `scripts/qa/terminal/run-rg03-pack.sh`
- Modify: `scripts/run-mvp2-009-010-012-smoke.sh`
- Modify: `README.md`

- [ ] **Step 1: Write the failing dry-run smoke path**

```bash
#!/usr/bin/env bash
set -euo pipefail

tmpdir="$(mktemp -d)"
bash scripts/qa/terminal/run-rg03-pack.sh --dry-run --tools "vim,tmux,lazygit" --output-dir "$tmpdir"
test -f "$tmpdir/notes/rg03-summary.md"
test -f "$tmpdir/scenarios/RG-03/results.json"
```

- [ ] **Step 2: Run the dry-run smoke path to confirm the runner is missing**

Run: `bash tests/integration/rg03_pack_smoke.sh`

Expected: FAIL because `run-rg03-pack.sh` does not exist or does not emit the expected files.

- [ ] **Step 3: Implement the compatibility runner**

```bash
while IFS= read -r tool; do
  case "$tool" in
    vim|nvim|tmux|lazygit|yazi) ;;
    *) echo "unsupported tool: $tool" >&2; exit 1 ;;
  esac
done < <(printf '%s\n' "${selected_tools[@]}")

cat >"$output_dir/scenarios/RG-03/results.json" <<JSON
{
  "selected_tools": ["${selected_tools[@]}"],
  "mode": "${mode}",
  "status": "incomplete"
}
JSON
```

- [ ] **Step 4: Delegate the root smoke script to the new harness**

Run: `bash scripts/run-mvp2-009-010-012-smoke.sh`

Expected: PASS for build/test steps and a printed next-step summary that now points to the `RG-03` harness.

- [ ] **Step 5: Re-run the integration smoke check**

Run: `bash tests/integration/rg03_pack_smoke.sh`

Expected: PASS, with emitted summary and `results.json`.

- [ ] **Step 6: Commit the harness**

```bash
git add scripts/qa/terminal/run-rg03-pack.sh \
  scripts/run-mvp2-009-010-012-smoke.sh \
  tests/integration/rg03_pack_smoke.sh \
  README.md
git commit -m "chore: add RG-03 compatibility harness"
```

## Chunk 3: AC-04 Evidence Closure

### Task 6: Run the real compatibility pass for at least 3 TUIs and close the docs loop

**Files:**
- Modify: `evidence/manifest.json`
- Modify: `evidence/gate-results.json`
- Modify: `evidence/environment.json`
- Create: `evidence/scenarios/RG-03/results.json`
- Create: `evidence/notes/rg03-<tool>-checklist.md`
- Create: `evidence/notes/rg03-<tool>-caveat.md`
- Create: `evidence/screens/rg03-<tool>.mp4` or equivalent screen capture file

- [ ] **Step 1: Refresh build/test baselines before manual validation**

Run: `cargo test && swift test --package-path apps/macos`

Expected: PASS.

- [ ] **Step 2: Run the compatibility harness against real installed tools**

Run: `bash scripts/qa/terminal/run-rg03-pack.sh`

Expected: the script selects at least 3 installed tools or exits with a clear “insufficient tools installed” message.

- [ ] **Step 3: Capture the required per-tool evidence**

For each selected tool:

```text
1. launch
2. type input
3. enter alternate screen
4. resize
5. paste
6. quit and confirm shell recovery
```

Write:
- `evidence/notes/rg03-<tool>-checklist.md`
- `evidence/notes/rg03-<tool>-caveat.md`
- `evidence/screens/rg03-<tool>.mp4`

- [ ] **Step 4: Refresh the gate files**

Run: `bash scripts/qa/terminal/write-evidence-manifest.sh evidence`

Expected: `manifest.json`, `environment.json`, and `gate-results.json` reflect the current run and `RG-03` status.

- [ ] **Step 5: Run the full post-PTY smoke path**

Run: `bash scripts/run-mvp2-009-010-012-smoke.sh`

Expected: PASS on automated steps and a final checklist pointing at the completed evidence files.

- [ ] **Step 6: Commit the final closure artifacts**

```bash
git add evidence README.md scripts/qa/terminal scripts/run-mvp2-009-010-012-smoke.sh
git commit -m "docs: record terminal core AC-04 evidence"
```

## Final Verification Checklist

- `swift test --package-path apps/macos`
- `cargo test`
- `bash tests/integration/rg03_pack_smoke.sh`
- `bash scripts/run-mvp2-009-010-012-smoke.sh`
- `bash scripts/qa/terminal/run-rg03-pack.sh`

## Expected Outcome

- `MVP2-010` no longer has an ambiguous active pane after split or restore.
- `MVP2-012` has explicit hosted-terminal first-responder behavior and operator-visible find/paste/copy flows.
- `MVP2-013` has a repeatable compatibility runner and the evidence required to claim `RG-03`.
- The repository has a concrete release-evidence structure for `RG-02` and `RG-03`, aligned with the execution docs.
