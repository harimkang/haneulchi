import Foundation
import Testing
@testable import HaneulchiApp

@Test("inventory groups rows by disposition in correct order: InUse, Recoverable, SafeToDelete, Stale")
func testInventoryGroupsRowsByDisposition() {
    let rows: [WorktreeInventoryViewModel.Row] = [
        .init(worktreeId: "wt-stale", path: "/p/stale", projectName: "proj", branch: "main", disposition: .stale, isPinned: false, isDegraded: false, sizeBytes: nil, lastAccessedAt: nil),
        .init(worktreeId: "wt-safe", path: "/p/safe", projectName: "proj", branch: "main", disposition: .safeToDelete, isPinned: false, isDegraded: false, sizeBytes: nil, lastAccessedAt: nil),
        .init(worktreeId: "wt-recoverable", path: "/p/rec", projectName: "proj", branch: "feat", disposition: .recoverable, isPinned: false, isDegraded: false, sizeBytes: nil, lastAccessedAt: nil),
        .init(worktreeId: "wt-inuse", path: "/p/inuse", projectName: "proj", branch: "main", disposition: .inUse, isPinned: true, isDegraded: false, sizeBytes: nil, lastAccessedAt: nil),
    ]

    let vm = WorktreeInventoryViewModel(rows: rows)

    #expect(vm.inUseRows.count == 1)
    #expect(vm.inUseRows.first?.worktreeId == "wt-inuse")

    #expect(vm.recoverableRows.count == 1)
    #expect(vm.recoverableRows.first?.worktreeId == "wt-recoverable")

    #expect(vm.safeToDeleteRows.count == 1)
    #expect(vm.safeToDeleteRows.first?.worktreeId == "wt-safe")

    #expect(vm.staleRows.count == 1)
    #expect(vm.staleRows.first?.worktreeId == "wt-stale")
}

@Test("summary card shows correct counts for each disposition")
func testInventorySummaryCardValues() {
    let rows: [WorktreeInventoryViewModel.Row] = [
        .init(worktreeId: "wt-1", path: "/p/1", projectName: "p", branch: nil, disposition: .inUse, isPinned: false, isDegraded: false, sizeBytes: nil, lastAccessedAt: nil),
        .init(worktreeId: "wt-2", path: "/p/2", projectName: "p", branch: nil, disposition: .inUse, isPinned: false, isDegraded: false, sizeBytes: nil, lastAccessedAt: nil),
        .init(worktreeId: "wt-3", path: "/p/3", projectName: "p", branch: nil, disposition: .recoverable, isPinned: false, isDegraded: false, sizeBytes: nil, lastAccessedAt: nil),
        .init(worktreeId: "wt-4", path: "/p/4", projectName: "p", branch: nil, disposition: .safeToDelete, isPinned: false, isDegraded: false, sizeBytes: nil, lastAccessedAt: nil),
        .init(worktreeId: "wt-5", path: "/p/5", projectName: "p", branch: nil, disposition: .stale, isPinned: false, isDegraded: false, sizeBytes: nil, lastAccessedAt: nil),
    ]

    let vm = WorktreeInventoryViewModel(rows: rows)
    let card = vm.summaryCard

    #expect(card.total == 5)
    #expect(card.inUse == 2)
    #expect(card.recoverable == 1)
    #expect(card.safeToDelete == 1)
    #expect(card.stale == 1)
}

@Test("InUse row has canOpenFinder true and canOpenSession true")
func testInventoryOpenActionsForInUseRow() {
    let inUseRow = WorktreeInventoryViewModel.Row(
        worktreeId: "wt-inuse",
        taskID: "task-inuse",
        path: "/p/inuse",
        projectName: "proj",
        branch: "main",
        disposition: .inUse,
        isPinned: false,
        isDegraded: false,
        sizeBytes: nil,
        lastAccessedAt: nil
    )

    #expect(inUseRow.canOpenFinder == true)
    #expect(inUseRow.canOpenSession == true)
}

@Test("Stale row has canOpenFinder true and canOpenSession false")
func testInventoryOpenActionsForStaleRow() {
    let staleRow = WorktreeInventoryViewModel.Row(
        worktreeId: "wt-stale",
        path: "/p/stale",
        projectName: "proj",
        branch: nil,
        disposition: .stale,
        isPinned: false,
        isDegraded: false,
        sizeBytes: nil,
        lastAccessedAt: nil
    )

    #expect(staleRow.canOpenFinder == true)
    #expect(staleRow.canOpenSession == false)
}

@Test("empty path row disables Open in Finder")
func testEmptyPathDisablesOpenInFinder() {
    let row = WorktreeInventoryViewModel.Row(
        worktreeId: "wt-1",
        taskID: "",
        path: "",
        projectName: "proj",
        branch: nil,
        disposition: .inUse,
        isPinned: false,
        isDegraded: false,
        sizeBytes: nil,
        lastAccessedAt: nil
    )
    #expect(row.canOpenFinder == false)
}

@Test("Recoverable row with non-empty path has canOpenSession true")
func testRecoverableRowWithPathCanOpenSession() {
    let row = WorktreeInventoryViewModel.Row(
        worktreeId: "wt-rec",
        path: "/p/rec",
        projectName: "proj",
        branch: "feat",
        disposition: .recoverable,
        isPinned: false,
        isDegraded: false,
        sizeBytes: nil,
        lastAccessedAt: nil
    )

    #expect(row.canOpenSession == true)
}

@Test("Recoverable row with empty path has canOpenSession false")
func testRecoverableRowWithEmptyPathCannotOpenSession() {
    let row = WorktreeInventoryViewModel.Row(
        worktreeId: "wt-rec",
        path: "",
        projectName: "proj",
        branch: "feat",
        disposition: .recoverable,
        isPinned: false,
        isDegraded: false,
        sizeBytes: nil,
        lastAccessedAt: nil
    )

    #expect(row.canOpenSession == false)
}

@Test("InUse row has no delete action")
func testInventoryActionGatingPreventsDeleteOfInUseWorktree() {
    let inUseRow = WorktreeInventoryViewModel.Row(
        worktreeId: "wt-inuse",
        path: "/p/inuse",
        projectName: "proj",
        branch: "main",
        disposition: .inUse,
        isPinned: false,
        isDegraded: false,
        sizeBytes: nil,
        lastAccessedAt: nil
    )

    #expect(inUseRow.canDelete == false)
    #expect(inUseRow.canRecover == false)
    #expect(inUseRow.canPin == false)

    let recoverableRow = WorktreeInventoryViewModel.Row(
        worktreeId: "wt-rec",
        path: "/p/rec",
        projectName: "proj",
        branch: "feat",
        disposition: .recoverable,
        isPinned: false,
        isDegraded: false,
        sizeBytes: nil,
        lastAccessedAt: nil
    )

    #expect(recoverableRow.canDelete == false)
    #expect(recoverableRow.canRecover == true)
    #expect(recoverableRow.canPin == true)

    let safeRow = WorktreeInventoryViewModel.Row(
        worktreeId: "wt-safe",
        path: "/p/safe",
        projectName: "proj",
        branch: nil,
        disposition: .safeToDelete,
        isPinned: false,
        isDegraded: false,
        sizeBytes: nil,
        lastAccessedAt: nil
    )

    #expect(safeRow.canDelete == true)
    #expect(safeRow.canRecover == false)
    #expect(safeRow.canPin == true)
}
