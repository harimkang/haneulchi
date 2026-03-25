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

enum ClaimState: String, Codable, Equatable, Sendable {
    case none
    case claimed
    case released
    case stale
}

enum SessionFocusState: String, Codable, Equatable, Sendable {
    case focused
    case background
}

struct WorkflowRuntimeStatus: Decodable, Equatable, Sendable {
    let state: WorkflowHealth
    let path: String
    let lastGoodHash: String?
    let lastReloadAt: String?
    let lastError: String?

    enum CodingKeys: String, CodingKey {
        case state
        case path
        case lastGoodHash = "last_good_hash"
        case lastReloadAt = "last_reload_at"
        case lastError = "last_error"
    }
}

struct TrackerStatus: Decodable, Equatable, Sendable {
    let state: String
    let lastSyncAt: String?
    let health: String

    enum CodingKeys: String, CodingKey {
        case state
        case lastSyncAt = "last_sync_at"
        case health
    }
}

struct AppShellSnapshot: Decodable, Equatable, Sendable {
    struct Meta: Decodable, Equatable, Sendable {
        let snapshotRev: Int
        let runtimeRev: Int
        let projectionRev: Int
        let snapshotAt: Date

        enum CodingKeys: String, CodingKey {
            case snapshotRev = "snapshot_rev"
            case runtimeRev = "runtime_rev"
            case projectionRev = "projection_rev"
            case snapshotAt = "snapshot_at"
        }

        init(snapshotRev: Int, runtimeRev: Int, projectionRev: Int, snapshotAt: Date) {
            self.snapshotRev = snapshotRev
            self.runtimeRev = runtimeRev
            self.projectionRev = projectionRev
            self.snapshotAt = snapshotAt
        }

        init(from decoder: Decoder) throws {
            let container = try decoder.container(keyedBy: CodingKeys.self)
            snapshotRev = try container.decode(Int.self, forKey: .snapshotRev)
            runtimeRev = try container.decode(Int.self, forKey: .runtimeRev)
            projectionRev = try container.decode(Int.self, forKey: .projectionRev)
            snapshotAt = try SnapshotDateParser.decodeRequiredDate(
                from: container,
                forKey: .snapshotAt,
            )
        }
    }

    struct OpsSummary: Decodable, Equatable, Sendable {
        let cadenceMs: Int
        let lastTickAt: String?
        let lastReconcileAt: String?
        let runningSlots: Int
        let maxSlots: Int
        let retryQueueCount: Int
        let queuedClaimCount: Int
        let workflowHealth: WorkflowHealth
        let trackerHealth: String
        let paused: Bool

        enum CodingKeys: String, CodingKey {
            case cadenceMs = "cadence_ms"
            case lastTickAt = "last_tick_at"
            case lastReconcileAt = "last_reconcile_at"
            case runningSlots = "running_slots"
            case maxSlots = "max_slots"
            case retryQueueCount = "retry_queue_count"
            case retryDueCount = "retry_due_count"
            case queuedClaimCount = "queued_claim_count"
            case workflowHealth = "workflow_health"
            case trackerHealth = "tracker_health"
            case paused
        }

        init(
            cadenceMs: Int = 15000,
            lastTickAt: String? = nil,
            lastReconcileAt: String? = nil,
            runningSlots: Int,
            maxSlots: Int,
            retryQueueCount: Int,
            queuedClaimCount: Int = 0,
            workflowHealth: WorkflowHealth,
            trackerHealth: String = "ok",
            paused: Bool = false,
        ) {
            self.cadenceMs = cadenceMs
            self.lastTickAt = lastTickAt
            self.lastReconcileAt = lastReconcileAt
            self.runningSlots = runningSlots
            self.maxSlots = maxSlots
            self.retryQueueCount = retryQueueCount
            self.queuedClaimCount = queuedClaimCount
            self.workflowHealth = workflowHealth
            self.trackerHealth = trackerHealth
            self.paused = paused
        }

