# WF-00 / WF-10 Shell Navigation Checklist

Automated regression run on `2026-03-22`:
- `bash scripts/smoke.sh shell` passed.

Validation status:
- Saved-project route restore:
Automated: `AppShellBootstrapTests.bootstrapRestoresPersistedRoute`
Manual: pending hotkey/visual verification in app
- Shell chrome visibility (`Top App Bar`, `Left Rail`, `Bottom Status Strip`):
Automated: `AppShellChromeStateTests`
Manual: pending visual verification in app
- `Cmd+1..5` primary route switching:
Automated: `RouteTests`, shared shell action coverage
Manual: pending keyboard exercise in app
- `Cmd+Shift+U` latest unread jump:
Automated: `AppShellSnapshotSourceTests`, `AppShellActionTests.jumpToLatestUnreadUsesProjectedAttention`, `CommandPaletteViewModelTests.commandPaletteIncludesLatestUnreadCommand`
Manual: pending keyboard exercise in app
- `Cmd+Shift+P` command palette route command:
Automated: `CommandPaletteViewModelTests`, shell smoke passed
Manual: pending palette interaction in app
- File search visible notice:
Automated: `AppShellActionTests.fileSelectionActionUsesSharedDispatcher`
Manual: pending palette interaction in app
- Task search routes to `Task Board`:
Automated: `AppShellActionTests.createTaskDraftActionCreatesPersistedTask`
Manual: pending palette interaction in app
- Inventory rows come from real restore roots:
Automated: `InventorySearchProjectionStoreTests`, shell smoke passed
Manual: pending palette interaction in app
- `Cmd+,` opens Settings:
Automated: `WelcomeReadinessViewModelTests.openSettingsTargetsTheDocumentedSettingsRoute`
Manual: pending keyboard exercise in app
