import Foundation
@testable import HaneulchiApp
import Testing

@Test(
    "command palette catalog groups commands, files, sessions, tasks, and inventory from authoritative and derived queries",
)
func commandPaletteCatalogUsesSharedState() {
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
                attentionCount: 0,
            ),
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
                unreadCount: 0,
            ),
        ],
        attention: [],
        retryQueue: [],
        warnings: [],
    )
    let files = [ProjectFileIndex.Entry(
        relativePath: "README.md",
        absolutePath: "/tmp/demo/README.md",
    )]
    let tasks = [
        TaskSearchProjectionStore.Row(
            taskID: "task_01",
            projectID: "proj_demo",
            title: "Wire app shell",
            state: .ready,
            automationMode: .manual,
            linkedSessionID: nil,
        ),
    ]
    let inventory = [
        InventorySearchProjectionStore.Row(
            itemID: "inv_01",
            title: "demo",
            rootPath: "/tmp/demo",
            disposition: "in_use",
        ),
    ]

    let catalog = CommandPaletteCatalog.build(
        snapshot: snapshot,
        files: files,
        tasks: tasks,
        inventory: inventory,
    )

    #expect(catalog.sections.map(\.kind) == [.commands, .files, .sessions, .tasks, .inventory])
    #expect(catalog.sections.first(where: { $0.kind == .files })?.items.first?.title == "README.md")
    #expect(catalog.sections.first(where: { $0.kind == .tasks })?.items.first?
        .title == "Wire app shell")
    #expect(catalog.sections.first(where: { $0.kind == .inventory })?.items.first?.title == "demo")
}

@MainActor
@Test("palette query filters across sections and returns a shared action intent")
func paletteFiltersAcrossSections() {
    let snapshot = AppShellSnapshot(
        meta: .init(snapshotRev: 2, runtimeRev: 2, projectionRev: 2, snapshotAt: .now),
        ops: .init(runningSlots: 1, maxSlots: 4, retryQueueCount: 0, workflowHealth: .ok),
        app: .init(activeRoute: .projectFocus, focusedSessionID: "restore-1", degradedFlags: []),
        projects: [],
        sessions: [],
        attention: [],
        retryQueue: [],
        warnings: [],
    )
    let files = [ProjectFileIndex.Entry(
        relativePath: "README.md",
        absolutePath: "/tmp/demo/README.md",
    )]

    let viewModel = CommandPaletteViewModel(
        catalog: .build(snapshot: snapshot, files: files, tasks: [], inventory: []),
    )

    viewModel.query = "read"

    #expect(viewModel.filteredSections.first(where: { $0.kind == .files })?.items.first?
        .title == "README.md")
    #expect(viewModel.selection?.action == .queueFileSelection("/tmp/demo/README.md"))
}

@Test("command palette includes latest unread when the shared snapshot has attention")
func commandPaletteIncludesLatestUnreadCommand() {
    let snapshot = AppShellSnapshot(
        meta: .init(snapshotRev: 2, runtimeRev: 2, projectionRev: 2, snapshotAt: .now),
        ops: .init(runningSlots: 1, maxSlots: 4, retryQueueCount: 0, workflowHealth: .ok),
        app: .init(
            activeRoute: .projectFocus,
            focusedSessionID: "restore-1",
            degradedFlags: [.degraded],
        ),
        projects: [],
        sessions: [],
        attention: [
            .init(
                attentionID: "attention-1",
                headline: "Preset binaries missing",
                severity: .degraded,
                targetRoute: .attentionCenter,
                targetSessionID: nil,
            ),
        ],
        retryQueue: [],
        warnings: [
            .init(
                warningID: "warning-presetBinaries",
                severity: .degraded,
                headline: "Preset binaries missing",
                nextAction: "Open Settings",
            ),
        ],
    )

    let catalog = CommandPaletteCatalog.build(
        snapshot: snapshot,
        files: [],
        tasks: [],
        inventory: [],
    )
    let commandItems = catalog.sections.first(where: { $0.kind == .commands })?.items ?? []

    #expect(commandItems.contains(where: { $0.action == .jumpToLatestUnread }))
}

@Test("command palette exposes workflow and automation operator commands")
func commandPaletteExposesOperatorCommands() {
    let snapshot = AppShellSnapshot(
        meta: .init(snapshotRev: 2, runtimeRev: 2, projectionRev: 2, snapshotAt: .now),
        ops: .init(runningSlots: 1, maxSlots: 4, retryQueueCount: 1, workflowHealth: .ok),
        app: .init(activeRoute: .projectFocus, focusedSessionID: "restore-1", degradedFlags: []),
        projects: [],
        sessions: [],
        attention: [],
        retryQueue: [],
        warnings: [],
    )

    let catalog = CommandPaletteCatalog.build(
        snapshot: snapshot,
        files: [],
        tasks: [],
        inventory: [],
    )
    let commandItems = catalog.sections.first(where: { $0.kind == .commands })?.items ?? []

    #expect(commandItems.contains(where: { $0.title == "Refresh Automation Snapshot" && $0.action == .refreshShellSnapshot }))
    #expect(commandItems.contains(where: { $0.title == "Reconcile Now" && $0.action == .reconcileAutomation }))
    #expect(commandItems.contains(where: { $0.title == "Reload Workflow Contract" && $0.action == .reloadWorkflow }))
    #expect(commandItems.contains(where: { $0.title == "Export State JSON" && $0.action == .exportSnapshot }))
    #expect(commandItems.contains(where: { $0.title == "Open Automation Panel" && $0.action == .openSettings }))
}