        init(from decoder: Decoder) throws {
            let container = try decoder.container(keyedBy: CodingKeys.self)
            cadenceMs = try container.decodeIfPresent(Int.self, forKey: .cadenceMs) ?? 15000
            lastTickAt = try container.decodeIfPresent(String.self, forKey: .lastTickAt)
            lastReconcileAt = try container.decodeIfPresent(String.self, forKey: .lastReconcileAt)
            runningSlots = try container.decode(Int.self, forKey: .runningSlots)
            maxSlots = try container.decode(Int.self, forKey: .maxSlots)
            retryQueueCount =
                try container.decodeIfPresent(Int.self, forKey: .retryQueueCount)
                    ?? container.decodeIfPresent(Int.self, forKey: .retryDueCount)
                    ?? 0
            queuedClaimCount = try container
                .decodeIfPresent(Int.self, forKey: .queuedClaimCount) ?? 0
            workflowHealth = try container.decodeIfPresent(
                WorkflowHealth.self,
                forKey: .workflowHealth,
            ) ?? .none
            trackerHealth = try container.decodeIfPresent(String.self, forKey: .trackerHealth) ?? "ok"
            paused = try container.decodeIfPresent(Bool.self, forKey: .paused) ?? false
        }
    }

    struct AppState: Decodable, Equatable, Sendable {
        let activeRoute: Route
        let focusedSessionID: String?
        let degradedFlags: [WarningFlag]

        enum CodingKeys: String, CodingKey {
            case activeRoute = "active_route"
            case focusedSessionID = "focused_session_id"
            case degradedFlags = "degraded_flags"
        }
    }

    struct ProjectSummary: Decodable, Equatable, Identifiable, Sendable {
        let projectID: String
        let name: String
        let rootPath: String
        let status: ProjectStatus
        let workflowState: WorkflowHealth
        let sessionCount: Int
        let taskCounts: [String: Int]
        let attentionCount: Int

        enum CodingKeys: String, CodingKey {
            case projectID = "project_id"
            case name
            case rootPath = "root_path"
            case status
            case workflowState = "workflow_state"
            case sessionCount = "session_count"
            case taskCounts = "task_counts"
            case attentionCount = "attention_count"
        }

        init(
            projectID: String,
            name: String,
            rootPath: String,
            status: ProjectStatus,
            workflowState: WorkflowHealth,
            sessionCount: Int,
            attentionCount: Int,
            taskCounts: [String: Int] = [:],
        ) {
            self.projectID = projectID
            self.name = name
            self.rootPath = rootPath
            self.status = status
            self.workflowState = workflowState
            self.sessionCount = sessionCount
            self.taskCounts = taskCounts
            self.attentionCount = attentionCount
        }

        var id: String {
            projectID
        }
    }

    struct SessionSummary: Decodable, Equatable, Identifiable, Sendable {
        let sessionID: String
        let projectID: String?
        let taskID: String?
        let automationMode: TaskBoardAutomationModePayload?
        let trackerBindingState: String?
        let title: String
        let currentDirectory: String?
        let workspaceRoot: String?
        let baseRoot: String?
        let branch: String?
        let latestSummary: String?
        let providerID: String?
        let modelID: String?
        let dispatchReason: String?
        let latestCommentary: String?
        let commentaryUpdatedAt: String?
        let activeWindowTitle: String?
        let mode: SessionMode
        let runtimeState: SessionRuntimeState
        let manualControlState: ManualControlState
        let dispatchState: DispatchState
        let claimState: ClaimState
        let adapterKind: String?
        let unreadCount: Int
        let lastActivityAt: String?
        let focusState: SessionFocusState
        let canFocus: Bool
        let canTakeover: Bool
        let canReleaseTakeover: Bool

        enum CodingKeys: String, CodingKey {
            case sessionID = "session_id"
            case projectID = "project_id"
            case taskID = "task_id"
            case automationMode = "automation_mode"
            case trackerBindingState = "tracker_binding_state"
            case title
            case currentDirectory = "cwd"
            case workspaceRoot = "workspace_root"
            case baseRoot = "base_root"
            case branch
            case latestSummary = "latest_summary"
            case providerID = "provider_id"
            case modelID = "model_id"
            case dispatchReason = "dispatch_reason"
            case latestCommentary = "latest_commentary"
            case commentaryUpdatedAt = "commentary_updated_at"
            case activeWindowTitle = "active_window_title"
            case mode
            case runtimeState = "runtime_state"
            case manualControlState = "manual_control"
            case dispatchState = "dispatch_state"
            case claimState = "claim_state"
            case adapterKind = "adapter_kind"
            case unreadCount = "unread_count"
            case lastActivityAt = "last_activity_at"
            case focusState = "focus_state"
            case canFocus = "can_focus"
            case canTakeover = "can_takeover"
            case canReleaseTakeover = "can_release_takeover"
        }

