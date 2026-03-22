import Foundation

struct TaskDrawerModel: Equatable, Sendable {
    let automationMode: TaskBoardAutomationModePayload?
    let claimState: ClaimState
    let trackerBindingState: String?
    let requireReview: Bool
    let maxRuntimeMinutes: Int?
    let unsafeOverridePolicy: String?
    let blockerReason: String?
    let taskID: String
    let sessionID: String
    let sessionTitle: String
    let workspaceRoot: String?
    let baseRoot: String?
    let workflowName: String
    let workflowPath: String
    let strategy: String?
    let reviewChecklist: [String]
    let allowedAgents: [String]
    let lastGoodHash: String?
    let lastReloadAt: String?
    let lastError: String?
    let lastBootstrapOutcome: String?
    let renderedPromptPath: String?
    let hookPhaseResults: [WorkflowStatusPayload.HookPhaseResult]
    let timeline: [TaskTimelineEntry]
    let primaryActionTitle: String

    static func resolve(
        from snapshot: AppShellSnapshot,
        workflowStatus: WorkflowStatusPayload?,
        timeline: [TaskTimelineEntry] = []
    ) -> Self? {
        let focusedSession = snapshot.sessions.first(where: {
            $0.focusState == .focused || snapshot.app.focusedSessionID == $0.sessionID
        }) ?? snapshot.sessions.first

        guard let session = focusedSession, let taskID = session.taskID else {
            return nil
        }

        return Self(
            automationMode: session.automationMode,
            claimState: session.claimState,
            trackerBindingState: session.trackerBindingState,
            requireReview: workflowStatus?.workflow?.requireReview ?? false,
            maxRuntimeMinutes: workflowStatus?.workflow?.maxRuntimeMinutes,
            unsafeOverridePolicy: workflowStatus?.workflow?.unsafeOverridePolicy,
            blockerReason: blockerReason(
                automationMode: session.automationMode,
                claimState: session.claimState,
                workflowStatus: workflowStatus
            ),
            taskID: taskID,
            sessionID: session.sessionID,
            sessionTitle: session.title,
            workspaceRoot: session.workspaceRoot,
            baseRoot: session.baseRoot,
            workflowName: workflowStatus?.workflow?.name ?? "Workflow Contract",
            workflowPath: workflowStatus?.path ?? "",
            strategy: workflowStatus?.workflow?.strategy,
            reviewChecklist: workflowStatus?.workflow?.reviewChecklist ?? [],
            allowedAgents: workflowStatus?.workflow?.allowedAgents ?? [],
            lastGoodHash: workflowStatus?.lastGoodHash,
            lastReloadAt: workflowStatus?.lastReloadAt,
            lastError: workflowStatus?.lastError,
            lastBootstrapOutcome: workflowStatus?.lastBootstrap?.outcomeCode,
            renderedPromptPath: workflowStatus?.lastBootstrap?.renderedPromptPath,
            hookPhaseResults: workflowStatus?.lastBootstrap?.hookPhaseResults ?? [],
            timeline: timeline,
            primaryActionTitle: "Detach Session"
        )
    }

    private static func blockerReason(
        automationMode: TaskBoardAutomationModePayload?,
        claimState: ClaimState,
        workflowStatus: WorkflowStatusPayload?
    ) -> String? {
        if let automationMode, automationMode == .manual {
            return "manual_mode"
        }
        if workflowStatus?.state == .invalidKeptLastGood {
            return "workflow_invalid"
        }
        if let allowedAgents = workflowStatus?.workflow?.allowedAgents,
           !allowedAgents.isEmpty,
           !allowedAgents.contains("codex") {
            return "task_not_eligible_for_dispatch"
        }
        if claimState == .claimed {
            return "task_claim_conflict"
        }
        return nil
    }
}
