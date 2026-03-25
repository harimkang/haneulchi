# Sprint 05 Inventory, Recovery & Ship Exit Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement Sprint 5 code-close scope: recovery-first inventory, WF-09 settings/integrations, real Keychain secret injection, Rust-backed persistence/restore, degraded recovery flows, and automated verification while deferring manual operator evidence work.

**Architecture:** Extend the existing Rust single-authority snapshot and shared SQLite store instead of adding new Swift-owned state. Rust owns inventory/settings/recovery models, persistence, and action semantics; SwiftUI reads those projections through FFI and only manages presentation, focus, and overlay flow. Existing Swift JSON restore stores remain temporary fallbacks until Rust persistence is wired end-to-end.

**Tech Stack:** Rust (`hc-domain`, `hc-storage`, `hc-control-plane`, `hc-runtime`, `hc-workflow`, `hc-ffi`, `hc-api`, `hc-cli`), SwiftUI/AppKit shell, SQLite (`rusqlite`), macOS Security/Keychain, Unix domain socket API, `portable-pty`, Swift Package Manager.

---

## Scope and Defer Lock

**In scope**

- WF-13 recovery-first inventory projection and UI
- WF-09 `Automation & Integrations` section and workflow reload diagnostics
- Actual Keychain-backed secret ref storage and env injection
- Rust-backed app/session/layout/worktree persistence and restore
- Degraded recovery code paths for:
  - `preset_missing`
  - `missing_project_path`
  - `deleted_repo`
  - `keychain_ref_missing`
  - `invalid_workflow_reload`
  - `before_run_hook_failure`
  - `worktree_unreachable`
  - `crashed_restore`
  - `stale_claim_reconcile`
- Automated readiness / compatibility smoke / degraded / redaction / parity coverage

**Deferred after Sprint 5**

- Manual TUI operator certification (`yazi`, `lazygit`, `vim` or `nvim`, `tmux`, IME)
- Screenshots, videos, runbooks, checklist JSON, evidence manifest assembly
- Hosted RG-01 ~ RG-10 promotion and final ship decision

---

## File Map

### Rust domain and storage

- `crates/hc-domain/src/inventory.rs`: inventory vocabulary, row/disposition payloads
- `crates/hc-domain/src/settings.rs`: terminal settings, secrets, persistence, recovery payloads
- `crates/hc-storage/src/cache.rs`: cache scan, quota, and entry repositories
- `crates/hc-storage/src/settings.rs`: terminal settings, secret refs, worktree policies, notification rules
- `crates/hc-storage/src/persistence.rs`: projects, layouts, session metadata, app state repositories
- `crates/hc-storage/src/keychain.rs`: macOS Keychain boundary, metadata-only DB integration
- `crates/hc-storage/src/worktrees.rs`: lifecycle, size, pin state, last access, stale lookup helpers

### Rust control plane and runtime

- `crates/hc-control-plane/src/inventory.rs`: recovery-first inventory projection and action gating
- `crates/hc-control-plane/src/settings.rs`: settings services and defaults
- `crates/hc-control-plane/src/persistence.rs`: save/restore orchestration
- `crates/hc-control-plane/src/recovery.rs`: degraded issue detection and recovery action resolution
- `crates/hc-control-plane/src/commands.rs`: shared command/query entry points
- `crates/hc-runtime/src/terminal/session.rs`: environment-aware terminal launch config
- `crates/hc-workflow/tests/*.rs`: workflow reload / hook failure regression coverage

### FFI / API / CLI boundary

- `crates/hc-ffi/src/inventory_bridge.rs`: inventory queries and actions for Swift
- `crates/hc-ffi/src/settings_bridge.rs`: settings and secret-ref FFI entry points
- `crates/hc-ffi/src/recovery_bridge.rs`: degraded issue and restore status FFI entry points
- `crates/hc-ffi/src/api_server_bridge.rs`: local API runtime-info summary for Settings
- `crates/hc-api/tests/*.rs`: UDS local-only and redaction contract tests
- `crates/hc-cli/tests/cli_parity.rs`: snapshot/session parity regression after snapshot expansion

