import Foundation
import Testing
@testable import HaneulchiApp

private final class SendableBox<T>: @unchecked Sendable {
    var value: T

    init(_ value: T) {
        self.value = value
    }
}

@Test("app shell shows welcome launcher until a project is selected")
@MainActor
func appShellBootstrapsLauncherWhenNoProjectExists() throws {
    let store = ProjectLauncherStore.inMemory
    let model = try AppShellModel.bootstrap(
        projectStore: store,
        restoreStore: .inMemory,
        preferencesStore: .inMemory
    )

    #expect(model.entrySurface == .welcome(.firstRun))
}

@Test("saved last project with a blocked shell re-enters the launcher in recovery mode")
@MainActor
func appShellBootstrapsLauncherForDegradedRecovery() async throws {
    let store = ProjectLauncherStore.inMemory
    let project = LauncherProject(
        projectID: "proj_demo",
        name: "demo",
        rootPath: "/tmp/demo",
        lastOpenedAt: .now
    )
    try store.recordOpen(project)
    try store.saveLastSelectedProject(project)

    let report = ReadinessReport(
        project: project,
        checks: [
            .init(name: .shell, status: .blocked, headline: "Shell unavailable", detail: "Configured shell could not be determined.", nextAction: "Open Settings"),
            .init(name: .workflow, status: .ready, headline: "Workflow contract detected", detail: "WORKFLOW.md is present.", nextAction: nil),
        ]
    )

    let model = try await AppShellModel.bootstrap(
        projectStore: store,
        restoreStore: .inMemory,
        preferencesStore: .inMemory,
        initialReport: report
    )

    #expect(model.entrySurface == .welcome(.degradedRecovery))
}

@Test("saved last project with informational readiness gaps stays in shell")
@MainActor
func appShellBootstrapsShellForInformationalReadinessGaps() async throws {
    let store = ProjectLauncherStore.inMemory
    let project = LauncherProject(
        projectID: "proj_demo",
        name: "demo",
        rootPath: "/tmp/demo",
        lastOpenedAt: .now
    )
    try store.recordOpen(project)
    try store.saveLastSelectedProject(project)

    let report = ReadinessReport(
        project: project,
        checks: [
            .init(name: .shell, status: .ready, headline: "Shell ready", detail: "zsh available", nextAction: nil),
            .init(name: .shellIntegration, status: .degraded, headline: "Shell integration not installed", detail: "Command markers are not configured yet.", nextAction: "Open Settings"),
            .init(name: .workflow, status: .degraded, headline: "Workflow contract not found", detail: "Future launches can still use a generic shell.", nextAction: "Continue with Generic Shell"),
        ]
    )

    let model = try await AppShellModel.bootstrap(
        projectStore: store,
        restoreStore: .inMemory,
        preferencesStore: .inMemory,
        initialReport: report
    )

    #expect(model.entrySurface == .shell)
}

@Test("live default keeps shell entry for a previously selected project when gaps are informational")
@MainActor
func liveDefaultEvaluatesStartupReadiness() async throws {
    let store = ProjectLauncherStore.inMemory
    let project = LauncherProject(
        projectID: "proj_demo",
        name: "demo",
        rootPath: "/tmp/demo",
        lastOpenedAt: .now
    )
    try store.recordOpen(project)
    try store.saveLastSelectedProject(project)

    let runner = ReadinessProbeRunner(
        processClient: .mock(
            shellPath: "/bin/zsh",
            commands: [
                "which git": .success("/opt/homebrew/bin/git\n"),
                "which claude": .failure("missing"),
                "which codex": .failure("missing"),
                "which gemini": .failure("missing"),
                "which yazi": .failure("missing"),
                "which lazygit": .failure("missing"),
                "which git [shell:/bin/zsh]": .success("/opt/homebrew/bin/git\n"),
            ]
        ),
        keychainClient: .mock(isAvailable: true),
        workflowProbe: .mock(.none)
    )

    let model = AppShellModel.liveDefault(
        projectStore: store,
        restoreStore: .inMemory,
        preferencesStore: .inMemory,
        readinessRunner: runner
    )

    await model.startupReadinessTask?.value

    #expect(model.entrySurface == .shell)
    #expect(model.readinessReport?.requiresRecoverySurface == false)
}

