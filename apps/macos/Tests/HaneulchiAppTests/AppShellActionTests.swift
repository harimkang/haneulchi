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
    #expect(model.pendingProjectFocusFilePath == "/tmp/demo/README.md")
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

@MainActor
@Test("new session actions present the sheet and launching a preset stores a new restore bundle")
func newSessionActionLaunchesPresetIntoProjectFocus() async throws {
    let restoreStore = TerminalSessionRestoreStore.inMemory
    let project = LauncherProject(
        projectID: "proj_demo",
        name: "demo",
        rootPath: "/tmp/demo",
        lastOpenedAt: .now
    )
    let registry = PresetRegistry(
        presets: [
            .init(
                id: "codex",
                title: "Codex",
                binary: "codex",
                defaultArgs: ["--sandbox", "workspace-write"],
                capabilityFlags: ["agent"],
                requiresShellIntegration: false,
                installState: .installed
            )
        ]
    )
    let model = AppShellModel(
        entrySurface: .shell,
        selectedRoute: .controlTower,
        selectedProject: project,
        recentProjects: [project],
        readinessReport: nil,
        projectStore: .inMemory,
        restoreStore: restoreStore,
        preferencesStore: .inMemory,
        presetRegistry: registry
    )

    await model.perform(.presentNewSessionSheet)

    #expect(model.isNewSessionSheetPresented == true)

    let descriptor = SessionLaunchDescriptor(
        mode: .preset,
        title: "Codex",
        presetID: "codex",
        restoreBundle: .init(
            launch: .init(
                program: "codex",
                args: ["--sandbox", "workspace-write"],
                currentDirectory: "/tmp/demo",
                geometry: .defaultShell
            ),
            geometry: .defaultShell
        ),
        workspaceRoot: nil,
        workflowSummary: nil
    )

    await model.perform(.launchSession(descriptor))

    let savedBundles = try restoreStore.load()
    #expect(model.selectedRoute == .projectFocus)
    #expect(model.isNewSessionSheetPresented == false)
    #expect(savedBundles.last?.launch.program == "codex")
    #expect(savedBundles.last?.launch.currentDirectory == "/tmp/demo")
}

private final class SnapshotBridgeState: @unchecked Sendable {
    private let lock = NSLock()
    private var snapshot: AppShellSnapshot

    init(snapshot: AppShellSnapshot) {
        self.snapshot = snapshot
    }

    func focus(_ sessionID: String) {
        lock.lock()
        defer { lock.unlock() }
        snapshot = AppShellSnapshot(
            meta: snapshot.meta,
            ops: snapshot.ops,
            app: .init(activeRoute: .projectFocus, focusedSessionID: sessionID, degradedFlags: snapshot.app.degradedFlags),
            projects: snapshot.projects,
            sessions: snapshot.sessions.map { session in
                .init(
                    sessionID: session.sessionID,
                    title: session.title,
                    currentDirectory: session.currentDirectory,
                    mode: session.mode,
                    runtimeState: session.runtimeState,
                    manualControlState: session.manualControlState,
                    dispatchState: session.dispatchState,
                    unreadCount: session.unreadCount,
                    projectID: session.projectID,
                    taskID: session.taskID,
                    workspaceRoot: session.workspaceRoot,
                    baseRoot: session.baseRoot,
                    branch: session.branch,
                    latestSummary: session.latestSummary,
                    claimState: session.claimState,
                    adapterKind: session.adapterKind,
                    lastActivityAt: session.lastActivityAt,
                    focusState: session.sessionID == sessionID ? .focused : .background,
                    canFocus: session.canFocus,
                    canTakeover: session.canTakeover,
                    canReleaseTakeover: session.canReleaseTakeover
                )
            },
            attention: snapshot.attention,
            retryQueue: snapshot.retryQueue,
            warnings: snapshot.warnings,
            workflow: snapshot.workflow,
            tracker: snapshot.tracker
        )
    }

    func current() -> AppShellSnapshot {
        lock.lock()
        defer { lock.unlock() }
        return snapshot
    }
}