### Swift shell

- `apps/macos/Sources/HaneulchiApp/Bridge/CoreBridge.swift`: Swift entry point for new FFI calls
- `apps/macos/Sources/HaneulchiApp/Settings/*`: WF-09 sections, view models, actions
- `apps/macos/Sources/HaneulchiApp/Inventory/*`: WF-13 inventory overlay and row rendering
- `apps/macos/Sources/HaneulchiApp/AppShell/*`: overlay presentation, shortcuts, command palette, restore flow
- `apps/macos/Sources/HaneulchiApp/ProjectFocus/WorkflowDrawerView.swift`: workflow reload diagnostics parity
- `apps/macos/Sources/HaneulchiApp/Terminal/TerminalSessionController.swift`: launch env + restore metadata integration

---

## Chunk 1: Rust Foundations

### Task 1: Add Sprint 5 domain vocabulary

**Files:**

- Create: `crates/hc-domain/src/inventory.rs`
- Create: `crates/hc-domain/src/settings.rs`
- Modify: `crates/hc-domain/src/lib.rs`
- Test: `crates/hc-domain/tests/inventory_vocabulary.rs`
- Test: `crates/hc-domain/tests/settings_vocabulary.rs`

- [ ] Write failing domain tests for `InventoryDisposition`, `WorktreeLifecycleState`, `SecretRef`, `DegradedIssue`, and `RecoveryAction`.
- [ ] Add `inventory.rs` with `WorktreeLifecycleState`, `InventoryDisposition`, `InventorySummary`, `InventoryRow`, and `RestorePointSummary`.
- [ ] Add `settings.rs` with `TerminalSettings`, `SecretRef`, `WorktreePolicy`, `NotificationRule`, `ProjectRecord`, `LayoutRecord`, `SessionMetadataRecord`, `AppStateRecord`, `DegradedIssue`, and `RecoveryAction`.
- [ ] Re-export new modules from `crates/hc-domain/src/lib.rs` without changing existing snapshot/session enum labels.
- [ ] Run `cargo test -p hc-domain --test inventory_vocabulary --test settings_vocabulary` and expect `test result: ok`.
- [ ] Commit with `feat: add sprint 5 domain vocabulary`.

### Task 2: Extend SQLite schema and repositories

**Files:**

- Create: `crates/hc-storage/src/cache.rs`
- Create: `crates/hc-storage/src/settings.rs`
- Create: `crates/hc-storage/src/persistence.rs`
- Create: `crates/hc-storage/src/keychain.rs`
- Modify: `crates/hc-storage/src/lib.rs`
- Modify: `crates/hc-storage/src/schema.rs`
- Modify: `crates/hc-storage/src/worktrees.rs`
- Test: `crates/hc-storage/tests/cache_repository.rs`
- Test: `crates/hc-storage/tests/settings_repository.rs`
- Test: `crates/hc-storage/tests/persistence_repository.rs`
- Modify: `crates/hc-storage/tests/worktree_repository.rs`

- [ ] Write failing storage tests for cache rows, settings rows, restore rows, and expanded worktree lifecycle fields.
- [ ] Add Sprint 5 tables and columns to `crates/hc-storage/src/schema.rs`: `cache_roots`, `cache_entries`, `cache_quotas`, `terminal_settings`, `secret_refs`, `worktree_policies`, `notification_rules`, `projects`, `layouts`, `session_metadata`, `app_state`, plus lifecycle/size/pin fields on `worktrees`.
- [ ] Implement `CacheRepository` in `crates/hc-storage/src/cache.rs`.
- [ ] Implement settings repositories in `crates/hc-storage/src/settings.rs`.
- [ ] Implement restore/persistence repositories in `crates/hc-storage/src/persistence.rs`.
- [ ] Expand `crates/hc-storage/src/worktrees.rs` with lifecycle, pin, stale lookup, and last-access helpers.
- [ ] Export new repositories from `crates/hc-storage/src/lib.rs` and add `SqliteStore` accessors.
- [ ] Run `cargo test -p hc-storage --test worktree_repository --test cache_repository --test settings_repository --test persistence_repository` and expect all PASS.
- [ ] Commit with `feat: add sprint 5 storage schema and repositories`.