@Test("bootstrapped shell restores the last active route when a project exists")
@MainActor
func bootstrapRestoresPersistedRoute() throws {
    let store = ProjectLauncherStore.inMemory
    let preferences = AppShellPreferencesStore.inMemory
    let project = LauncherProject(
        projectID: "proj_demo",
        name: "demo",
        rootPath: "/tmp/demo",
        lastOpenedAt: .now
    )
    try store.recordOpen(project)
    try store.saveLastSelectedProject(project)
    try preferences.save(.init(lastActiveRoute: .controlTower))

    let model = try AppShellModel.bootstrap(
        projectStore: store,
        restoreStore: .inMemory,
        preferencesStore: preferences
    )

    #expect(model.selectedRoute == .controlTower)
}

@MainActor
@Test("live default can activate the core bridge snapshot path")
func liveDefaultCanUseCoreBridgeSnapshot() async throws {
    let store = ProjectLauncherStore.inMemory
    let project = LauncherProject(
        projectID: "proj_demo",
        name: "demo",
        rootPath: "/tmp/demo",
        lastOpenedAt: .now
    )
    try store.recordOpen(project)
    try store.saveLastSelectedProject(project)

    let bridge = CoreBridge(
        runtimeInfo: {
            TerminalBackendDescriptor(rendererID: "swiftterm", transport: "ffi_c_abi", demoMode: false)
        },
        spawnSession: { _ in throw CoreBridgeError.operationFailed("spawn_unused") },
        drainSession: { _ in Data() },
        writeSession: { _, _ in },
        resizeSession: { _, _ in },
        terminateSession: { _ in },
        snapshotSession: { _ in throw CoreBridgeError.operationFailed("snapshot_unused") },
        stateSnapshot: {
            AppShellSnapshot(
                meta: .init(snapshotRev: 2, runtimeRev: 2, projectionRev: 2, snapshotAt: .now),
                ops: .init(runningSlots: 1, maxSlots: 4, retryQueueCount: 0, workflowHealth: .ok),
                app: .init(activeRoute: .projectFocus, focusedSessionID: "bridge-session", degradedFlags: []),
                projects: [],
                sessions: [
                    .init(
                        sessionID: "bridge-session",
                        title: "Bridge Session",
                        currentDirectory: "/tmp/demo",
                        mode: .generic,
                        runtimeState: .running,
                        manualControlState: .none,
                        dispatchState: .dispatchable,
                        unreadCount: 0,
                        focusState: .focused
                    )
                ],
                attention: [],
                retryQueue: [],
                warnings: []
            )
        }
    )

    let model = AppShellModel.liveDefault(
        projectStore: store,
        restoreStore: .inMemory,
        preferencesStore: .inMemory,
        coreBridge: bridge
    )

    await model.refreshShellSnapshot()

    #expect(model.shellSnapshot?.app.focusedSessionID == "bridge-session")
    #expect(model.shellSnapshot?.sessions.first?.sessionID == "bridge-session")
}

@MainActor
@Test("app shell can request local control server startup through the bridge")
func appShellRequestsLocalControlServerStartup() async throws {
    let started = SendableBox<[Bool]>([])
    let bridge = CoreBridge(
        runtimeInfo: {
            TerminalBackendDescriptor(rendererID: "swiftterm", transport: "ffi_c_abi", demoMode: false)
        },
        spawnSession: { _ in throw CoreBridgeError.operationFailed("spawn_unused") },
        drainSession: { _ in Data() },
        writeSession: { _, _ in },
        resizeSession: { _, _ in },
        terminateSession: { _ in },
        snapshotSession: { _ in throw CoreBridgeError.operationFailed("snapshot_unused") },
        startLocalControlServer: { started.value.append(true) }
    )
    let model = AppShellModel(
        entrySurface: .shell,
        selectedRoute: .projectFocus,
        selectedProject: nil,
        recentProjects: [],
        readinessReport: nil,
        projectStore: .inMemory,
        restoreStore: .inMemory,
        preferencesStore: .inMemory,
        coreBridge: bridge
    )

    await model.startLocalControlServerIfNeeded()

    #expect(started.value == [true])
}