        init(
            sessionID: String,
            title: String,
            currentDirectory: String?,
            mode: SessionMode,
            runtimeState: SessionRuntimeState,
            manualControlState: ManualControlState,
            dispatchState: DispatchState,
            unreadCount: Int,
            projectID: String? = nil,
            taskID: String? = nil,
            automationMode: TaskBoardAutomationModePayload? = nil,
            trackerBindingState: String? = nil,
            workspaceRoot: String? = nil,
            baseRoot: String? = nil,
            branch: String? = nil,
            latestSummary: String? = nil,
            providerID: String? = nil,
            modelID: String? = nil,
            dispatchReason: String? = nil,
            latestCommentary: String? = nil,
            commentaryUpdatedAt: String? = nil,
            activeWindowTitle: String? = nil,
            claimState: ClaimState = .none,
            adapterKind: String? = nil,
            lastActivityAt: String? = nil,
            focusState: SessionFocusState = .background,
            canFocus: Bool = true,
            canTakeover: Bool = false,
            canReleaseTakeover: Bool = false,
        ) {
            self.sessionID = sessionID
            self.projectID = projectID
            self.taskID = taskID
            self.automationMode = automationMode
            self.trackerBindingState = trackerBindingState
            self.title = title
            self.currentDirectory = currentDirectory
            self.workspaceRoot = workspaceRoot
            self.baseRoot = baseRoot
            self.branch = branch
            self.latestSummary = latestSummary
            self.providerID = providerID
            self.modelID = modelID
            self.dispatchReason = dispatchReason
            self.latestCommentary = latestCommentary
            self.commentaryUpdatedAt = commentaryUpdatedAt
            self.activeWindowTitle = activeWindowTitle
            self.mode = mode
            self.runtimeState = runtimeState
            self.manualControlState = manualControlState
            self.dispatchState = dispatchState
            self.claimState = claimState
            self.adapterKind = adapterKind
            self.unreadCount = unreadCount
            self.lastActivityAt = lastActivityAt
            self.focusState = focusState
            self.canFocus = canFocus
            self.canTakeover = canTakeover
            self.canReleaseTakeover = canReleaseTakeover
        }

        var id: String {
            sessionID
        }
    }

    struct AttentionSummary: Decodable, Equatable, Identifiable, Sendable {
        let attentionID: String
        let headline: String
        let severity: WarningFlag
        let targetRoute: Route
        let targetSessionID: String?
        let projectID: String?
        let taskID: String?
        let summary: String?
        let createdAt: String?
        let actionHint: String?

        enum CodingKeys: String, CodingKey {
            case attentionID = "attention_id"
            case headline
            case severity
            case targetRoute = "target_route"
            case targetSessionID = "target_session_id"
            case projectID = "project_id"
            case taskID = "task_id"
            case summary
            case createdAt = "created_at"
            case actionHint = "action_hint"
            case title
        }

        init(
            attentionID: String,
            headline: String,
            severity: WarningFlag,
            targetRoute: Route,
            targetSessionID: String?,
            projectID: String? = nil,
            taskID: String? = nil,
            summary: String? = nil,
            createdAt: String? = nil,
            actionHint: String? = nil,
        ) {
            self.attentionID = attentionID
            self.headline = headline
            self.severity = severity
            self.targetRoute = targetRoute
            self.targetSessionID = targetSessionID
            self.projectID = projectID
            self.taskID = taskID
            self.summary = summary
            self.createdAt = createdAt
            self.actionHint = actionHint
        }

