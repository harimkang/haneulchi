import Foundation

enum WarningFlag: String, Codable, Equatable, Sendable {
    case unread
    case degraded
    case failed
}

enum ProjectStatus: String, Codable, Equatable, Sendable {
    case active
    case idle
    case error
}

enum SessionMode: String, Codable, Equatable, Sendable {
    case generic
    case preset
    case structuredAdapter = "structured_adapter"
}

enum SessionRuntimeState: String, Codable, Equatable, Sendable {
    case launching
    case running
    case waitingInput = "waiting_input"
    case reviewReady = "review_ready"
    case blocked
    case done
    case error
    case exited
}

enum ManualControlState: String, Codable, Equatable, Sendable {
    case none
    case takeover
    case releasePending = "release_pending"
    case frozen
}

enum DispatchState: String, Codable, Equatable, Sendable {
    case notDispatchable = "not_dispatchable"
    case dispatchable
    case dispatchInFlight = "dispatch_in_flight"
    case dispatchFailed = "dispatch_failed"
}

enum WorkflowHealth: String, Codable, Equatable, Sendable {
    case none
    case ok
    case invalidKeptLastGood = "invalid_kept_last_good"
    case reloadPending = "reload_pending"
}

struct AppShellSnapshot: Equatable, Sendable {
    struct Meta: Equatable, Sendable {
        let snapshotRev: Int
        let runtimeRev: Int
        let projectionRev: Int
        let snapshotAt: Date
    }

    struct OpsSummary: Equatable, Sendable {
        let runningSlots: Int
        let maxSlots: Int
        let retryQueueCount: Int
        let workflowHealth: WorkflowHealth
    }

    struct AppState: Equatable, Sendable {
        let activeRoute: Route
        let focusedSessionID: String?
        let degradedFlags: [WarningFlag]
    }

    struct ProjectSummary: Equatable, Identifiable, Sendable {
        let projectID: String
        let name: String
        let rootPath: String
        let status: ProjectStatus
        let workflowState: WorkflowHealth
        let sessionCount: Int
        let attentionCount: Int

        var id: String { projectID }
    }

    struct SessionSummary: Equatable, Identifiable, Sendable {
        let sessionID: String
        let title: String
        let currentDirectory: String?
        let mode: SessionMode
        let runtimeState: SessionRuntimeState
        let manualControlState: ManualControlState
        let dispatchState: DispatchState
        let unreadCount: Int

        var id: String { sessionID }
    }

    struct AttentionSummary: Equatable, Identifiable, Sendable {
        let attentionID: String
        let headline: String
        let severity: WarningFlag
        let targetRoute: Route
        let targetSessionID: String?

        var id: String { attentionID }
    }

    struct RetryQueueSummary: Equatable, Identifiable, Sendable {
        let retryID: String
        let taskID: String?
        let sessionID: String?
        let dueAt: Date

        var id: String { retryID }
    }

    struct WarningSummary: Equatable, Identifiable, Sendable {
        let warningID: String
        let severity: WarningFlag
        let headline: String
        let nextAction: String?

        var id: String { warningID }
    }

    let meta: Meta
    let ops: OpsSummary
    let app: AppState
    let projects: [ProjectSummary]
    let sessions: [SessionSummary]
    let attention: [AttentionSummary]
    let retryQueue: [RetryQueueSummary]
    let warnings: [WarningSummary]

    static func empty(activeRoute: Route, selectedProject: LauncherProject? = nil) -> Self {
        let projects = selectedProject.map {
            [
                ProjectSummary(
                    projectID: $0.projectID,
                    name: $0.name,
                    rootPath: $0.rootPath,
                    status: .active,
                    workflowState: .none,
                    sessionCount: 0,
                    attentionCount: 0
                )
            ]
        } ?? []

        return Self(
            meta: .init(snapshotRev: 1, runtimeRev: 1, projectionRev: 1, snapshotAt: .now),
            ops: .init(runningSlots: 0, maxSlots: 1, retryQueueCount: 0, workflowHealth: .none),
            app: .init(activeRoute: activeRoute, focusedSessionID: nil, degradedFlags: []),
            projects: projects,
            sessions: [],
            attention: [],
            retryQueue: [],
            warnings: []
        )
    }
}
