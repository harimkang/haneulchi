import Foundation
import Testing
@testable import HaneulchiApp

@MainActor
@Test("local shell snapshot mirrors the accepted top-level groups, metadata, and enum vocabulary")
func localSnapshotUsesCurrentShellInputs() async throws {
    let restoreStore = TerminalSessionRestoreStore.inMemory
    try restoreStore.save([.genericShell(at: "/tmp/demo")])

    let project = LauncherProject(
        projectID: "proj_demo",
        name: "demo",
        rootPath: "/tmp/demo",
        lastOpenedAt: .now
    )
    let report = ReadinessReport(
        project: project,
        checks: [
            .init(name: .shell, status: .ready, headline: "Shell ready", detail: "zsh available", nextAction: nil),
            .init(name: .presetBinaries, status: .degraded, headline: "Preset binaries missing", detail: "Generic shell remains available.", nextAction: "Open Settings"),
        ]
    )

    let snapshot = try await LocalAppShellSnapshotSource(restoreStore: restoreStore).load(
        activeRoute: .projectFocus,
        selectedProject: project,
        readinessReport: report,
        recentProjects: [project]
    )

    #expect(snapshot.meta.snapshotRev >= 1)
    #expect(snapshot.meta.runtimeRev >= 1)
    #expect(snapshot.meta.projectionRev >= 1)
    #expect(snapshot.app.activeRoute == .projectFocus)
    #expect(snapshot.ops.runningSlots == 0)
    #expect(snapshot.ops.retryQueueCount == 0)
    #expect(snapshot.projects.map(\.rootPath) == ["/tmp/demo"])
    #expect(snapshot.sessions.count == 1)
    #expect(snapshot.sessions.first?.mode == .generic)
    #expect(snapshot.sessions.first?.runtimeState == .exited)
    #expect(snapshot.attention.count == 1)
    #expect(snapshot.attention.first?.headline == "Preset binaries missing")
    #expect(snapshot.attention.first?.targetRoute == .attentionCenter)
    #expect(snapshot.retryQueue.isEmpty)
    #expect(snapshot.warnings.map(\.severity) == [.degraded])
}