### Task 3: Thread Keychain-backed env through terminal launch

**Files:**

- Modify: `crates/hc-runtime/src/terminal/session.rs`
- Modify: `crates/hc-runtime/tests/session_lifecycle.rs`
- Modify: `apps/macos/Sources/HaneulchiApp/Sessions/SessionLaunchDescriptor.swift`
- Modify: `apps/macos/Sources/HaneulchiApp/Terminal/TerminalSessionController.swift`
- Modify: `apps/macos/Sources/HaneulchiApp/Terminal/TerminalSessionRestoreStore.swift`

- [ ] Write a failing runtime test proving an injected env var reaches the spawned process.
- [ ] Add an `environment` map to `TerminalLaunchConfig` in `crates/hc-runtime/src/terminal/session.rs` and pass it into `portable_pty::CommandBuilder`.
- [ ] Preserve env-capable launch data in Swift `TerminalSessionLaunchRequest`, `SessionLaunchDescriptor`, and restore bundle models without rendering secret values in UI summaries.
- [ ] Re-run `cargo test -p hc-runtime --test session_lifecycle` and expect the new env injection test to pass.
- [ ] Run `swift test --package-path apps/macos --filter TerminalSessionControllerTests` and expect PASS after the Swift launch model change.
- [ ] Commit with `feat: support secret env injection in terminal launch`.

---

## Chunk 2: Control Plane Services

### Task 4: Build recovery-first inventory projection and actions

**Files:**

- Create: `crates/hc-control-plane/src/inventory.rs`
- Modify: `crates/hc-control-plane/src/worktrees.rs`
- Modify: `crates/hc-control-plane/src/attention.rs`
- Modify: `crates/hc-control-plane/src/lib.rs`
- Modify: `crates/hc-control-plane/src/commands.rs`
- Test: `crates/hc-control-plane/tests/inventory_projection.rs`
- Modify: `crates/hc-control-plane/tests/worktree_provisioning.rs`

- [ ] Write failing control-plane tests for the four disposition groups: `in_use`, `recoverable`, `safe_to_delete`, `stale`.
- [ ] Implement `crates/hc-control-plane/src/inventory.rs` to merge worktrees, cache rows, restore metadata, and degraded flags into a single `InventoryRow` projection.
- [ ] Extend `crates/hc-control-plane/src/worktrees.rs` with lifecycle transitions, pin/unpin behavior, cleanup behavior, and stale row detection.
- [ ] Emit `cleanup_suggestion` and `recovery_required` attention items from `crates/hc-control-plane/src/attention.rs` using existing vocabulary.
- [ ] Expose inventory list/summary/actions from `crates/hc-control-plane/src/commands.rs` and re-export from `crates/hc-control-plane/src/lib.rs`.
- [ ] Run `cargo test -p hc-control-plane --test inventory_projection --test worktree_provisioning` and expect PASS.
- [ ] Commit with `feat: add recovery-first inventory projection`.

### Task 5: Implement settings, persistence, and degraded recovery services

**Files:**

- Create: `crates/hc-control-plane/src/settings.rs`
- Create: `crates/hc-control-plane/src/persistence.rs`
- Create: `crates/hc-control-plane/src/recovery.rs`
- Modify: `crates/hc-control-plane/src/commands.rs`
- Modify: `crates/hc-control-plane/src/shared_store.rs`
- Modify: `crates/hc-control-plane/src/snapshot.rs`
- Modify: `crates/hc-control-plane/src/reconcile.rs`
- Modify: `crates/hc-control-plane/src/workflow_projection.rs`
- Test: `crates/hc-control-plane/tests/persistence_restore.rs`
- Test: `crates/hc-control-plane/tests/recovery_flows.rs`
- Test: `crates/hc-control-plane/tests/security_redaction.rs`