@MainActor
@Test("local control server startup surfaces socket ownership diagnostics")
func appShellSurfacesLocalControlServerStartupFailure() async throws {
    let bridge = CoreBridge(
        runtimeInfo: {
            TerminalBackendDescriptor(rendererID: "swiftterm", transport: "ffi_c_abi", demoMode: false)
        },
        spawnSession: { _ in throw CoreBridgeError.operationFailed("spawn_unused") },
        drainSession: { _ in Data() },
        writeSession: { _, _ in },
        resizeSession: { _, _ in },
        terminateSession: { _ in },
        snapshotSession: { _ in throw CoreBridgeError.operationFailed("snapshot_unused") },
        startLocalControlServer: {
            throw CoreBridgeError.operationFailed("socket_already_owned:/tmp/haneulchi/control.sock")
        }
    )
    let model = AppShellModel(
        entrySurface: .shell,
        selectedRoute: .projectFocus,
        selectedProject: nil,
        recentProjects: [],
        readinessReport: nil,
        projectStore: .inMemory,
        restoreStore: .inMemory,
        preferencesStore: .inMemory,
        coreBridge: bridge
    )

    await model.startLocalControlServerIfNeeded()

    #expect(model.transientNotice?.contains("Local control API") == true)
    #expect(model.transientNotice?.contains("/tmp/haneulchi/control.sock") == true)
}

@MainActor
@Test("bootstrap prefers Rust-persisted route over JSON preferences when bridge returns a state")
func testBootstrapPrefersRustPersistedRoute() throws {
    let store = ProjectLauncherStore.inMemory
    let project = LauncherProject(
        projectID: "proj_demo",
        name: "demo",
        rootPath: "/tmp/demo",
        lastOpenedAt: .now
    )
    try store.recordOpen(project)
    try store.saveLastSelectedProject(project)

    // JSON preferences say controlTower
    let preferences = AppShellPreferencesStore.inMemory
    try preferences.save(.init(lastActiveRoute: .controlTower))

    // But the Rust bridge returns taskBoard
    let bridge = CoreBridge(
        runtimeInfo: {
            TerminalBackendDescriptor(rendererID: "swiftterm", transport: "ffi_c_abi", demoMode: false)
        },
        spawnSession: { _ in throw CoreBridgeError.operationFailed("spawn_unused") },
        drainSession: { _ in Data() },
        writeSession: { _, _ in },
        resizeSession: { _, _ in },
        terminateSession: { _ in },
        snapshotSession: { _ in throw CoreBridgeError.operationFailed("snapshot_unused") },
        loadAppState: {
            AppStatePayload(activeRoute: "task_board", lastProjectId: "proj_demo", lastSessionId: nil, savedAt: nil)
        }
    )

    let model = try AppShellModel.bootstrap(
        projectStore: store,
        restoreStore: .inMemory,
        preferencesStore: preferences,
        coreBridge: bridge
    )

    #expect(model.selectedRoute == .taskBoard)
}

@MainActor
@Test("bootstrap falls back to JSON preferences when Rust state is absent")
func testBootstrapFallsBackToJSONPreferencesWhenRustStateAbsent() throws {
    let store = ProjectLauncherStore.inMemory
    let project = LauncherProject(
        projectID: "proj_demo",
        name: "demo",
        rootPath: "/tmp/demo",
        lastOpenedAt: .now
    )
    try store.recordOpen(project)
    try store.saveLastSelectedProject(project)

    let preferences = AppShellPreferencesStore.inMemory
    try preferences.save(.init(lastActiveRoute: .controlTower))

    // Bridge returns nil — no Rust state
    let bridge = CoreBridge(
        runtimeInfo: {
            TerminalBackendDescriptor(rendererID: "swiftterm", transport: "ffi_c_abi", demoMode: false)
        },
        spawnSession: { _ in throw CoreBridgeError.operationFailed("spawn_unused") },
        drainSession: { _ in Data() },
        writeSession: { _, _ in },
        resizeSession: { _, _ in },
        terminateSession: { _ in },
        snapshotSession: { _ in throw CoreBridgeError.operationFailed("snapshot_unused") },
        loadAppState: { nil }
    )

    let model = try AppShellModel.bootstrap(
        projectStore: store,
        restoreStore: .inMemory,
        preferencesStore: preferences,
        coreBridge: bridge
    )

    #expect(model.selectedRoute == .controlTower)
}
