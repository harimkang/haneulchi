@testable import HaneulchiApp
import Testing

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
        restoreStore: .inMemory,
    )

    #expect(model.deck.layout.focusedSurface?.liveBundle?.launch
        .currentDirectory == "/tmp/auth-service")
}

@Test("selected project root overrides a stale restore bundle from another repo")
func selectedProjectRootOverridesRestoreBundle() throws {
    let store = TerminalSessionRestoreStore.inMemory
    try store.save([.genericShell(at: "/tmp/stale-repo")])

    let model = try ProjectFocusView.Model.bootstrap(
        selectedProjectRoot: "/tmp/auth-service",
        restoreStore: store,
    )

    #expect(model.deck.layout.focusedSurface?.liveBundle?.launch
        .currentDirectory == "/tmp/auth-service")
}

@Test(
    "project focus bootstrap falls back to recoverable session metadata when no restore bundle exists",
)
func projectFocusBootstrapUsesRecoverableSessions() throws {
    let model = try ProjectFocusView.Model.bootstrap(
        selectedProjectRoot: "/tmp/demo",
        restoreStore: .inMemory,
        recoverableSessions: [
            .init(
                sessionId: "ses_recover_01",
                projectId: "proj_demo",
                title: "Recoverable shell",
                cwd: "/tmp/demo/worktrees/task-104",
                branch: "hc/task-104",
                lastActiveAt: "2026-03-25T00:00:00Z",
                isRecoverable: true,
            ),
        ],
    )

    #expect(model.deck.layout.focusedSurface?.liveBundle?.launch
        .currentDirectory == "/tmp/demo/worktrees/task-104")
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
                canTakeover: false,
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
                canTakeover: true,
            ),
        ],
        attention: [],
        retryQueue: [],
        warnings: [],
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
    #expect(rows[1].signal?.badgeState == .manualTakeover)
    #expect(rows[1].showsManualContinueCTA == true)
}

@Test("session signals preserve semantic badge states for review and blocked flows")
func sessionSignalsExposeSemanticBadgeStates() {
    let reviewReady = AppShellSnapshot.SessionSummary(
        sessionID: "ses_review",
        title: "Review",
        currentDirectory: "/tmp/demo",
        mode: .structuredAdapter,
        runtimeState: .reviewReady,
        manualControlState: .none,
        dispatchState: .dispatchable,
        unreadCount: 0,
        focusState: .background,
    )
    let waitingInput = AppShellSnapshot.SessionSummary(
        sessionID: "ses_waiting",
        title: "Needs input",
        currentDirectory: "/tmp/demo",
        mode: .structuredAdapter,
        runtimeState: .waitingInput,
        manualControlState: .none,
        dispatchState: .dispatchable,
        unreadCount: 0,
        focusState: .background,
    )
    let blocked = AppShellSnapshot.SessionSummary(
        sessionID: "ses_blocked",
        title: "Blocked",
        currentDirectory: "/tmp/demo",
        mode: .structuredAdapter,
        runtimeState: .blocked,
        manualControlState: .none,
        dispatchState: .notDispatchable,
        unreadCount: 0,
        focusState: .background,
    )

    #expect(SessionSignalPresentation.from(session: reviewReady, isFocused: false)?
        .badgeState == .reviewReady)
    #expect(SessionSignalPresentation.from(session: waitingInput, isFocused: false)?
        .badgeState == .waitingInput)
    #expect(SessionSignalPresentation.from(session: blocked, isFocused: false)?
        .badgeState == .blocked)
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
                canTakeover: false,
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
                canTakeover: true,
            ),
        ],
        attention: [],
        retryQueue: [],
        warnings: [],
    )

    let focusedSession = InspectorPanelView.focusedSession(from: snapshot)

    #expect(focusedSession?.sessionID == "ses_02")
    #expect(focusedSession?.taskID == "task_02")
}