- [ ] Write failing tests for restore round-trips, stale-claim reconcile, invalid workflow reload surfacing, missing Keychain refs, and redaction.
- [ ] Implement terminal settings read/update services in `crates/hc-control-plane/src/settings.rs`.
- [ ] Implement Rust-backed save/restore orchestration in `crates/hc-control-plane/src/persistence.rs`, including last project, last route, layout, and session metadata.
- [ ] Implement degraded issue detection and action routing in `crates/hc-control-plane/src/recovery.rs` for all Sprint 5 scenarios.
- [ ] Update `crates/hc-control-plane/src/reconcile.rs` and `crates/hc-control-plane/src/workflow_projection.rs` so startup and runtime reconcile paths emit `stale_claim_reconcile` and `invalid_workflow_reload` issues with the same codes the UI will show.
- [ ] Update `crates/hc-control-plane/src/snapshot.rs` so snapshot warnings/ops fields surface the new recovery and workflow health information without leaking secrets.
- [ ] Run `cargo test -p hc-control-plane --test persistence_restore --test recovery_flows --test security_redaction` and expect PASS.
- [ ] Commit with `feat: add sprint 5 persistence and degraded recovery services`.

### Task 6: Expose Sprint 5 data through FFI and harden the local control boundary

**Files:**

- Create: `crates/hc-ffi/src/inventory_bridge.rs`
- Create: `crates/hc-ffi/src/settings_bridge.rs`
- Create: `crates/hc-ffi/src/recovery_bridge.rs`
- Modify: `crates/hc-ffi/src/lib.rs`
- Modify: `crates/hc-ffi/include/hc_ffi.h`
- Modify: `crates/hc-ffi/src/api_server_bridge.rs`
- Create: `crates/hc-ffi/tests/inventory_bridge.rs`
- Create: `crates/hc-ffi/tests/settings_bridge.rs`
- Create: `crates/hc-ffi/tests/recovery_bridge.rs`
- Modify: `crates/hc-ffi/tests/runtime_info.rs`
- Create: `crates/hc-api/tests/security_contract.rs`
- Modify: `crates/hc-api/tests/server_contract.rs`
- Modify: `crates/hc-cli/tests/cli_parity.rs`

- [ ] Write failing FFI bridge tests for inventory summary/list/actions, settings payloads, recovery payloads, and local API runtime info.
- [ ] Implement new FFI bridge modules and export them from `crates/hc-ffi/src/lib.rs` and `crates/hc-ffi/include/hc_ffi.h`.
- [ ] Extend `crates/hc-ffi/src/api_server_bridge.rs` with a read-only runtime-info payload that includes socket path and local-only transport summary for the Settings surface.
- [ ] Add `crates/hc-api/tests/security_contract.rs` to verify Unix socket `0600` permissions and redaction from `/v1/state` and `/v1/sessions`.
- [ ] Refresh `crates/hc-api/tests/server_contract.rs` and `crates/hc-cli/tests/cli_parity.rs` if snapshot/session payloads gain new optional fields.
- [ ] Run `cargo test -p hc-ffi --test inventory_bridge --test settings_bridge --test recovery_bridge --test runtime_info` and expect PASS.
- [ ] Run `cargo test -p hc-api --test server_contract --test security_contract` and `cargo test -p hc-cli --test cli_parity` and expect PASS.
- [ ] Commit with `feat: expose sprint 5 projections through ffi and api`.

---

## Chunk 3: Swift Shell Integration

### Task 7: Upgrade WF-09 Settings and workflow surfaces

**Files:**