@MainActor
@Test("jump to session uses bridge focus and refreshed snapshot state")
func jumpToSessionUsesBridgeFocus() async throws {
    let snapshotState = SnapshotBridgeState(
        snapshot: AppShellSnapshot(
            meta: .init(snapshotRev: 1, runtimeRev: 1, projectionRev: 1, snapshotAt: .now),
            ops: .init(runningSlots: 2, maxSlots: 4, retryQueueCount: 0, workflowHealth: .ok),
            app: .init(activeRoute: .projectFocus, focusedSessionID: "ses_01", degradedFlags: []),
            projects: [],
            sessions: [
                .init(
                    sessionID: "ses_01",
                    title: "One",
                    currentDirectory: "/tmp/demo",
                    mode: .generic,
                    runtimeState: .running,
                    manualControlState: .none,
                    dispatchState: .dispatchable,
                    unreadCount: 0,
                    focusState: .focused
                ),
                .init(
                    sessionID: "ses_02",
                    title: "Two",
                    currentDirectory: "/tmp/demo",
                    mode: .generic,
                    runtimeState: .running,
                    manualControlState: .none,
                    dispatchState: .dispatchable,
                    unreadCount: 0,
                    focusState: .background
                ),
            ],
            attention: [],
            retryQueue: [],
            warnings: []
        )
    )
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
        stateSnapshot: { snapshotState.current() },
        sessionsList: { snapshotState.current().sessions },
        focusSession: { sessionID in snapshotState.focus(sessionID) }
    )
    let model = AppShellModel(
        entrySurface: .shell,
        selectedRoute: .controlTower,
        selectedProject: nil,
        recentProjects: [],
        readinessReport: nil,
        projectStore: .inMemory,
        restoreStore: .inMemory,
        preferencesStore: .inMemory,
        coreBridge: bridge
    )

    await model.refreshShellSnapshot()
    await model.perform(.jumpToSession("ses_02"))

    #expect(model.selectedRoute == .projectFocus)
    #expect(model.shellSnapshot?.app.focusedSessionID == "ses_02")
    #expect(model.shellSnapshot?.sessions.first(where: { $0.sessionID == "ses_02" })?.focusState == .focused)
}

