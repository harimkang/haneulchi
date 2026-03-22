import Testing
@testable import HaneulchiApp

@Test("project focus demo boots with one hosted surface and split disabled")
func projectFocusDemoSurfaceContract() {
    let model = ProjectFocusView.Model.demo

    #expect(model.deck.layout.paneIDs.count == 1)
    #expect(model.deck.layout.rootSplitID == nil)
    #expect(model.deck.showsSplitControls == false)
    #expect(model.deck.layout.focusedSurface?.fixtureName == "hello-world.ansi")
}

@Test("runtime project focus model switches the focused surface to live session mode")
func runtimeProjectFocusUsesLiveSurface() {
    let model = ProjectFocusView.Model.runtimeDemo

    #expect(model.deck.layout.focusedSurface?.isLive == true)
    #expect(model.deck.layout.focusedSurface?.fixtureName == nil)
}

@Test("bootstrap project focus model only uses a live restore when one exists")
func bootstrapProjectFocusModelUsesRestoreBundleIfPresent() throws {
    let emptyStore = TerminalSessionRestoreStore.inMemory
    #expect(try ProjectFocusView.Model.bootstrap(restoreStore: emptyStore) == .demo)

    let restoredStore = TerminalSessionRestoreStore.inMemory
    try restoredStore.save([.demo])
    let restored = try ProjectFocusView.Model.bootstrap(restoreStore: restoredStore)

    #expect(restored.deck.layout.focusedSurface?.isLive == true)
}

@Test("selected project without a restore bundle boots a live shell rooted at the project path")
func projectFocusBootstrapFallsBackToSelectedProjectRoot() throws {
    let model = try ProjectFocusView.Model.bootstrap(
        selectedProjectRoot: "/tmp/auth-service",
        restoreStore: .inMemory
    )

    #expect(model.deck.layout.focusedSurface?.liveBundle?.launch.currentDirectory == "/tmp/auth-service")
}

@Test("selected project root overrides a stale restore bundle from another repo")
func selectedProjectRootOverridesRestoreBundle() throws {
    let store = TerminalSessionRestoreStore.inMemory
    try store.save([.genericShell(at: "/tmp/stale-repo")])

    let model = try ProjectFocusView.Model.bootstrap(
        selectedProjectRoot: "/tmp/auth-service",
        restoreStore: store
    )

    #expect(model.deck.layout.focusedSurface?.liveBundle?.launch.currentDirectory == "/tmp/auth-service")
}

@Test("live project focus layouts can retarget focus deterministically")
func liveProjectFocusLayoutCanRetargetFocus() {
    var layout = TerminalDeckLayout.singleLiveDemo
    layout.splitFocusedPane(axis: .vertical)
    let originalPane = layout.paneIDs[0]

    layout.focusPane(originalPane)

    #expect(layout.focusedPaneID == originalPane)
    #expect(layout.focusedSurface?.isLive == true)
}

@Test("session stack rows expose summary, unread, branch, manual continue state, and signal tone")
func sessionStackRowsReflectSnapshotVocabulary() {
    let snapshot = AppShellSnapshot(
        meta: .init(snapshotRev: 1, runtimeRev: 1, projectionRev: 1, snapshotAt: .now),
        ops: .init(runningSlots: 2, maxSlots: 4, retryQueueCount: 0, workflowHealth: .ok),
        app: .init(activeRoute: .projectFocus, focusedSessionID: "ses_02", degradedFlags: []),
        projects: [],
        sessions: [
            .init(
                sessionID: "ses_01",
                title: "Build",
                currentDirectory: "/tmp/demo",
                mode: .generic,
                runtimeState: .running,
                manualControlState: .none,
                dispatchState: .dispatchable,
                unreadCount: 1,
                branch: "main",
                latestSummary: "Running tests",
                focusState: .background,
                canTakeover: false
            ),
            .init(
                sessionID: "ses_02",
                title: "Review",
                currentDirectory: "/tmp/demo",
                mode: .preset,
                runtimeState: .waitingInput,
                manualControlState: .takeover,
                dispatchState: .dispatchable,
                unreadCount: 3,
                branch: "feature/task-104",
                latestSummary: "Awaiting operator answer",
                focusState: .focused,
                canTakeover: true
            ),
        ],
        attention: [],
        retryQueue: [],
        warnings: []
    )

    let rows = SessionStackView.rows(from: snapshot)

    #expect(rows.count == 2)
    #expect(rows[0].summary == "Running tests")
    #expect(rows[0].branch == "main")
    #expect(rows[0].unreadCount == 1)
    #expect(rows[0].signal?.tone == .weak)
    #expect(rows[0].signal?.label == "1 unread")
    #expect(rows[1].isFocused == true)
    #expect(rows[1].signal?.tone == .strong)
    #expect(rows[1].signal?.label == "manual takeover")
    #expect(rows[1].showsManualContinueCTA == true)
}

@Test("inspector resolves the focused session instead of the first session")
func inspectorUsesFocusedSessionContext() {
    let snapshot = AppShellSnapshot(
        meta: .init(snapshotRev: 1, runtimeRev: 1, projectionRev: 1, snapshotAt: .now),
        ops: .init(runningSlots: 2, maxSlots: 4, retryQueueCount: 0, workflowHealth: .ok),
        app: .init(activeRoute: .projectFocus, focusedSessionID: "ses_02", degradedFlags: []),
        projects: [],
        sessions: [
            .init(
                sessionID: "ses_01",
                title: "Build",
                currentDirectory: "/tmp/demo",
                mode: .generic,
                runtimeState: .running,
                manualControlState: .none,
                dispatchState: .dispatchable,
                unreadCount: 1,
                projectID: "proj_demo",
                taskID: "task_01",
                workspaceRoot: "/tmp/demo/.haneulchi/task_01",
                baseRoot: ".",
                branch: "main",
                latestSummary: "Running tests",
                focusState: .background,
                canTakeover: false
            ),
            .init(
                sessionID: "ses_02",
                title: "Review",
                currentDirectory: "/tmp/demo",
                mode: .preset,
                runtimeState: .waitingInput,
                manualControlState: .takeover,
                dispatchState: .dispatchable,
                unreadCount: 3,
                projectID: "proj_demo",
                taskID: "task_02",
                workspaceRoot: "/tmp/demo/.haneulchi/task_02",
                baseRoot: ".",
                branch: "feature/task-104",
                latestSummary: "Awaiting operator answer",
                focusState: .focused,
                canTakeover: true
            ),
        ],
        attention: [],
        retryQueue: [],
        warnings: []
    )

    let focusedSession = InspectorPanelView.focusedSession(from: snapshot)

    #expect(focusedSession?.sessionID == "ses_02")
    #expect(focusedSession?.taskID == "task_02")
}