        init(from decoder: Decoder) throws {
            let container = try decoder.container(keyedBy: CodingKeys.self)
            attentionID = try container.decode(String.self, forKey: .attentionID)
            headline = try container.decodeIfPresent(String.self, forKey: .headline)
                ?? container.decode(String.self, forKey: .title)
            severity = try container.decode(WarningFlag.self, forKey: .severity)
            targetRoute = try container
                .decodeIfPresent(Route.self, forKey: .targetRoute) ?? .attentionCenter
            targetSessionID = try container.decodeIfPresent(String.self, forKey: .targetSessionID)
            projectID = try container.decodeIfPresent(String.self, forKey: .projectID)
            taskID = try container.decodeIfPresent(String.self, forKey: .taskID)
            summary = try container.decodeIfPresent(String.self, forKey: .summary)
            createdAt = try container.decodeIfPresent(String.self, forKey: .createdAt)
            actionHint = try container.decodeIfPresent(String.self, forKey: .actionHint)
        }

        var id: String {
            attentionID
        }
    }

    struct RetryQueueSummary: Decodable, Equatable, Identifiable, Sendable {
        let taskID: String
        let projectID: String
        let attempt: Int
        let reasonCode: String
        let dueAt: String?
        let backoffMs: Int
        let claimState: ClaimState

        enum CodingKeys: String, CodingKey {
            case taskID = "task_id"
            case projectID = "project_id"
            case attempt
            case reasonCode = "reason_code"
            case dueAt = "due_at"
            case backoffMs = "backoff_ms"
            case claimState = "claim_state"
        }

        init(
            taskID: String,
            projectID: String,
            attempt: Int,
            reasonCode: String,
            dueAt: String?,
            backoffMs: Int,
            claimState: ClaimState = .none,
        ) {
            self.taskID = taskID
            self.projectID = projectID
            self.attempt = attempt
            self.reasonCode = reasonCode
            self.dueAt = dueAt
            self.backoffMs = backoffMs
            self.claimState = claimState
        }

        init(from decoder: Decoder) throws {
            let container = try decoder.container(keyedBy: CodingKeys.self)
            taskID = try container.decode(String.self, forKey: .taskID)
            projectID = try container.decode(String.self, forKey: .projectID)
            attempt = try container.decode(Int.self, forKey: .attempt)
            reasonCode = try container.decode(String.self, forKey: .reasonCode)
            dueAt = try container.decodeIfPresent(String.self, forKey: .dueAt)
            backoffMs = try container.decode(Int.self, forKey: .backoffMs)
            claimState = try container
                .decodeIfPresent(ClaimState.self, forKey: .claimState) ?? .none
        }

        var id: String {
            "\(projectID):\(taskID):\(attempt)"
        }
    }

    struct RecentArtifactSummary: Decodable, Equatable, Identifiable, Sendable {
        let taskID: String
        let projectID: String
        let summary: String
        let jumpTarget: String
        let manifestPath: String?

        enum CodingKeys: String, CodingKey {
            case taskID = "task_id"
            case projectID = "project_id"
            case summary
            case jumpTarget = "jump_target"
            case manifestPath = "manifest_path"
        }

        init(
            taskID: String,
            projectID: String,
            summary: String,
            jumpTarget: String,
            manifestPath: String?,
        ) {
            self.taskID = taskID
            self.projectID = projectID
            self.summary = summary
            self.jumpTarget = jumpTarget
            self.manifestPath = manifestPath
        }

        var id: String {
            "\(projectID):\(taskID):\(jumpTarget)"
        }
    }

    struct WarningSummary: Decodable, Equatable, Identifiable, Sendable {
        let warningID: String
        let severity: WarningFlag
        let headline: String
        let nextAction: String?

        enum CodingKeys: String, CodingKey {
            case warningID = "warning_id"
            case severity
            case headline
            case nextAction = "next_action"
        }

        var id: String {
            warningID
        }
    }

    let meta: Meta
    let ops: OpsSummary
    let app: AppState
    let projects: [ProjectSummary]
    let sessions: [SessionSummary]
    let attention: [AttentionSummary]
    let retryQueue: [RetryQueueSummary]
    let warnings: [WarningSummary]
    let workflow: WorkflowRuntimeStatus?
    let tracker: TrackerStatus?
    let recentArtifacts: [RecentArtifactSummary]

