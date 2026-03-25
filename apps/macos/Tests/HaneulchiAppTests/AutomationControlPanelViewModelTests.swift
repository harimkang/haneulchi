@testable import HaneulchiApp
import Testing

private func makePanelSnapshot() -> AppShellSnapshot {
    AppShellSnapshot(
        meta: .init(snapshotRev: 7, runtimeRev: 4, projectionRev: 9, snapshotAt: .now),
        ops: .init(
            cadenceMs: 15000,
            lastTickAt: "2026-03-23T12:00:00Z",
            lastReconcileAt: "2026-03-23T12:00:30Z",
            runningSlots: 2,
            maxSlots: 4,
            retryQueueCount: 3,
            queuedClaimCount: 1,
            workflowHealth: .invalidKeptLastGood,
            trackerHealth: "degraded",
            paused: false,
        ),
        app: .init(activeRoute: .settings, focusedSessionID: nil, degradedFlags: [.degraded]),
        projects: [],
        sessions: [],
        attention: [],
        retryQueue: [],
        warnings: [],
        workflow: .init(
            state: .invalidKeptLastGood,
            path: "/tmp/demo/WORKFLOW.md",
            lastGoodHash: "sha256:abc123",
            lastReloadAt: "2026-03-23T12:00:00Z",
            lastError: "front matter parse error",
        ),
        tracker: .init(state: "local_only", lastSyncAt: nil, health: "degraded"),
    )
}

@Test(
    "automation control panel view model groups orchestrator, workflow, API, CLI, tracker, and action bar state",
)
func automationControlPanelViewModelUsesLiveSnapshotDiagnostics() {
    let snapshot = makePanelSnapshot()
    let model = AutomationControlPanelViewModel(
        snapshot: snapshot,
        runtimeInfo: .init(rendererID: "swiftterm", transport: "ffi_c_abi", demoMode: false),
    )

    #expect(model.orchestratorSummary.contains("2/4"))
    #expect(model.orchestratorSummary.contains("3 retry"))
    #expect(model.workflowSummary.contains("invalid_kept_last_good"))
    #expect(model.apiSummary.contains("ffi_c_abi"))
    #expect(model.cliSummary.contains("same snapshot contract"))
    #expect(model.trackerSummary.contains("degraded"))
    #expect(model.actions == ["Refresh", "Reconcile", "Reload Workflow", "Export Snapshot"])
}
