import Foundation
@testable import HaneulchiApp
import Testing

@Test(
    "chrome state derives top-bar chips, left-rail badges, and bottom-strip items from one snapshot",
)
func chromeStateUsesSingleSnapshot() {
    let snapshot = AppShellSnapshot(
        meta: .init(snapshotRev: 3, runtimeRev: 3, projectionRev: 2, snapshotAt: .now),
        ops: .init(runningSlots: 2, maxSlots: 4, retryQueueCount: 0, workflowHealth: .ok),
        app: .init(
            activeRoute: .controlTower,
            focusedSessionID: "restore-1",
            degradedFlags: [.degraded],
        ),
        projects: [
            .init(
                projectID: "proj_demo",
                name: "demo",
                rootPath: "/tmp/demo",
                status: .active,
                workflowState: .ok,
                sessionCount: 2,
                attentionCount: 1,
            ),
        ],
        sessions: [
            .init(
                sessionID: "restore-1",
                title: "demo",
                currentDirectory: "/tmp/demo",
                mode: .generic,
                runtimeState: .running,
                manualControlState: .none,
                dispatchState: .notDispatchable,
                unreadCount: 0,
            ),
            .init(
                sessionID: "restore-2",
                title: "api",
                currentDirectory: "/tmp/demo",
                mode: .generic,
                runtimeState: .running,
                manualControlState: .none,
                dispatchState: .notDispatchable,
                unreadCount: 0,
            ),
        ],
        attention: [
            .init(
                attentionID: "att_01",
                headline: "Manual input required",
                severity: .degraded,
                targetRoute: .attentionCenter,
                targetSessionID: "restore-2",
            ),
        ],
        retryQueue: [],
        warnings: [
            .init(
                warningID: "warn_01",
                severity: .degraded,
                headline: "Preset binaries missing",
                nextAction: "Open Settings",
            ),
        ],
    )

    let chrome = AppShellChromeState(
        snapshot: snapshot,
        selectedProjectName: "demo",
        transientNotice: "File queued",
    )

    #expect(chrome.topBarChips.map(\.title).contains("degraded"))
    #expect(chrome.leftRailItems.first?.route == .projectFocus)
    #expect(chrome.bottomStripItems.map(\.title) == [
        "logs",
        "problems",
        "terminal",
        "runtime hint",
    ])
    #expect(chrome.bottomStripItems.first(where: { $0.title == "terminal" })?
        .detail == "2 sessions")
    #expect(chrome.transientNotice == "File queued")
}
