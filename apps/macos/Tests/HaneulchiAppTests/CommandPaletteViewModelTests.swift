import Foundation
import Testing
@testable import HaneulchiApp

@Test("command palette catalog groups commands, files, sessions, tasks, and inventory from authoritative and derived queries")
func commandPaletteCatalogUsesSharedState() async throws {
    let snapshot = AppShellSnapshot(
        meta: .init(snapshotRev: 2, runtimeRev: 2, projectionRev: 2, snapshotAt: .now),
        ops: .init(runningSlots: 1, maxSlots: 4, retryQueueCount: 0, workflowHealth: .ok),
        app: .init(activeRoute: .projectFocus, focusedSessionID: "restore-1", degradedFlags: []),
        projects: [
            .init(
                projectID: "proj_demo",
                name: "demo",
                rootPath: "/tmp/demo",
                status: .active,
                workflowState: .ok,
                sessionCount: 1,
                attentionCount: 0
            )
        ],
        sessions: [
            .init(
                sessionID: "restore-1",
                title: "demo",
                currentDirectory: "/tmp/demo",
                mode: .generic,
                runtimeState: .running,
                manualControlState: .none,
                dispatchState: .notDispatchable,
                unreadCount: 0
            )
        ],
        attention: [],
        retryQueue: [],
        warnings: []
    )
    let files = [ProjectFileIndex.Entry(relativePath: "README.md", absolutePath: "/tmp/demo/README.md")]
    let tasks = [
        TaskSearchProjectionStore.Row(
            taskID: "task_01",
            projectID: "proj_demo",
            title: "Wire app shell",
            state: .ready,
            automationMode: .manual,
            linkedSessionID: nil
        )
    ]
    let inventory = [
        InventorySearchProjectionStore.Row(
            itemID: "inv_01",
            title: "demo",
            rootPath: "/tmp/demo",
            kind: .sharedRoot
        )
    ]

    let catalog = CommandPaletteCatalog.build(
        snapshot: snapshot,
        files: files,
        tasks: tasks,
        inventory: inventory
    )

    #expect(catalog.sections.map(\.kind) == [.commands, .files, .sessions, .tasks, .inventory])
    #expect(catalog.sections.first(where: { $0.kind == .files })?.items.first?.title == "README.md")
    #expect(catalog.sections.first(where: { $0.kind == .tasks })?.items.first?.title == "Wire app shell")
    #expect(catalog.sections.first(where: { $0.kind == .inventory })?.items.first?.title == "demo")
}

@MainActor
@Test("palette query filters across sections and returns a shared action intent")
func paletteFiltersAcrossSections() async throws {
    let snapshot = AppShellSnapshot(
        meta: .init(snapshotRev: 2, runtimeRev: 2, projectionRev: 2, snapshotAt: .now),
        ops: .init(runningSlots: 1, maxSlots: 4, retryQueueCount: 0, workflowHealth: .ok),
        app: .init(activeRoute: .projectFocus, focusedSessionID: "restore-1", degradedFlags: []),
        projects: [],
        sessions: [],
        attention: [],
        retryQueue: [],
        warnings: []
    )
    let files = [ProjectFileIndex.Entry(relativePath: "README.md", absolutePath: "/tmp/demo/README.md")]

    let viewModel = CommandPaletteViewModel(
        catalog: .build(snapshot: snapshot, files: files, tasks: [], inventory: [])
    )

    viewModel.query = "read"

    #expect(viewModel.filteredSections.first(where: { $0.kind == .files })?.items.first?.title == "README.md")
    #expect(viewModel.selection?.action == .queueFileSelection("/tmp/demo/README.md"))
}