@MainActor
@Test("workflow drawer action loads validate payload and reload updates state")
func workflowDrawerUsesBridgeValidateAndReload() async throws {
    let payloads = WorkflowPayloadRecorder(
        validatePayload: Data(
            #"""
            {
              "state": "ok",
              "path": "/tmp/demo/WORKFLOW.md",
              "last_good_hash": "sha256:abc123",
              "last_reload_at": "2026-03-22T00:00:00Z",
              "last_error": null,
              "workflow": {
                "name": "Demo Workflow",
                "strategy": "worktree",
                "base_root": ".",
                "review_checklist": ["tests passed"],
                "allowed_agents": ["codex"],
                "hooks": ["after_create"]
              }
            }
            """#.utf8
        ),
        reloadPayload: Data(
            #"""
            {
              "state": "invalid_kept_last_good",
              "path": "/tmp/demo/WORKFLOW.md",
              "last_good_hash": "sha256:abc123",
              "last_reload_at": "2026-03-22T00:10:00Z",
              "last_error": "front matter parse error",
              "workflow": {
                "name": "Demo Workflow",
                "strategy": "worktree",
                "base_root": ".",
                "review_checklist": ["tests passed"],
                "allowed_agents": ["codex"],
                "hooks": ["after_create"]
              }
            }
            """#.utf8
        )
    )
    let project = LauncherProject(
        projectID: "proj_demo",
        name: "demo",
        rootPath: "/tmp/demo",
        lastOpenedAt: .now
    )
    let bridge = CoreBridge(
        runtimeInfo: { TerminalBackendDescriptor(rendererID: "swiftterm", transport: "ffi_c_abi", demoMode: false) },
        spawnSession: { _ in throw CoreBridgeError.operationFailed("spawn_unused") },
        drainSession: { _ in Data() },
        writeSession: { _, _ in },
        resizeSession: { _, _ in },
        terminateSession: { _ in },
        snapshotSession: { _ in throw CoreBridgeError.operationFailed("snapshot_unused") },
        workflowValidate: { _ in payloads.validatePayload },
        workflowReload: { _ in payloads.reloadPayload }
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
        coreBridge: bridge
    )

    await model.perform(.presentWorkflowDrawer)
    #expect(model.isWorkflowDrawerPresented == true)
    #expect(model.workflowStatus?.state == .ok)

    await model.perform(.reloadWorkflow)
    #expect(model.workflowStatus?.state == .invalidKeptLastGood)
    #expect(model.workflowStatus?.lastError == "front matter parse error")
}

@MainActor
@Test("refresh shell snapshot syncs workflow status from the authoritative snapshot")
func refreshShellSnapshotSynchronizesWorkflowStatus() async throws {
    let project = LauncherProject(
        projectID: "proj_demo",
        name: "demo",
        rootPath: "/tmp/demo",
        lastOpenedAt: .now
    )
    let bridge = CoreBridge(
        runtimeInfo: { TerminalBackendDescriptor(rendererID: "swiftterm", transport: "ffi_c_abi", demoMode: false) },
        spawnSession: { _ in throw CoreBridgeError.operationFailed("spawn_unused") },
        drainSession: { _ in Data() },
        writeSession: { _, _ in },
        resizeSession: { _, _ in },
        terminateSession: { _ in },
        snapshotSession: { _ in throw CoreBridgeError.operationFailed("snapshot_unused") },
        stateSnapshot: {
            AppShellSnapshot(
                meta: .init(snapshotRev: 1, runtimeRev: 1, projectionRev: 1, snapshotAt: .now),
                ops: .init(runningSlots: 0, maxSlots: 1, retryQueueCount: 0, workflowHealth: .reloadPending),
                app: .init(activeRoute: .projectFocus, focusedSessionID: nil, degradedFlags: []),
                projects: [],
                sessions: [],
                attention: [],
                retryQueue: [],
                warnings: [],
                workflow: .init(
                    state: .reloadPending,
                    path: "/tmp/demo/WORKFLOW.md",
                    lastGoodHash: "sha256:abc123",
                    lastReloadAt: "2026-03-23T00:00:00Z",
                    lastError: nil
                ),
                tracker: .init(state: "local_only", lastSyncAt: nil, health: "ok")
            )
        }
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
        coreBridge: bridge
    )

    await model.refreshShellSnapshot()

    #expect(model.workflowStatus?.state == .reloadPending)
    #expect(model.workflowStatus?.lastGoodHash == "sha256:abc123")
}

@MainActor
@Test("refresh shell snapshot clears stale workflow status when the selected project has no workflow")
func refreshShellSnapshotClearsWorkflowStatusWhenWorkflowIsMissing() async throws {
    let project = LauncherProject(
        projectID: "proj_no_workflow",
        name: "demo",
        rootPath: "/tmp/no-workflow",
        lastOpenedAt: .now
    )
    let bridge = CoreBridge(
        runtimeInfo: { TerminalBackendDescriptor(rendererID: "swiftterm", transport: "ffi_c_abi", demoMode: false) },
        spawnSession: { _ in throw CoreBridgeError.operationFailed("spawn_unused") },
        drainSession: { _ in Data() },
        writeSession: { _, _ in },
        resizeSession: { _, _ in },
        terminateSession: { _ in },
        snapshotSession: { _ in throw CoreBridgeError.operationFailed("snapshot_unused") },
        stateSnapshot: {
            AppShellSnapshot(
                meta: .init(snapshotRev: 1, runtimeRev: 1, projectionRev: 1, snapshotAt: .now),
                ops: .init(runningSlots: 0, maxSlots: 1, retryQueueCount: 0, workflowHealth: .none),
                app: .init(activeRoute: .projectFocus, focusedSessionID: nil, degradedFlags: []),
                projects: [],
                sessions: [],
                attention: [],
                retryQueue: [],
                warnings: [],
                workflow: nil,
                tracker: .init(state: "local_only", lastSyncAt: nil, health: "ok")
            )
        },
        workflowValidate: { _ in Data(#"{"state":"none"}"#.utf8) }
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
        coreBridge: bridge,
        workflowStatus: .init(
            state: .ok,
            path: "/tmp/old/WORKFLOW.md",
            lastGoodHash: "sha256:old",
            lastReloadAt: "2026-03-23T00:00:00Z",
            lastError: nil,
            workflow: .init(
                name: "Old Workflow",
                strategy: "worktree",
                baseRoot: ".",
                reviewChecklist: ["old"],
                allowedAgents: ["codex"],
                hooks: [],
                hookRuns: [:],
                templateBody: nil
            )
        )
    )

    await model.refreshShellSnapshot()

    #expect(model.workflowStatus == nil)
}

@MainActor
@Test("task context drawer reuses workflow summary for the focused task")
func taskContextDrawerUsesWorkflowSummary() async throws {
    let project = LauncherProject(
        projectID: "proj_demo",
        name: "demo",
        rootPath: "/tmp/demo",
        lastOpenedAt: .now
    )
    let bridge = CoreBridge(
        runtimeInfo: { TerminalBackendDescriptor(rendererID: "swiftterm", transport: "ffi_c_abi", demoMode: false) },
        spawnSession: { _ in throw CoreBridgeError.operationFailed("spawn_unused") },
        drainSession: { _ in Data() },
        writeSession: { _, _ in },
        resizeSession: { _, _ in },
        terminateSession: { _ in },
        snapshotSession: { _ in throw CoreBridgeError.operationFailed("snapshot_unused") },
        stateSnapshot: {
            AppShellSnapshot(
                meta: .init(snapshotRev: 1, runtimeRev: 1, projectionRev: 1, snapshotAt: .now),
                ops: .init(runningSlots: 1, maxSlots: 2, retryQueueCount: 0, workflowHealth: .ok),
                app: .init(activeRoute: .projectFocus, focusedSessionID: "ses_01", degradedFlags: []),
                projects: [],
                sessions: [
                    .init(
                        sessionID: "ses_01",
                        title: "Review",
                        currentDirectory: "/tmp/demo",
                        mode: .preset,
                        runtimeState: .waitingInput,
                        manualControlState: .takeover,
                        dispatchState: .dispatchable,
                        unreadCount: 1,
                        projectID: "proj_demo",
                        taskID: "task_104",
                        workspaceRoot: "/tmp/demo/.haneulchi/isolated/task-104",
                        baseRoot: ".",
                        branch: "feature/task-104",
                        latestSummary: "Awaiting operator answer",
                        focusState: .focused,
                        canTakeover: true
                    ),
                ],
                attention: [],
                retryQueue: [],
                warnings: [],
                workflow: .init(
                    state: .ok,
                    path: "/tmp/demo/WORKFLOW.md",
                    lastGoodHash: "sha256:abc123",
                    lastReloadAt: "2026-03-23T00:00:00Z",
                    lastError: nil
                ),
                tracker: .init(state: "local_only", lastSyncAt: nil, health: "ok")
            )
        },
        workflowValidate: { _ in
            Data(
                #"""
                {
                  "state": "ok",
                  "path": "/tmp/demo/WORKFLOW.md",
                  "last_good_hash": "sha256:abc123",
                  "last_reload_at": "2026-03-23T00:00:00Z",
                  "last_error": null,
                  "workflow": {
                    "name": "Demo Workflow",
                    "strategy": "worktree",
                    "base_root": ".",
                    "review_checklist": ["tests passed", "diff reviewed"],
                    "allowed_agents": ["codex", "claude"],
                    "hooks": ["after_create"]
                  }
                }
                """#.utf8
            )
        }
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
        coreBridge: bridge
    )

    await model.refreshShellSnapshot()
    await model.perform(.presentTaskContextDrawer)

    #expect(model.isTaskContextDrawerPresented == true)
    #expect(model.taskContextDrawerModel?.taskID == "task_104")
    #expect(model.taskContextDrawerModel?.workflowName == "Demo Workflow")
    #expect(model.taskContextDrawerModel?.allowedAgents == ["codex", "claude"])
}

@MainActor
@Test("open settings also loads the full workflow summary for the settings surface")
func openSettingsLoadsWorkflowSummary() async throws {
    let project = LauncherProject(
        projectID: "proj_demo",
        name: "demo",
        rootPath: "/tmp/demo",
        lastOpenedAt: .now
    )
    let bridge = CoreBridge(
        runtimeInfo: { TerminalBackendDescriptor(rendererID: "swiftterm", transport: "ffi_c_abi", demoMode: false) },
        spawnSession: { _ in throw CoreBridgeError.operationFailed("spawn_unused") },
        drainSession: { _ in Data() },
        writeSession: { _, _ in },
        resizeSession: { _, _ in },
        terminateSession: { _ in },
        snapshotSession: { _ in throw CoreBridgeError.operationFailed("snapshot_unused") },
        workflowValidate: { _ in
            Data(
                #"""
                {
                  "state": "ok",
                  "path": "/tmp/demo/WORKFLOW.md",
                  "last_good_hash": "sha256:abc123",
                  "last_reload_at": null,
                  "last_error": null,
                  "workflow": {
                    "name": "Demo Workflow",
                    "strategy": "worktree",
                    "base_root": ".",
                    "review_checklist": ["tests passed"],
                    "allowed_agents": ["codex", "claude"],
                    "hooks": ["after_create", "before_run"]
                  }
                }
                """#.utf8
            )
        }
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
        coreBridge: bridge
    )

    await model.perform(.openSettings)

    #expect(model.selectedRoute == .settings)
    #expect(model.workflowStatus?.workflow?.name == "Demo Workflow")
    #expect(model.workflowStatus?.workflow?.allowedAgents == ["codex", "claude"])
    #expect(model.settingsStatusViewModel?.workflowRow?.title == "Demo Workflow")
    #expect(model.settingsStatusViewModel?.automationRows.first(where: { $0.id == .localAPI })?.statusLabel == "available")
}

@MainActor
@Test("isolated launch materializes workspace, executes hooks, and writes a rendered prompt artifact")
func isolatedLaunchBootstrapsWorkflowArtifacts() async throws {
    let projectRoot = FileManager.default.temporaryDirectory
        .appendingPathComponent("isolated-launch-\(UUID().uuidString)", isDirectory: true)
    try FileManager.default.createDirectory(at: projectRoot, withIntermediateDirectories: true)

    let afterCreate = projectRoot.appendingPathComponent("after-create.sh")
    try """
    #!/bin/sh
    echo after-create > "$PWD/after-create.txt"
    """.write(to: afterCreate, atomically: true, encoding: .utf8)
    try FileManager.default.setAttributes([.posixPermissions: 0o755], ofItemAtPath: afterCreate.path)

    let beforeRun = projectRoot.appendingPathComponent("before-run.sh")
    try """
    #!/bin/sh
    echo before-run > "$PWD/before-run.txt"
    """.write(to: beforeRun, atomically: true, encoding: .utf8)
    try FileManager.default.setAttributes([.posixPermissions: 0o755], ofItemAtPath: beforeRun.path)

    let project = LauncherProject(
        projectID: "proj_demo",
        name: "demo",
        rootPath: projectRoot.path,
        lastOpenedAt: .now
    )
    let bridge = CoreBridge(
        runtimeInfo: { TerminalBackendDescriptor(rendererID: "swiftterm", transport: "ffi_c_abi", demoMode: false) },
        spawnSession: { _ in throw CoreBridgeError.operationFailed("spawn_unused") },
        drainSession: { _ in Data() },
        writeSession: { _, _ in },
        resizeSession: { _, _ in },
        terminateSession: { _ in },
        snapshotSession: { _ in throw CoreBridgeError.operationFailed("snapshot_unused") },
        workflowValidate: { _ in
            Data(
                #"""
                {
                  "state": "ok",
                  "path": "/tmp/demo/WORKFLOW.md",
                  "last_good_hash": "sha256:abc123",
                  "last_reload_at": null,
                  "last_error": null,
                  "workflow": {
                    "name": "Demo Workflow",
                    "strategy": "worktree",
                    "base_root": ".",
                    "review_checklist": ["tests passed"],
                    "allowed_agents": ["codex"],
                    "hooks": ["after_create", "before_run"],
                    "hook_runs": {
                      "after_create": "\#(afterCreate.path)",
                      "before_run": "\#(beforeRun.path)"
                    },
                    "template_body": "Project: {{project.name}}"
                  }
                }
                """#.utf8
            )
        }
    )
    let restoreStore = TerminalSessionRestoreStore.inMemory
    let model = AppShellModel(
        entrySurface: .shell,
        selectedRoute: .projectFocus,
        selectedProject: project,
        recentProjects: [project],
        readinessReport: nil,
        projectStore: .inMemory,
        restoreStore: restoreStore,
        preferencesStore: .inMemory,
        coreBridge: bridge
    )

    let isolatedRoot = projectRoot
        .appendingPathComponent(".haneulchi/isolated/task-104", isDirectory: true)
        .path
    let descriptor = SessionLaunchDescriptor(
        mode: .isolated,
        title: "task-104",
        presetID: nil,
        restoreBundle: .genericShell(at: isolatedRoot),
        workspaceRoot: isolatedRoot,
        workflowSummary: WorkflowLaunchSummary(
            name: "Demo Workflow",
            strategy: "worktree",
            baseRoot: ".",
            reviewChecklist: ["tests passed"],
            allowedAgents: ["codex"]
        )
    )

    await model.perform(.launchSession(descriptor))

    let savedBundles = try restoreStore.load()
    let renderedPrompt = isolatedRoot + "/prompt.rendered.md"
    #expect(FileManager.default.fileExists(atPath: isolatedRoot))
    #expect(FileManager.default.fileExists(atPath: isolatedRoot + "/after-create.txt"))
    #expect(FileManager.default.fileExists(atPath: isolatedRoot + "/before-run.txt"))
    #expect(FileManager.default.fileExists(atPath: renderedPrompt))
    #expect(try String(contentsOfFile: renderedPrompt, encoding: .utf8).contains("Project: demo") == true)
    #expect(savedBundles.last?.launch.currentDirectory == isolatedRoot)
}

private final class WorkflowPayloadRecorder: @unchecked Sendable {
    let validatePayload: Data
    let reloadPayload: Data

    init(validatePayload: Data, reloadPayload: Data) {
        self.validatePayload = validatePayload
        self.reloadPayload = reloadPayload
    }
}
