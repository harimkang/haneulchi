import Foundation
@testable import HaneulchiApp
import Testing

@Test(
    "task drawer model resolves linked session, workspace, and workflow details from the authoritative snapshot",
)
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
                focusState: .background,
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
                automationMode: .assisted,
                trackerBindingState: "local_only",
                workspaceRoot: "/tmp/demo/worktrees/task_ready",
                baseRoot: "Sources",
                latestSummary: "Review evidence",
                focusState: .focused,
            ),
        ],
        attention: [],
        retryQueue: [],
        warnings: [],
    )
    let workflow = WorkflowStatusPayload(
        state: .ok,
        path: "/tmp/demo/WORKFLOW.md",
        lastGoodHash: "sha256:abc123",
        lastReloadAt: "2026-03-23T07:00:00Z",
        lastError: nil,
        lastBootstrap: nil,
        workflow: .init(
            name: "Review Workflow",
            strategy: "worktree",
            baseRoot: "Sources",
            requireReview: true,
            maxRuntimeMinutes: 45,
            unsafeOverridePolicy: "explicit_only",
            reviewChecklist: ["tests", "lint"],
            allowedAgents: ["codex"],
            hooks: [],
            hookRuns: [:],
            templateBody: nil,
        ),
    )

    let model = TaskDrawerModel.resolve(from: snapshot, workflowStatus: workflow)

    #expect(model?.taskID == "task_ready")
    #expect(model?.sessionID == "ses_02")
    #expect(model?.sessionTitle == "Review")
    #expect(model?.workspaceRoot == "/tmp/demo/worktrees/task_ready")
    #expect(model?.baseRoot == "Sources")
    #expect(model?.workflowName == "Review Workflow")
    #expect(model?.automationMode == .assisted)
    #expect(model?.dispatchState == .dispatchable)
    #expect(model?.retryState == nil)
    #expect(model?.requireReview == true)
    #expect(model?.maxRuntimeMinutes == 45)
    #expect(model?.unsafeOverridePolicy == "explicit_only")
    #expect(model?.workflowBindingSummary.contains("ok") == true)
    #expect(model?.lineageSummary.contains("ses_02") == true)
    #expect(model?.primaryActionTitle == "Detach Session")
}

@Test("task drawer model keeps append-only timeline items and broken-link warnings visible")
func taskDrawerModelKeepsTimelineWarningsVisible() {
    let snapshot = AppShellSnapshot(
        meta: .init(snapshotRev: 4, runtimeRev: 4, projectionRev: 4, snapshotAt: .now),
        ops: .init(runningSlots: 1, maxSlots: 2, retryQueueCount: 0, workflowHealth: .ok),
        app: .init(activeRoute: .projectFocus, focusedSessionID: "ses_02", degradedFlags: []),
        projects: [],
        sessions: [
            .init(
                sessionID: "ses_02",
                title: "Review",
                currentDirectory: "/tmp/demo",
                mode: .generic,
                runtimeState: .reviewReady,
                manualControlState: .none,
                dispatchState: .dispatchable,
                unreadCount: 0,
                projectID: "proj_demo",
                taskID: "task_review",
                automationMode: .manual,
                trackerBindingState: "bound",
                workspaceRoot: "/tmp/demo/worktrees/task_review",
                baseRoot: ".",
                latestSummary: "Review ready",
                focusState: .focused,
            ),
        ],
        attention: [],
        retryQueue: [],
        warnings: [],
    )
    let workflow = WorkflowStatusPayload(
        state: .ok,
        path: "/tmp/demo/WORKFLOW.md",
        lastGoodHash: "sha256:abc123",
        lastReloadAt: "2026-03-23T07:10:00Z",
        lastError: nil,
        lastBootstrap: .init(
            workspaceRoot: "/tmp/demo/worktrees/task_review",
            baseRoot: ".",
            sessionCwd: "/tmp/demo/worktrees/task_review",
            renderedPromptPath: "/tmp/demo/worktrees/task_review/prompt.rendered.md",
            phaseSequence: ["resolve", "launch", "evidence"],
            hookPhaseResults: [],
            outcomeCode: "launch_succeeded",
            warningCodes: [],
            claimReleased: false,
            launchExitCode: 0,
            lastKnownGoodHash: "sha256:abc123",
        ),
        workflow: .init(
            name: "Review Workflow",
            strategy: "worktree",
            baseRoot: ".",
            requireReview: true,
            maxRuntimeMinutes: 30,
            unsafeOverridePolicy: "explicit_only",
            reviewChecklist: ["tests"],
            allowedAgents: ["gemini"],
            hooks: [],
            hookRuns: [:],
            templateBody: nil,
        ),
    )
    let timeline = [
        TaskTimelineEntry(
            id: "evt_01",
            kind: "review_ready",
            actor: "workflow",
            summary: "Review evidence captured",
            warningReason: nil,
            createdAt: "2026-03-23T07:12:00Z",
        ),
        TaskTimelineEntry(
            id: "evt_02",
            kind: "warning",
            actor: "timeline",
            summary: "Missing review item reference",
            warningReason: "broken_link",
            createdAt: "2026-03-23T07:13:00Z",
        ),
    ]

    let model = TaskDrawerModel.resolve(
        from: snapshot,
        workflowStatus: workflow,
        timeline: timeline,
    )

    #expect(model?.timeline.count == 2)
    #expect(model?.timeline.last?.warningReason == "broken_link")
    #expect(model?.lastBootstrapOutcome == "launch_succeeded")
    #expect(model?.bootstrapPhaseSummary == "resolve -> launch -> evidence")
    #expect(model?.lineageSummary.contains("/tmp/demo/worktrees/task_review") == true)
    #expect(model?.trackerBindingState == "bound")
    #expect(model?.blockerReason == "manual_mode")
}

