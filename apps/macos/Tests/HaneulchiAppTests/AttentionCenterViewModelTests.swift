import Foundation
import Testing
@testable import HaneulchiApp

@MainActor
@Test("attention center view model maps open, resolve, dismiss, and snooze actions")
func attentionCenterViewModelMapsActions() async throws {
    let snapshot = AppShellSnapshot(
        meta: .init(snapshotRev: 1, runtimeRev: 1, projectionRev: 1, snapshotAt: .now),
        ops: .init(runningSlots: 1, maxSlots: 2, retryQueueCount: 0, workflowHealth: .ok),
        app: .init(activeRoute: .attentionCenter, focusedSessionID: nil, degradedFlags: []),
        projects: [],
        sessions: [
            .init(
                sessionID: "ses_takeover",
                title: "Manual review",
                currentDirectory: "/tmp/demo",
                mode: .structuredAdapter,
                runtimeState: .waitingInput,
                manualControlState: .takeover,
                dispatchState: .dispatchable,
                unreadCount: 0,
                projectID: "proj_demo",
                latestSummary: "Operator reviewing latest result.",
                focusState: .background,
                canTakeover: true
            ),
            .init(
                sessionID: "ses_failed",
                title: "Dispatch failed",
                currentDirectory: "/tmp/demo",
                mode: .structuredAdapter,
                runtimeState: .running,
                manualControlState: .none,
                dispatchState: .dispatchFailed,
                unreadCount: 0,
                projectID: "proj_demo",
                latestSummary: "Target stale.",
                dispatchReason: "stale_target_session",
                focusState: .background
            ),
        ],
        attention: [
            .init(
                attentionID: "att_waiting",
                headline: "Needs input",
                severity: .unread,
                targetRoute: .projectFocus,
                targetSessionID: "ses_waiting",
                summary: "Operator answer required."
            ),
            .init(
                attentionID: "att_review",
                headline: "Review ready",
                severity: .degraded,
                targetRoute: .reviewQueue,
                targetSessionID: nil,
                taskID: "task_review",
                summary: "Evidence pack available."
            ),
        ],
        retryQueue: [],
        warnings: []
    )

    var opened: [AppShellAction] = []
    var resolved: [String] = []
    var dismissed: [String] = []
    var snoozed: [String] = []

    let viewModel = AttentionCenterViewModel(
        snapshot: snapshot,
        openTarget: { opened.append($0) },
        resolveAttention: { resolved.append($0) },
        dismissAttention: { dismissed.append($0) },
        snoozeAttention: { snoozed.append($0) }
    )

    #expect(viewModel.items.map(\.id) == ["ses_takeover", "ses_failed", "att_review", "att_waiting"])
    #expect(viewModel.items[0].stateLabel == "manual takeover")
    #expect(viewModel.items[1].stateLabel == "dispatch_failed")

    viewModel.open(viewModel.items[3])
    viewModel.resolve(viewModel.items[2])
    viewModel.dismiss(viewModel.items[2])
    viewModel.snooze(viewModel.items[3])

    #expect(opened == [.jumpToSession("ses_waiting")])
    #expect(resolved == ["att_review"])
    #expect(dismissed == ["att_review"])
    #expect(snoozed == ["att_waiting"])
}
