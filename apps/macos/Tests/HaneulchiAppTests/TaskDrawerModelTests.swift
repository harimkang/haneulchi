import Foundation
import Testing
@testable import HaneulchiApp

@Test("task drawer model resolves linked session, workspace, and workflow details from the authoritative snapshot")
func taskDrawerModelUsesSnapshotAndWorkflowProjection() {
    let snapshot = AppShellSnapshot(
        meta: .init(snapshotRev: 3, runtimeRev: 3, projectionRev: 3, snapshotAt: .now),
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
                latestSummary: "Running tests",
                focusState: .background
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
                latestSummary: "Review evidence",
                focusState: .focused
            ),
        ],
        attention: [],
        retryQueue: [],
        warnings: []
    )
    let workflow = WorkflowStatusPayload(
        state: .ok,
        path: "/tmp/demo/WORKFLOW.md",
        lastGoodHash: "sha256:abc123",
        lastReloadAt: "2026-03-23T07:00:00Z",
        lastError: nil,
        workflow: .init(
            name: "Review Workflow",
            strategy: "worktree",
            baseRoot: "Sources",
            reviewChecklist: ["tests", "lint"],
            allowedAgents: ["codex"],
            hooks: [],
            hookRuns: [:],
            templateBody: nil
        )
    )

    let model = TaskDrawerModel.resolve(from: snapshot, workflowStatus: workflow)

    #expect(model?.taskID == "task_ready")
    #expect(model?.sessionID == "ses_02")
    #expect(model?.sessionTitle == "Review")
    #expect(model?.workspaceRoot == "/tmp/demo/worktrees/task_ready")
    #expect(model?.baseRoot == "Sources")
    #expect(model?.workflowName == "Review Workflow")
    #expect(model?.primaryActionTitle == "Detach Session")
}