- Modify: `apps/macos/Sources/HaneulchiApp/Bridge/CoreBridge.swift`
- Modify: `apps/macos/Sources/HaneulchiApp/Settings/SettingsView.swift`
- Modify: `apps/macos/Sources/HaneulchiApp/Settings/SettingsStatusViewModel.swift`
- Modify: `apps/macos/Sources/HaneulchiApp/Settings/ReadinessSettingsSection.swift`
- Modify: `apps/macos/Sources/HaneulchiApp/Settings/SettingsAutomationStatusSection.swift`
- Modify: `apps/macos/Sources/HaneulchiApp/Settings/AutomationControlPanelView.swift`
- Modify: `apps/macos/Sources/HaneulchiApp/Settings/AutomationControlPanelViewModel.swift`
- Create: `apps/macos/Sources/HaneulchiApp/Settings/TerminalSettingsSection.swift`
- Create: `apps/macos/Sources/HaneulchiApp/Settings/SecretsSettingsSection.swift`
- Create: `apps/macos/Sources/HaneulchiApp/Settings/WorktreeRecoverySection.swift`
- Modify: `apps/macos/Sources/HaneulchiApp/ProjectFocus/WorkflowDrawerView.swift`
- Modify: `apps/macos/Tests/HaneulchiAppTests/CoreBridgeTests.swift`
- Modify: `apps/macos/Tests/HaneulchiAppTests/SettingsStatusViewModelTests.swift`
- Modify: `apps/macos/Tests/HaneulchiAppTests/AutomationControlPanelViewModelTests.swift`
- Modify: `apps/macos/Tests/HaneulchiAppTests/WorkflowContractViewModelTests.swift`

- [ ] Write failing Swift tests for Settings sections, workflow reload error visibility, and API runtime-info summaries.
- [ ] Extend `CoreBridge` with inventory/settings/recovery/runtime-info calls that match the new FFI bridges.
- [ ] Restructure `SettingsView` into explicit Sprint 5 sections: `Terminal`, `Diagnostics`, `Secrets`, `Worktree & Recovery`, `Automation & Integrations`.
- [ ] Surface workflow watch state, last reload time, last error, local API socket path, CLI status, cadence/slots/retry defaults, and tracker summary from Rust-backed view models.
- [ ] Update `WorkflowDrawerView` so invalid reload and last-known-good details match the same workflow vocabulary used in Settings.
- [ ] Run `swift test --package-path apps/macos --filter SettingsStatusViewModelTests` and `swift test --package-path apps/macos --filter WorkflowContractViewModelTests` and expect PASS.
- [ ] Commit with `feat: connect settings and workflow surfaces to sprint 5 state`.

### Task 8: Implement WF-13 recovery-first inventory surface

**Files:**

- Create: `apps/macos/Sources/HaneulchiApp/Inventory/WorktreeInventoryView.swift`
- Create: `apps/macos/Sources/HaneulchiApp/Inventory/WorktreeInventoryViewModel.swift`
- Create: `apps/macos/Sources/HaneulchiApp/Inventory/WorktreeInventoryRowView.swift`
- Modify: `apps/macos/Sources/HaneulchiApp/AppShell/AppShellAction.swift`
- Modify: `apps/macos/Sources/HaneulchiApp/AppShell/AppShellCommands.swift`
- Modify: `apps/macos/Sources/HaneulchiApp/AppShell/AppShellModel.swift`
- Modify: `apps/macos/Sources/HaneulchiApp/AppShell/AppShellView.swift`
- Modify: `apps/macos/Sources/HaneulchiApp/AppShell/CommandPalette/CommandPaletteCatalog.swift`
- Modify: `apps/macos/Sources/HaneulchiApp/AppShell/CommandPalette/InventorySearchProjectionStore.swift`
- Create: `apps/macos/Tests/HaneulchiAppTests/WorktreeInventoryViewModelTests.swift`
- Modify: `apps/macos/Tests/HaneulchiAppTests/AppShellActionTests.swift`
- Modify: `apps/macos/Tests/HaneulchiAppTests/InventorySearchProjectionStoreTests.swift`

