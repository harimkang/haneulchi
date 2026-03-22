import Foundation

struct TaskDrawerModel: Equatable, Sendable {
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
    let primaryActionTitle: String

    static func resolve(
        from snapshot: AppShellSnapshot,
        workflowStatus: WorkflowStatusPayload?
    ) -> Self? {
        let focusedSession = snapshot.sessions.first(where: {
            $0.focusState == .focused || snapshot.app.focusedSessionID == $0.sessionID
        }) ?? snapshot.sessions.first

        guard let session = focusedSession, let taskID = session.taskID else {
            return nil
        }

        return Self(
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
            primaryActionTitle: "Detach Session"
        )
    }
}
