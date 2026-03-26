import Foundation

struct TaskDrawerModel: Equatable, Sendable {
    let automationMode: TaskBoardAutomationModePayload?
    let claimState: ClaimState
    let trackerBindingState: String?
    let dispatchState: DispatchState
    let dispatchReason: String?
    let retryState: String?
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
    let workflowBindingSummary: String
    let lineageSummary: String
    let reviewChecklist: [String]
    let allowedAgents: [String]
    let lastGoodHash: String?
    let lastReloadAt: String?
    let lastError: String?
    let lastBootstrapOutcome: String?
    let bootstrapPhaseSummary: String?
    let renderedPromptPath: String?
    let hookPhaseResults: [WorkflowStatusPayload.HookPhaseResult]
    let timeline: [TaskTimelineEntry]
    let primaryActionTitle: String

    static func resolve(
        from snapshot: AppShellSnapshot,
        workflowStatus: WorkflowStatusPayload?,
        timeline: [TaskTimelineEntry] = [],
        targetTaskID: String? = nil,
    ) -> Self? {
        let focusedSession = targetTaskID.flatMap { taskID in
            snapshot.sessions.first(where: { $0.taskID == taskID })
        } ?? snapshot.sessions.first(where: {
            $0.focusState == .focused || snapshot.app.focusedSessionID == $0.sessionID
        }) ?? snapshot.sessions.first

        guard let session = focusedSession, let taskID = session.taskID else {
            return nil
        }
        let retryState = snapshot.retryQueue
            .first(where: { $0.taskID == taskID })
            .map { "attempt \($0.attempt) · due \($0.dueAt ?? "pending")" }

        return Self(
            automationMode: session.automationMode,
            claimState: session.claimState,
            trackerBindingState: session.trackerBindingState,
            dispatchState: session.dispatchState,
            dispatchReason: session.dispatchReason,
            retryState: retryState,
            requireReview: workflowStatus?.workflow?.requireReview ?? false,
            maxRuntimeMinutes: workflowStatus?.workflow?.maxRuntimeMinutes,
            unsafeOverridePolicy: workflowStatus?.workflow?.unsafeOverridePolicy,
            blockerReason: blockerReason(
                automationMode: session.automationMode,
                claimState: session.claimState,
                workflowStatus: workflowStatus,
            ),
            taskID: taskID,
            sessionID: session.sessionID,
            sessionTitle: session.title,
            workspaceRoot: session.workspaceRoot,
            baseRoot: session.baseRoot,
            workflowName: workflowStatus?.workflow?.name ?? "Workflow Contract",
            workflowPath: workflowStatus?.path ?? "",
            strategy: workflowStatus?.workflow?.strategy,
            workflowBindingSummary: workflowBindingSummary(workflowStatus: workflowStatus),
            lineageSummary: lineageSummary(session: session),
            reviewChecklist: workflowStatus?.workflow?.reviewChecklist ?? [],
            allowedAgents: workflowStatus?.workflow?.allowedAgents ?? [],
            lastGoodHash: workflowStatus?.lastGoodHash,
            lastReloadAt: workflowStatus?.lastReloadAt,
            lastError: workflowStatus?.lastError,
            lastBootstrapOutcome: workflowStatus?.lastBootstrap?.outcomeCode,
            bootstrapPhaseSummary: workflowStatus?.lastBootstrap?.phaseSequence.joined(
                separator: " -> ",
            ),
            renderedPromptPath: workflowStatus?.lastBootstrap?.renderedPromptPath,
            hookPhaseResults: workflowStatus?.lastBootstrap?.hookPhaseResults ?? [],
            timeline: timeline,
            primaryActionTitle: "Detach Session",
        )
    }

    private static func blockerReason(
        automationMode: TaskBoardAutomationModePayload?,
        claimState: ClaimState,
        workflowStatus: WorkflowStatusPayload?,
    ) -> String? {
        if let automationMode, automationMode == .manual {
            return "manual_mode"
        }
        if workflowStatus?.state == .invalidKeptLastGood {
            return "workflow_invalid"
        }
        if let allowedAgents = workflowStatus?.workflow?.allowedAgents,
           !allowedAgents.isEmpty,
           !allowedAgents.contains("codex")
        {
            return "task_not_eligible_for_dispatch"
        }
        if claimState == .claimed {
            return "task_claim_conflict"
        }
        return nil
    }

    private static func workflowBindingSummary(workflowStatus: WorkflowStatusPayload?) -> String {
        var parts: [String] = []
        if let state = workflowStatus?.state.rawValue {
            parts.append(state)
        }
        if let lastGoodHash = workflowStatus?.lastGoodHash {
            parts.append("last good \(lastGoodHash)")
        }
        if let lastReloadAt = workflowStatus?.lastReloadAt {
            parts.append("reloaded \(lastReloadAt)")
        }
        return parts.isEmpty ? "none" : parts.joined(separator: " · ")
    }

    private static func lineageSummary(session: AppShellSnapshot.SessionSummary) -> String {
        [
            "session \(session.sessionID)",
            session.workspaceRoot.map { "workspace \($0)" } ?? "workspace none",
            session.taskID.map { "task \($0)" } ?? "task none",
        ].joined(separator: " · ")
    }
}
