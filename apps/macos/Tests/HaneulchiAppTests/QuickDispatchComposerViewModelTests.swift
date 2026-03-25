@testable import HaneulchiApp
import Testing

@Test("quick dispatch composer groups adapter watch targets and disables send for stale sessions")
func quickDispatchComposerViewModelBuildsTargetsAndEnablement() {
    let snapshot = AppShellSnapshot(
        meta: .init(snapshotRev: 1, runtimeRev: 1, projectionRev: 1, snapshotAt: .now),
        ops: .init(runningSlots: 1, maxSlots: 2, retryQueueCount: 0, workflowHealth: .ok),
        app: .init(activeRoute: .projectFocus, focusedSessionID: "ses_dispatch", degradedFlags: []),
        projects: [],
        sessions: [
            .init(
                sessionID: "ses_dispatch",
                title: "Claude session",
                currentDirectory: "/tmp/demo",
                mode: .structuredAdapter,
                runtimeState: .running,
                manualControlState: .none,
                dispatchState: .dispatchable,
                unreadCount: 0,
                projectID: "proj_demo",
                providerID: "anthropic",
                modelID: "claude-sonnet-4",
                dispatchReason: "dispatchable",
                latestCommentary: "Awaiting dispatch.",
                activeWindowTitle: "Terminal 1",
                adapterKind: "claude_code",
                canFocus: true,
                canTakeover: true,
                canReleaseTakeover: false,
            ),
            .init(
                sessionID: "ses_stale",
                title: "Stale target",
                currentDirectory: "/tmp/demo",
                mode: .structuredAdapter,
                runtimeState: .running,
                manualControlState: .none,
                dispatchState: .dispatchFailed,
                unreadCount: 0,
                projectID: "proj_demo",
                providerID: "anthropic",
                modelID: "claude-sonnet-4",
                dispatchReason: "stale_target_session",
                latestCommentary: "Target stale.",
                activeWindowTitle: "Terminal 2",
                adapterKind: "claude_code",
                canFocus: true,
                canTakeover: false,
                canReleaseTakeover: false,
            ),
        ],
        attention: [],
        retryQueue: [],
        warnings: [],
    )

    let viewModel = QuickDispatchComposerViewModel(snapshot: snapshot, origin: .controlTower)

    #expect(viewModel.origin == .controlTower)
    #expect(viewModel.targets.count == 3)
    #expect(viewModel.targets[0].title == "Claude session")
    #expect(viewModel.targets[0].subtitle == "anthropic · claude-sonnet-4")
    #expect(viewModel.targets[1].disabledReason == "stale_target_session")
    #expect(viewModel.targets[2].isNewSession == true)

    var staleSelection = viewModel
    staleSelection.selectTarget(id: "ses_stale")
    staleSelection.messageText = "rerun tests"
    #expect(staleSelection.sendEnabled == false)
    #expect(staleSelection.sendDisabledReason == "stale_target_session")

    var readySelection = viewModel
    readySelection.selectTarget(id: "ses_dispatch")
    readySelection.messageText = "rerun tests"
    #expect(readySelection.sendEnabled == true)
    #expect(readySelection.sendDisabledReason == nil)
}