- [ ] Write failing Swift tests for inventory grouping order, action gating, and `Cmd+Shift+I` presentation flow.
- [ ] Add a dedicated inventory overlay/panel instead of a placeholder route, and wire presentation/dismissal through `AppShellAction`, `AppShellModel`, and `AppShellView`.
- [ ] Replace the current restore-root approximation in `InventorySearchProjectionStore` with real Rust inventory projection data.
- [ ] Render summary cards before filters, then render group sections in the exact order `In Use`, `Recoverable`, `Safe to Delete`, `Stale`.
- [ ] Gate row actions according to the Rust disposition/action contract and route open actions back into existing session/task/finder flows.
- [ ] Run `swift test --package-path apps/macos --filter WorktreeInventoryViewModelTests` and `swift test --package-path apps/macos --filter AppShellActionTests` and expect PASS.
- [ ] Commit with `feat: add recovery-first worktree inventory`.

### Task 9: Move restore flow onto Rust persistence

**Files:**

- Modify: `apps/macos/Sources/HaneulchiApp/AppShell/AppShellPreferencesStore.swift`
- Modify: `apps/macos/Sources/HaneulchiApp/Terminal/TerminalSessionRestoreStore.swift`
- Modify: `apps/macos/Sources/HaneulchiApp/AppShell/LocalAppShellSnapshotSource.swift`
- Modify: `apps/macos/Sources/HaneulchiApp/AppShell/AppShellModel.swift`
- Modify: `apps/macos/Sources/HaneulchiApp/ProjectFocus/ProjectFocusView.swift`
- Modify: `apps/macos/Sources/HaneulchiApp/ProjectFocus/ProjectFocusWorkspaceState.swift`
- Modify: `apps/macos/Sources/HaneulchiApp/Sessions/NewSessionSheetViewModel.swift`
- Modify: `apps/macos/Tests/HaneulchiAppTests/AppShellBootstrapTests.swift`
- Modify: `apps/macos/Tests/HaneulchiAppTests/AppShellSnapshotSourceTests.swift`
- Modify: `apps/macos/Tests/HaneulchiAppTests/TerminalSessionRestoreStoreTests.swift`
- Modify: `apps/macos/Tests/HaneulchiAppTests/NewSessionSheetViewModelTests.swift`

- [ ] Write failing Swift tests for last project / route / layout restore, degraded startup entry, and recoverable session presentation.
- [ ] Prefer Rust-persisted `app_state`, `layouts`, and `session_metadata` when bootstrapping `AppShellModel`; keep legacy JSON stores as fallback only.
- [ ] Update `LocalAppShellSnapshotSource` and restore helpers so dead sessions appear as archived/recoverable metadata rather than pretending to be live.
- [ ] Propagate degraded restore issues into launcher, Settings, inventory, and attention surfaces using the same recovery codes from Rust.
- [ ] Re-run `swift test --package-path apps/macos --filter AppShellBootstrapTests` and `swift test --package-path apps/macos --filter AppShellSnapshotSourceTests` and expect PASS.
- [ ] Commit with `feat: wire rust-backed restore into app shell`.

---

## Chunk 4: Automated Verification and Defer Tracking

### Task 10: Add automated readiness, compatibility, and degraded suites

**Files:**

- Create: `crates/hc-control-plane/tests/readiness_suite.rs`
- Create: `crates/hc-control-plane/tests/compatibility_suite.rs`
- Create: `crates/hc-control-plane/tests/degraded_suite.rs`
- Modify: `crates/hc-workflow/tests/bootstrap_runtime.rs`
- Modify: `crates/hc-workflow/tests/reload_runtime.rs`
- Modify: `crates/hc-runtime/tests/session_restore.rs`

- [ ] Write failing readiness tests for git, shell, preset binaries, shell markers, workflow presence, Keychain availability, and generic shell fallback.
- [ ] Write optional-tool compatibility smoke tests that skip cleanly when a TUI binary is absent, but fail if an installed binary cannot launch / resize / quit in the expected automated path.
- [ ] Write degraded tests for every Sprint 5 code scenario plus cleanup permission error and secret redaction.
- [ ] Extend workflow tests so invalid reload, last-known-good retention, future retry semantics, and `before_run` failure behavior stay aligned with the Sprint 5 recovery matrix.
- [ ] Run `cargo test -p hc-workflow --test bootstrap_runtime --test reload_runtime`.
- [ ] Run `cargo test -p hc-runtime --test session_restore`.
- [ ] Run `cargo test -p hc-control-plane --test readiness_suite --test compatibility_suite --test degraded_suite`.
- [ ] Commit with `test: add sprint 5 automated regression coverage`.

