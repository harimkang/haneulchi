import Foundation
import Testing
@testable import HaneulchiApp

@MainActor
@Test("file selection intent routes back to project focus and records a visible shell notice")
func fileSelectionActionUsesSharedDispatcher() async throws {
    let restoreStore = TerminalSessionRestoreStore.inMemory
    let project = LauncherProject(
        projectID: "proj_demo",
        name: "demo",
        rootPath: "/tmp/demo",
        lastOpenedAt: .now
    )

    let model = AppShellModel(
        entrySurface: .shell,
        selectedRoute: .controlTower,
        selectedProject: project,
        recentProjects: [project],
        readinessReport: nil,
        projectStore: .inMemory,
        restoreStore: restoreStore,
        preferencesStore: .inMemory
    )

    await model.perform(.queueFileSelection("/tmp/demo/README.md"))

    #expect(model.selectedRoute == .projectFocus)
    #expect(model.transientNotice?.contains("README.md") == true)
    #expect(model.shellSnapshot?.app.activeRoute == .projectFocus)
}

@MainActor
@Test("palette presentation is shared state owned by the shell model")
func toggleCommandPaletteOwnsPresentationState() async throws {
    let model = AppShellModel(
        entrySurface: .shell,
        selectedRoute: .projectFocus,
        selectedProject: nil,
        recentProjects: [],
        readinessReport: nil,
        projectStore: .inMemory,
        restoreStore: .inMemory,
        preferencesStore: .inMemory
    )

    #expect(model.isCommandPalettePresented == false)

    await model.perform(.toggleCommandPalette)
    #expect(model.isCommandPalettePresented == true)

    await model.perform(.dismissCommandPalette)
    #expect(model.isCommandPalettePresented == false)
}

@MainActor
@Test("create task draft action writes a real row to the task projection store")
func createTaskDraftActionCreatesPersistedTask() async throws {
    let taskStore = TaskSearchProjectionStore.inMemory
    let project = LauncherProject(
        projectID: "proj_demo",
        name: "demo",
        rootPath: "/tmp/demo",
        lastOpenedAt: .now
    )

    let model = AppShellModel(
        entrySurface: .shell,
        selectedRoute: .projectFocus,
        selectedProject: project,
        recentProjects: [project],
        readinessReport: nil,
        projectStore: .inMemory,
        restoreStore: .inMemory,
        preferencesStore: .inMemory,
        taskSearchProjectionStore: taskStore
    )

    await model.perform(.createTaskDraft("Wire app shell"))

    let rows = try taskStore.search("wire")

    #expect(rows.count == 1)
    #expect(rows.first?.title == "Wire app shell")
    #expect(model.selectedRoute == .taskBoard)
}

@MainActor
@Test("create task draft failure keeps the current route and reports a failure notice")
func createTaskDraftFailureDoesNotReportSuccess() async throws {
    let project = LauncherProject(
        projectID: "proj_demo",
        name: "demo",
        rootPath: "/tmp/demo",
        lastOpenedAt: .now
    )
    let failingStore = TaskSearchProjectionStore(
        search: { _ in [] },
        upsert: { _ in },
        createDraft: { _, _ in
            struct DraftFailure: Error {}
            throw DraftFailure()
        }
    )

    let model = AppShellModel(
        entrySurface: .shell,
        selectedRoute: .controlTower,
        selectedProject: project,
        recentProjects: [project],
        readinessReport: nil,
        projectStore: .inMemory,
        restoreStore: .inMemory,
        preferencesStore: .inMemory,
        taskSearchProjectionStore: failingStore
    )

    await model.perform(.createTaskDraft("Wire app shell"))

    #expect(model.selectedRoute == .controlTower)
    #expect(model.transientNotice?.contains("could not be created") == true)
}

@MainActor
@Test("latest unread uses projected attention from the shared shell snapshot")
func jumpToLatestUnreadUsesProjectedAttention() async throws {
    let project = LauncherProject(
        projectID: "proj_demo",
        name: "demo",
        rootPath: "/tmp/demo",
        lastOpenedAt: .now
    )
    let report = ReadinessReport(
        project: project,
        checks: [
            .init(name: .shell, status: .ready, headline: "Shell ready", detail: "/bin/zsh", nextAction: nil),
            .init(name: .presetBinaries, status: .degraded, headline: "Preset binaries missing", detail: "Generic shell remains available.", nextAction: "Open Settings"),
        ]
    )
    let model = AppShellModel(
        entrySurface: .shell,
        selectedRoute: .projectFocus,
        selectedProject: project,
        recentProjects: [project],
        readinessReport: report,
        projectStore: .inMemory,
        restoreStore: .inMemory,
        preferencesStore: .inMemory
    )

    await model.refreshShellSnapshot()
    await model.perform(.jumpToLatestUnread)

    #expect(model.selectedRoute == .attentionCenter)
    #expect(model.transientNotice?.contains("Preset binaries missing") == true)
}