    enum CodingKeys: String, CodingKey {
        case meta
        case ops
        case app
        case projects
        case sessions
        case attention
        case retryQueue = "retry_queue"
        case warnings
        case workflow
        case tracker
        case recentArtifacts = "recent_artifacts"
    }

    private enum OpsEnvelopeKeys: String, CodingKey {
        case automation
        case workflow
        case tracker
        case app
    }

    init(
        meta: Meta,
        ops: OpsSummary,
        app: AppState,
        projects: [ProjectSummary],
        sessions: [SessionSummary],
        attention: [AttentionSummary],
        retryQueue: [RetryQueueSummary],
        warnings: [WarningSummary],
        workflow: WorkflowRuntimeStatus? = nil,
        tracker: TrackerStatus? = nil,
        recentArtifacts: [RecentArtifactSummary] = [],
    ) {
        self.meta = meta
        self.ops = ops
        self.app = app
        self.projects = projects
        self.sessions = sessions
        self.attention = attention
        self.retryQueue = retryQueue
        self.warnings = warnings
        self.workflow = workflow
        self.tracker = tracker
        self.recentArtifacts = recentArtifacts
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        meta = try container.decode(Meta.self, forKey: .meta)

        if let opsContainer = try? container.nestedContainer(
            keyedBy: OpsEnvelopeKeys.self,
            forKey: .ops,
        ),
            opsContainer.contains(.automation)
        {
            let automation = try opsContainer.decode(OpsSummary.self, forKey: .automation)
            let decodedWorkflow = try opsContainer.decodeIfPresent(
                WorkflowRuntimeStatus.self,
                forKey: .workflow,
            )
            let decodedTracker = try opsContainer.decodeIfPresent(
                TrackerStatus.self,
                forKey: .tracker,
            )
            workflow = decodedWorkflow
            tracker = decodedTracker
            app = try opsContainer.decodeIfPresent(AppState.self, forKey: .app)
                ?? AppState(activeRoute: .projectFocus, focusedSessionID: nil, degradedFlags: [])
            ops = OpsSummary(
                cadenceMs: automation.cadenceMs,
                lastTickAt: automation.lastTickAt,
                lastReconcileAt: automation.lastReconcileAt,
                runningSlots: automation.runningSlots,
                maxSlots: automation.maxSlots,
                retryQueueCount: automation.retryQueueCount,
                queuedClaimCount: automation.queuedClaimCount,
                workflowHealth: decodedWorkflow?.state ?? automation.workflowHealth,
                trackerHealth: decodedTracker?.health ?? automation.trackerHealth,
                paused: automation.paused,
            )
        } else {
            ops = try container.decode(OpsSummary.self, forKey: .ops)
            workflow = try container.decodeIfPresent(WorkflowRuntimeStatus.self, forKey: .workflow)
            tracker = try container.decodeIfPresent(TrackerStatus.self, forKey: .tracker)
            app = try container.decode(AppState.self, forKey: .app)
        }

        projects = try container.decode([ProjectSummary].self, forKey: .projects)
        sessions = try container.decode([SessionSummary].self, forKey: .sessions)
        attention = try container.decode([AttentionSummary].self, forKey: .attention)
        retryQueue = try container.decode([RetryQueueSummary].self, forKey: .retryQueue)
        warnings = try container.decode([WarningSummary].self, forKey: .warnings)
        recentArtifacts =
            try container.decodeIfPresent([RecentArtifactSummary].self, forKey: .recentArtifacts)
                ?? []
    }

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
                    attentionCount: 0,
                ),
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
            warnings: [],
            workflow: nil,
            tracker: nil,
            recentArtifacts: [],
        )
    }
}

private enum SnapshotDateParser {
    static func decodeRequiredDate<K: CodingKey>(
        from container: KeyedDecodingContainer<K>,
        forKey key: K,
    ) throws -> Date {
        let rawValue = try container.decode(String.self, forKey: key)
        let formatter = ISO8601DateFormatter()
        guard let date = formatter.date(from: rawValue) else {
            throw DecodingError.dataCorruptedError(
                forKey: key,
                in: container,
                debugDescription: "Expected ISO8601 date string.",
            )
        }
        return date
    }
}
