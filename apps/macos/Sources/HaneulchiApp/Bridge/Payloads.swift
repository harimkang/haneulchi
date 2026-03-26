import Foundation

struct InventorySummaryPayload: Codable, Sendable {
    var total: Int
    var inUse: Int
    var recoverable: Int
    var safeToDelete: Int
    var stale: Int

    enum CodingKeys: String, CodingKey {
        case total
        case inUse = "in_use"
        case recoverable
        case safeToDelete = "safe_to_delete"
        case stale
    }
}

struct TerminalSettingsPayload: Codable, Sendable {
    var shell: String
    var defaultCols: Int
    var defaultRows: Int
    var scrollbackLines: Int
    var fontName: String
    var theme: String
    var cursorStyle: String

    enum CodingKeys: String, CodingKey {
        case shell
        case defaultCols = "default_cols"
        case defaultRows = "default_rows"
        case scrollbackLines = "scrollback_lines"
        case fontName = "font_name"
        case theme
        case cursorStyle = "cursor_style"
    }
}

struct RuntimeInfoSummaryPayload: Codable, Sendable {
    var socketPath: String?
    var transport: String
    var status: String

    enum CodingKeys: String, CodingKey {
        case socketPath = "socket_path"
        case transport
        case status
    }
}

struct DegradedIssuePayload: Codable, Sendable {
    var issueCode: String
    var details: String

    enum CodingKeys: String, CodingKey {
        case issueCode = "issue_code"
        case details
    }
}

struct InventoryRowPayload: Codable, Sendable {
    var worktreeId: String
    var taskId: String
    var path: String
    var projectName: String
    var branch: String?
    var disposition: String
    var isPinned: Bool
    var isDegraded: Bool
    var sizeBytes: Int?
    var lastAccessedAt: String?

    enum CodingKeys: String, CodingKey {
        case worktreeId = "worktree_id"
        case taskId = "task_id"
        case path
        case projectName = "project_name"
        case branch
        case disposition
        case isPinned = "is_pinned"
        case isDegraded = "is_degraded"
        case sizeBytes = "size_bytes"
        case lastAccessedAt = "last_accessed_at"
    }

    init(
        worktreeId: String,
        taskId: String = "",
        path: String,
        projectName: String,
        branch: String?,
        disposition: String,
        isPinned: Bool,
        isDegraded: Bool,
        sizeBytes: Int?,
        lastAccessedAt: String?,
    ) {
        self.worktreeId = worktreeId
        self.taskId = taskId
        self.path = path
        self.projectName = projectName
        self.branch = branch
        self.disposition = disposition
        self.isPinned = isPinned
        self.isDegraded = isDegraded
        self.sizeBytes = sizeBytes
        self.lastAccessedAt = lastAccessedAt
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        worktreeId = try container.decode(String.self, forKey: .worktreeId)
        taskId = try container.decodeIfPresent(String.self, forKey: .taskId) ?? ""
        path = try container.decode(String.self, forKey: .path)
        projectName = try container.decode(String.self, forKey: .projectName)
        branch = try container.decodeIfPresent(String.self, forKey: .branch)
        disposition = try container.decode(String.self, forKey: .disposition)
        isPinned = try container.decode(Bool.self, forKey: .isPinned)
        isDegraded = try container.decode(Bool.self, forKey: .isDegraded)
        sizeBytes = try container.decodeIfPresent(Int.self, forKey: .sizeBytes)
        lastAccessedAt = try container.decodeIfPresent(String.self, forKey: .lastAccessedAt)
    }
}

struct RecoveryContextPayload: Codable, Sendable {
    var workflowHealth: String
    var staleClaims: [String]

    enum CodingKeys: String, CodingKey {
        case workflowHealth = "workflow_health"
        case staleClaims = "stale_claims"
    }
}

struct AppStatePayload: Codable, Sendable {
    var activeRoute: String
    var lastProjectId: String?
    var lastSessionId: String?
    var savedAt: String?

    enum CodingKeys: String, CodingKey {
        case activeRoute = "active_route"
        case lastProjectId = "last_project_id"
        case lastSessionId = "last_session_id"
        case savedAt = "saved_at"
    }
}

struct RecoverableSessionPayload: Codable, Sendable {
    var sessionId: String
    var projectId: String
    var title: String
    var cwd: String
    var branch: String?
    var lastActiveAt: String?
    var isRecoverable: Bool

    enum CodingKeys: String, CodingKey {
        case sessionId = "session_id"
        case projectId = "project_id"
        case title
        case cwd
        case branch
        case lastActiveAt = "last_active_at"
        case isRecoverable = "is_recoverable"
    }
}

struct SessionDetailsPayload: Decodable, Sendable {
    struct RecentEvent: Decodable, Sendable {
        var kind: String
        var title: String
        var summary: String?
        var createdAt: String?
        var actionHint: String?

        enum CodingKeys: String, CodingKey {
            case kind
            case title
            case summary
            case createdAt = "created_at"
            case actionHint = "action_hint"
        }
    }

    var sessionID: String
    var title: String
    var workflowBinding: WorkflowRuntimeStatus
    var recentEvents: [RecentEvent]

    enum CodingKeys: String, CodingKey {
        case sessionID = "session_id"
        case title
        case workflowBinding = "workflow_binding"
        case recentEvents = "recent_events"
    }
}