@Test("task drawer model surfaces retry state, due time, and degraded workflow badges")
func taskDrawerModelSurfacesRetryAndDegradedAutomation() {
    let snapshot = AppShellSnapshot(
        meta: .init(snapshotRev: 5, runtimeRev: 5, projectionRev: 5, snapshotAt: .now),
        ops: .init(
            runningSlots: 1,
            maxSlots: 2,
            retryQueueCount: 1,
            workflowHealth: .invalidKeptLastGood,
        ),
        app: .init(
            activeRoute: .projectFocus,
            focusedSessionID: "ses_03",
            degradedFlags: [.degraded],
        ),
        projects: [],
        sessions: [
            .init(
                sessionID: "ses_03",
                title: "Retry",
                currentDirectory: "/tmp/demo",
                mode: .structuredAdapter,
                runtimeState: .waitingInput,
                manualControlState: .none,
                dispatchState: .dispatchFailed,
                unreadCount: 1,
                projectID: "proj_demo",
                taskID: "task_retry",
                automationMode: .autoEligible,
                trackerBindingState: "degraded",
                workspaceRoot: "/tmp/demo/worktrees/task_retry",
                baseRoot: ".",
                latestSummary: "Retry due",
                dispatchReason: "stale_target_session",
                focusState: .focused,
            ),
        ],
        attention: [],
        retryQueue: [
            .init(
                taskID: "task_retry",
                projectID: "proj_demo",
                attempt: 2,
                reasonCode: "adapter_timeout",
                dueAt: "2026-03-23T17:00:00Z",
                backoffMs: 60000,
                claimState: .claimed,
            ),
        ],
        warnings: [],
    )
    let workflow = WorkflowStatusPayload(
        state: .invalidKeptLastGood,
        path: "/tmp/demo/WORKFLOW.md",
        lastGoodHash: "sha256:abc123",
        lastReloadAt: "2026-03-23T16:55:00Z",
        lastError: "parse error",
        workflow: .init(
            name: "Retry Workflow",
            strategy: "worktree",
            baseRoot: ".",
            requireReview: false,
            maxRuntimeMinutes: 30,
            unsafeOverridePolicy: "explicit_only",
            reviewChecklist: [],
            allowedAgents: ["codex"],
            hooks: [],
            hookRuns: [:],
            templateBody: nil,
        ),
    )

    let model = TaskDrawerModel.resolve(from: snapshot, workflowStatus: workflow)

    #expect(model?.retryState == "attempt 2 · due 2026-03-23T17:00:00Z")
    #expect(model?.dispatchReason == "stale_target_session")
    #expect(model?.trackerBindingState == "degraded")
    #expect(model?.blockerReason == "workflow_invalid")
    #expect(model?.workflowBindingSummary.contains("invalid_kept_last_good") == true)
}
