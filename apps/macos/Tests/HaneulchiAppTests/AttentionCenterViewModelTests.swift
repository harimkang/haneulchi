import Foundation
@testable import HaneulchiAppUI
import Testing

@MainActor
@Test("attention center view model maps open, resolve, dismiss, and snooze actions")
func attentionCenterViewModelMapsActions() throws {
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
                canTakeover: true,
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
                focusState: .background,
            ),
        ],
        attention: [
            .init(
                attentionID: "att_waiting",
                headline: "Needs input",
                severity: .unread,
                targetRoute: .projectFocus,
                targetSessionID: "ses_waiting",
                summary: "Operator answer required.",
            ),
            .init(
                attentionID: "att_review",
                headline: "Review ready",
                severity: .degraded,
                targetRoute: .reviewQueue,
                targetSessionID: nil,
                taskID: "task_review",
                summary: "Evidence pack available.",
            ),
            .init(
                attentionID: "att_workflow",
                headline: "Workflow invalid",
                severity: .degraded,
                targetRoute: .attentionCenter,
                targetSessionID: nil,
                summary: "Last known good kept.",
                actionHint: "reload_workflow",
            ),
        ],
        retryQueue: [],
        warnings: [],
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
        snoozeAttention: { snoozed.append($0) },
    )

    #expect(viewModel.items.map(\.id) == [
        "ses_takeover",
        "ses_failed",
        "att_review",
        "att_workflow",
        "att_waiting",
    ])
    #expect(viewModel.items[0].stateLabel == "manual takeover")
    #expect(viewModel.items[1].stateLabel == "dispatch_failed")
    #expect(viewModel.items[0].attentionActionID == nil)
    #expect(viewModel.items[1].attentionActionID == "attention-dispatch-ses_failed")
    #expect(viewModel.items[3].attentionActionID == "att_workflow")

    let workflowItem = try #require(viewModel.items.first(where: { $0.id == "att_workflow" }))
    let reviewItem = try #require(viewModel.items.first(where: { $0.id == "att_review" }))
    let waitingItem = try #require(viewModel.items.first(where: { $0.id == "att_waiting" }))
    let dispatchFailedItem = try #require(viewModel.items.first(where: { $0.id == "ses_failed" }))

    viewModel.open(workflowItem)
    viewModel.open(waitingItem)
    viewModel.resolve(dispatchFailedItem)
    viewModel.resolve(reviewItem)
    viewModel.dismiss(reviewItem)
    viewModel.snooze(waitingItem)

    #expect(opened == [.presentWorkflowDrawer, .jumpToSession("ses_waiting")])
    #expect(resolved == ["attention-dispatch-ses_failed", "att_review"])
    #expect(dismissed == ["att_review"])
    #expect(snoozed == ["att_waiting"])
}

@Test("attention presentation keeps manual takeover ahead of failed and degraded")
func attentionPresentationUsesDistinctGroups() {
    let items: [AttentionCenterViewModel.Item] = [
        .init(
            id: "ses_takeover",
            headline: "Manual review",
            stateLabel: "manual takeover",
            summary: "Operator reviewing latest result.",
            severity: .failed,
            targetRouteTitle: Route.projectFocus.title,
            openAction: .jumpToSession("ses_takeover"),
            attentionActionID: nil,
        ),
        .init(
            id: "ses_failed",
            headline: "Dispatch failed",
            stateLabel: "dispatch_failed",
            summary: "Target stale.",
            severity: .failed,
            targetRouteTitle: Route.projectFocus.title,
            openAction: .jumpToSession("ses_failed"),
            attentionActionID: "attention-dispatch-ses_failed",
        ),
        .init(
            id: "att_review",
            headline: "Review ready",
            stateLabel: "degraded",
            summary: "Evidence pack available.",
            severity: .degraded,
            targetRouteTitle: Route.reviewQueue.title,
            openAction: .selectRoute(.reviewQueue),
            attentionActionID: "att_review",
        ),
        .init(
            id: "att_waiting",
            headline: "Needs input",
            stateLabel: "unread",
            summary: "Operator answer required.",
            severity: .unread,
            targetRouteTitle: Route.projectFocus.title,
            openAction: .jumpToSession("ses_waiting"),
            attentionActionID: "att_waiting",
        ),
    ]

    let groups = AttentionCenterPresentation.grouped(items)

    #expect(groups.map(\.group) == [.manualTakeover, .failed, .degraded, .unread])
    #expect(groups[0].group.badgeState == .manualTakeover)
    #expect(groups[0].group.accent == .manual)
    #expect(groups[1].group.accent == .error)
    #expect(groups[2].group.badgeState == .degraded)
    #expect(groups[2].group.accent == .warning)
    #expect(groups[3].group.accent == .warning)
}