@MainActor
@Test(
    "presenting the task drawer uses the focused session binding rather than inferred local state",
)
func taskDrawerUsesFocusedSessionBinding() async {
    let snapshot = AppShellSnapshot(
        meta: .init(snapshotRev: 4, runtimeRev: 4, projectionRev: 4, snapshotAt: .now),
        ops: .init(runningSlots: 1, maxSlots: 2, retryQueueCount: 0, workflowHealth: .ok),
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
                unreadCount: 0,
                projectID: "proj_demo",
                taskID: "task_01",
                workspaceRoot: "/tmp/demo/.haneulchi/task_01",
                baseRoot: ".",
                focusState: .background,
            ),
            .init(
                sessionID: "ses_02",
                title: "Review",
                currentDirectory: "/tmp/demo",
                mode: .generic,
                runtimeState: .running,
                manualControlState: .none,
                dispatchState: .dispatchable,
                unreadCount: 0,
                projectID: "proj_demo",
                taskID: "task_ready",
                workspaceRoot: "/tmp/demo/worktrees/task_ready",
                baseRoot: "Sources",
                focusState: .focused,
            ),
        ],
        attention: [],
        retryQueue: [],
        warnings: [],
    )
    let workflow = WorkflowStatusPayload(
        state: .ok,
        path: "/tmp/demo/WORKFLOW.md",
        lastGoodHash: "sha256:abc123",
        lastReloadAt: "2026-03-23T07:05:00Z",
        lastError: nil,
        workflow: .init(
            name: "Review Workflow",
            strategy: "worktree",
            baseRoot: "Sources",
            reviewChecklist: ["tests"],
            allowedAgents: ["codex"],
            hooks: [],
            hookRuns: [:],
            templateBody: nil,
        ),
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
        shellSnapshot: snapshot,
        workflowStatus: workflow,
    )

    await model.perform(.presentTaskContextDrawer)

    #expect(model.taskContextDrawerModel?.taskID == "task_ready")
    #expect(model.taskContextDrawerModel?.sessionID == "ses_02")
    #expect(model.taskContextDrawerModel?.workflowName == "Review Workflow")
}

@Test(
    "project focus explorer layout uses shared column widths, gutter, and supporting column alignment",
)
func projectFocusExplorerLayoutUsesSharedWorkspaceRhythm() {
    let layout = ProjectFocusWorkspaceLayoutMetrics.forPreset(.explorerTerminalInspector)

    #expect(layout.outerPadding == HaneulchiMetrics.Padding.card)
    #expect(layout.columnSpacing == HaneulchiMetrics.Spacing.md)
    #expect(layout.supportingColumnSpacing == HaneulchiMetrics.Spacing.md)
    #expect(layout.sessionColumnWidth == 248)
    #expect(layout.explorerColumnWidth == 272)
    #expect(layout.supportingColumnWidth == 352)
    #expect(layout.stacksSupportingPanelsInSharedColumn == true)
}

@Test(
    "project focus inspector switches to compact tabs when sections would overflow a segmented control",
)
func projectFocusInspectorUsesCompactTabsWhenSectionCountIsHigh() {
    let denseLayout = ProjectFocusWorkspaceLayoutMetrics.forPreset(
        .explorerTerminalInspector,
        inspectorSectionCount: InspectorSection.allCases.count,
    )
    let sparseLayout = ProjectFocusWorkspaceLayoutMetrics.forPreset(
        .explorerTerminalInspector,
        inspectorSectionCount: 3,
    )

    #expect(denseLayout.inspectorControlStyle == .compactScroll)
    #expect(sparseLayout.inspectorControlStyle == .segmented)
}

@Test("project focus collapses non-terminal columns as the available width tightens")
func projectFocusLayoutCollapsesSupportingColumnsOnNarrowWidths() {
    let wideLayout = ProjectFocusWorkspaceLayoutMetrics.forPreset(
        .explorerTerminalInspector,
        availableWidth: 1400,
    )
    let mediumLayout = ProjectFocusWorkspaceLayoutMetrics.forPreset(
        .explorerTerminalInspector,
        availableWidth: 1080,
    )
    let narrowLayout = ProjectFocusWorkspaceLayoutMetrics.forPreset(
        .explorerTerminalInspector,
        availableWidth: 880,
    )

    #expect(wideLayout.showsExplorerColumn == true)
    #expect(wideLayout.showsSupportingColumn == true)
    #expect(mediumLayout.showsExplorerColumn == false)
    #expect(mediumLayout.showsSupportingColumn == true)
    #expect(narrowLayout.showsExplorerColumn == false)
    #expect(narrowLayout.showsSupportingColumn == false)
}
