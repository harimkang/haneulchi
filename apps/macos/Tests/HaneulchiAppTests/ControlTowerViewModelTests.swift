import Foundation
import Testing
@testable import HaneulchiApp

@Test("control tower view model groups project cards, attention targets, recent artifacts, and ops strip labels")
func controlTowerViewModelUsesProjectionDrivenSnapshot() {
    let snapshot = AppShellSnapshot(
        meta: .init(snapshotRev: 11, runtimeRev: 8, projectionRev: 14, snapshotAt: .now),
        ops: .init(
            cadenceMs: 15_000,
            lastTickAt: "2026-03-23T12:00:00Z",
            lastReconcileAt: "2026-03-23T12:00:30Z",
            runningSlots: 2,
            maxSlots: 4,
            retryQueueCount: 3,
            queuedClaimCount: 1,
            workflowHealth: .invalidKeptLastGood,
            trackerHealth: "degraded",
            paused: false
        ),
        app: .init(activeRoute: .controlTower, focusedSessionID: "ses_waiting", degradedFlags: [.degraded]),
        projects: [
            .init(
                projectID: "proj_demo",
                name: "demo",
                rootPath: "/tmp/demo",
                status: .active,
                workflowState: .invalidKeptLastGood,
                sessionCount: 2,
                attentionCount: 2,
                taskCounts: ["Ready": 2]
            )
        ],
        sessions: [
            .init(
                sessionID: "ses_waiting",
                title: "Needs input",
                currentDirectory: "/tmp/demo",
                mode: .structuredAdapter,
                runtimeState: .waitingInput,
                manualControlState: .none,
                dispatchState: .dispatchable,
                unreadCount: 2,
                projectID: "proj_demo",
                latestSummary: "Awaiting operator answer",
                providerID: "anthropic",
                modelID: "claude-sonnet-4",
                dispatchReason: "dispatchable",
                latestCommentary: "Need a schema confirmation.",
                activeWindowTitle: "Terminal 1",
                claimState: .claimed,
                focusState: .focused,
                canFocus: true,
                canTakeover: true,
                canReleaseTakeover: false
            ),
            .init(
                sessionID: "ses_running",
                title: "Implementing",
                currentDirectory: "/tmp/demo",
                mode: .generic,
                runtimeState: .running,
                manualControlState: .none,
                dispatchState: .notDispatchable,
                unreadCount: 0,
                projectID: "proj_demo",
                latestSummary: "Applying the latest migration",
                claimState: .none,
                focusState: .background,
                canFocus: true,
                canTakeover: false,
                canReleaseTakeover: false
            )
        ],
        attention: [
            .init(
                attentionID: "att_waiting",
                headline: "Needs input",
                severity: .unread,
                targetRoute: .projectFocus,
                targetSessionID: "ses_waiting",
                projectID: "proj_demo",
                summary: "Operator answer required.",
                actionHint: "focus_session"
            ),
            .init(
                attentionID: "att_review",
                headline: "Review ready",
                severity: .degraded,
                targetRoute: .reviewQueue,
                targetSessionID: nil,
                projectID: "proj_demo",
                taskID: "task_review",
                summary: "Evidence pack available.",
                actionHint: "open_review"
            )
        ],
        retryQueue: [
            .init(
                taskID: "task_retry",
                projectID: "proj_demo",
                attempt: 2,
                reasonCode: "adapter_timeout",
                dueAt: "2026-03-23T12:01:00Z",
                backoffMs: 45_000,
                claimState: .claimed
            )
        ],
        warnings: [],
        workflow: .init(
            state: .invalidKeptLastGood,
            path: "/tmp/demo/WORKFLOW.md",
            lastGoodHash: "sha256:abc123",
            lastReloadAt: "2026-03-23T11:59:58Z",
            lastError: "front matter parse error"
        ),
        tracker: .init(state: "local_only", lastSyncAt: nil, health: "degraded"),
        recentArtifacts: [
            .init(
                taskID: "task_review",
                projectID: "proj_demo",
                summary: "Review ready",
                jumpTarget: "review_queue",
                manifestPath: "evidence/manifest.json"
            )
        ]
    )

    let viewModel = ControlTowerViewModel(snapshot: snapshot)

    #expect(viewModel.projectCards.count == 1)
    #expect(viewModel.projectCards.first?.statusLabel == "attention")
    #expect(viewModel.projectCards.first?.sessionCountLabel == "2 sessions")
    #expect(viewModel.projectCards.first?.attentionCountLabel == "2 items")
    #expect(viewModel.projectCards.first?.latestSummary == "Awaiting operator answer")
    #expect(viewModel.projectCards.first?.latestCommentary == "Need a schema confirmation.")
    #expect(viewModel.projectCards.first?.heatStrip.waitingInput == 1)
    #expect(viewModel.projectCards.first?.heatStrip.running == 1)

    #expect(viewModel.attentionItems.count == 2)
    #expect(viewModel.attentionItems.first?.targetRoute == .projectFocus)
    #expect(viewModel.attentionItems.first?.targetSessionID == "ses_waiting")
    #expect(viewModel.attentionItems.last?.targetRoute == .reviewQueue)

    #expect(viewModel.recentArtifacts.first?.targetRoute == .reviewQueue)
    #expect(viewModel.recentArtifacts.first?.manifestPath == "evidence/manifest.json")

    #expect(viewModel.opsModel.slotLabel == "2/4")
    #expect(viewModel.opsModel.queueLabel == "3 retry · 1 claimed")
    #expect(viewModel.opsModel.workflowHealth == "invalid_kept_last_good")
    #expect(viewModel.opsModel.trackerHealth == "degraded")
}
