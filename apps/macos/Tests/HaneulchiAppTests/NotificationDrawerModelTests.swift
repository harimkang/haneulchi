import Foundation
@testable import HaneulchiAppUI
import Testing

@Test("notification drawer model orders attention rows and maps deep links deterministically")
func notificationDrawerModelOrdersRowsAndTargets() {
    let snapshot = AppShellSnapshot(
        meta: .init(snapshotRev: 1, runtimeRev: 1, projectionRev: 1, snapshotAt: .now),
        ops: .init(runningSlots: 1, maxSlots: 2, retryQueueCount: 0, workflowHealth: .ok),
        app: .init(activeRoute: .projectFocus, focusedSessionID: nil, degradedFlags: []),
        projects: [],
        sessions: [
            .init(
                sessionID: "ses_manual",
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
                latestSummary: "Last dispatch failed.",
                dispatchReason: "stale_target_session",
                focusState: .background,
            ),
        ],
        attention: [
            .init(
                attentionID: "att_unread",
                headline: "Unread update",
                severity: .unread,
                targetRoute: .projectFocus,
                targetSessionID: "ses_unread",
                summary: "New commentary arrived.",
            ),
            .init(
                attentionID: "att_failed",
                headline: "Dispatch failed",
                severity: .failed,
                targetRoute: .reviewQueue,
                targetSessionID: nil,
                summary: "Target session is stale.",
            ),
            .init(
                attentionID: "att_degraded",
                headline: "Workflow invalid",
                severity: .degraded,
                targetRoute: .attentionCenter,
                targetSessionID: nil,
                summary: "Last known good was kept.",
            ),
        ],
        retryQueue: [],
        warnings: [],
    )

    let model = NotificationDrawerModel(snapshot: snapshot)

    #expect(model.items.map(\.id) == [
        "ses_manual",
        "ses_failed",
        "att_failed",
        "att_degraded",
        "att_unread",
    ])
    #expect(model.items.first?.stateLabel == "manual takeover")
    #expect(model.items[1].stateLabel == "dispatch_failed")
    #expect(model.items[1].summary.contains("stale_target_session") == true)
    #expect(model.items[2].action == .selectRoute(.reviewQueue))
    #expect(model.items.last?.action == .jumpToSession("ses_unread"))
    #expect(model.items.last?.summary == "New commentary arrived.")
}
