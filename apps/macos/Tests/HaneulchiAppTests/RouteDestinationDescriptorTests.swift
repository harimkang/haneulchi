import Foundation
import Testing
@testable import HaneulchiApp

@Test("placeholder descriptor reflects route-specific shell summary")
func placeholderDescriptorUsesRouteAndSnapshot() {
    let snapshot = AppShellSnapshot(
        meta: .init(snapshotRev: 2, runtimeRev: 2, projectionRev: 2, snapshotAt: .now),
        ops: .init(runningSlots: 1, maxSlots: 4, retryQueueCount: 0, workflowHealth: .ok),
        app: .init(activeRoute: .attentionCenter, focusedSessionID: "restore-1", degradedFlags: [.degraded]),
        projects: [
            .init(
                projectID: "proj_demo",
                name: "demo",
                rootPath: "/tmp/demo",
                status: .active,
                workflowState: .ok,
                sessionCount: 1,
                attentionCount: 1
            )
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
                unreadCount: 0
            )
        ],
        attention: [
            .init(
                attentionID: "att_01",
                headline: "Manual input required",
                severity: .degraded,
                targetRoute: .attentionCenter,
                targetSessionID: "restore-1"
            )
        ],
        retryQueue: [],
        warnings: []
    )

    let descriptor = RouteDestinationDescriptor.placeholder(for: .attentionCenter, snapshot: snapshot)

    #expect(descriptor.title == "Attention Center")
    #expect(descriptor.summary.contains("1 attention"))
    #expect(descriptor.nextActionTitle == "Open Attention Center")
}