### Task 11: Full verification pass and explicit defer list

**Files:**

- Modify: `docs/superpowers/specs/sprint-05-inventory-recovery-ship-exit-spec.md`
- Modify: `docs/superpowers/plans/sprint-05-implementation-plan.md`

- [ ] Run `cargo test --workspace` and expect the full Rust workspace to pass.
- [ ] Run `swift test --package-path apps/macos` and expect the Swift package tests to pass.
- [ ] Review `git diff -- docs/superpowers/specs docs/superpowers/plans` and confirm the spec/plan still match actual file boundaries after implementation.
- [ ] Re-state the post-Sprint 5 defer list in both docs: manual TUI runs, screenshots/videos/runbooks, evidence manifest assembly, hosted gate promotion.
- [ ] Commit with `docs: finalize sprint 5 execution and defer scope`.

---

## Verification Commands

```bash
cd /Users/harimkang/develop/applications/haneulchi

cargo test --workspace

cargo test -p hc-domain --test inventory_vocabulary --test settings_vocabulary

cargo test -p hc-storage \
  --test worktree_repository \
  --test cache_repository \
  --test settings_repository \
  --test persistence_repository

cargo test -p hc-runtime --test session_lifecycle --test session_restore

cargo test -p hc-workflow --test bootstrap_runtime --test reload_runtime

cargo test -p hc-control-plane \
  --test inventory_projection \
  --test worktree_provisioning \
  --test persistence_restore \
  --test recovery_flows \
  --test security_redaction \
  --test readiness_suite \
  --test compatibility_suite \
  --test degraded_suite

cargo test -p hc-ffi \
  --test inventory_bridge \
  --test settings_bridge \
  --test recovery_bridge \
  --test runtime_info

cargo test -p hc-api --test server_contract --test security_contract
cargo test -p hc-cli --test cli_parity

swift test --package-path apps/macos
```

---

## Explicit Defer List (Post-Sprint 5)

- Manual TUI operator sign-off (yazi, lazygit, vim/nvim, tmux, IME)
- Screenshots, videos, runbooks, operator checklist JSON, evidence manifest assembly
- Hosted RG-01 ~ RG-10 pass declaration
- Final ship/no-ship decision

이 plan의 완료 조건은 "Sprint 5의 코드 경계와 자동 검증이 준비됨" 이다. 수동 evidence를 채워 release gate를 최종 승격하는 일은 후속 sprint 또는 release-candidate 단계에서 수행한다.

---

## Sprint 5 Execution Status (2026-03-25)

**모든 코드 태스크 완료. 자동 검증 통과.**

| Task | Status | Commit |
|------|--------|--------|
| 1: Domain vocabulary | ✅ Complete | feat: add sprint 5 domain vocabulary |
| 2: Storage schema | ✅ Complete | feat: add sprint 5 storage schema and repositories |
| 3: Env injection | ✅ Complete | feat: support secret env injection in terminal launch |
| 4: Inventory projection | ✅ Complete | feat: add recovery-first inventory projection |
| 5: Settings/persistence/recovery | ✅ Complete | feat: add sprint 5 persistence and degraded recovery services |
| 6: FFI bridges | ✅ Complete | feat: expose sprint 5 projections through ffi and api |
| 7: WF-09 Settings UI | ✅ Complete | feat: connect settings and workflow surfaces to sprint 5 state |
| 8: WF-13 Inventory UI | ✅ Complete | feat: add recovery-first worktree inventory |
| 9: Rust-backed restore | ✅ Complete | feat: wire rust-backed restore into app shell |
| 10: Automated suites | ✅ Complete | test: add sprint 5 automated regression coverage |

**Verification results:**
- `cargo test --workspace`: 0 failures
- `swift test --package-path apps/macos`: 133 tests, 0 failures
