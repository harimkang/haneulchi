import Foundation
import HCCoreFFI

enum TaskBoardColumnID: String, Codable, CaseIterable, Equatable, Sendable {
    case inbox
    case ready
    case running
    case review
    case blocked
    case done

    var title: String {
        switch self {
        case .inbox: "Inbox"
        case .ready: "Ready"
        case .running: "Running"
        case .review: "Review"
        case .blocked: "Blocked"
        case .done: "Done"
        }
    }
}

enum TaskBoardAutomationModePayload: String, Codable, Equatable, Sendable {
    case manual
    case assisted
    case autoEligible = "auto_eligible"

    var label: String {
        switch self {
        case .manual: "manual"
        case .assisted: "assisted"
        case .autoEligible: "auto eligible"
        }
    }
}

struct TaskBoardProjectionPayload: Decodable, Equatable, Sendable {
    struct ProjectOption: Decodable, Equatable, Sendable {
        let projectID: String
        let taskCount: Int

        var title: String {
            projectID
        }

        enum CodingKeys: String, CodingKey {
            case projectID = "project_id"
            case taskCount = "task_count"
        }

        init(projectID: String, title _: String, taskCount: Int) {
            self.projectID = projectID
            self.taskCount = taskCount
        }
    }

    struct TaskCard: Decodable, Equatable, Identifiable, Sendable {
        let id: String
        let projectID: String
        let displayKey: String
        let title: String
        let description: String
        let priority: String
        let automationMode: TaskBoardAutomationModePayload
        let linkedSessionID: String?
        let column: TaskBoardColumnID

        enum CodingKeys: String, CodingKey {
            case id
            case projectID = "project_id"
            case displayKey = "display_key"
            case title
            case description
            case priority
            case automationMode = "automation_mode"
            case linkedSessionID = "linked_session_id"
            case column
        }

        init(
            id: String,
            projectID: String,
            displayKey: String,
            title: String,
            description: String,
            priority: String,
            automationMode: TaskBoardAutomationModePayload,
            linkedSessionID: String?,
            column: TaskBoardColumnID,
        ) {
            self.id = id
            self.projectID = projectID
            self.displayKey = displayKey
            self.title = title
            self.description = description
            self.priority = priority
            self.automationMode = automationMode
            self.linkedSessionID = linkedSessionID
            self.column = column
        }

        var evidenceReadinessLabel: String {
            column == .review ? "review_ready" : "not_ready"
        }

        var nextActionLabel: String {
            switch column {
            case .inbox:
                "triage task"
            case .ready:
                linkedSessionID == nil ? "claim or start session" : "open linked session"
            case .running:
                "follow active session"
            case .review:
                "open review queue"
            case .blocked:
                "resolve blocker"
            case .done:
                "archive when safe"
            }
        }

        var compactMetadataChips: [String] {
            [
                priority.uppercased(),
                evidenceReadinessLabel,
                automationMode.label,
            ]
        }

        var contextSummaryLabel: String {
            [projectID, linkedSessionID ?? "unassigned"].joined(separator: " · ")
        }
    }

    struct ColumnGroup: Decodable, Equatable, Sendable {
        let column: TaskBoardColumnID
        let tasks: [TaskCard]
    }

    let selectedProjectID: String?
    let projects: [ProjectOption]
    let columns: [ColumnGroup]

    enum CodingKeys: String, CodingKey {
        case selectedProjectID = "selected_project_id"
        case projects
        case columns
    }
}

@MainActor
final class TaskBoardViewModel: ObservableObject {
    struct ColumnModel: Equatable, Identifiable, Sendable {
        let column: TaskBoardColumnID
        let tasks: [TaskBoardProjectionPayload.TaskCard]

        var id: String {
            column.rawValue
        }

        var title: String {
            column.title
        }

        var taskCount: Int {
            tasks.count
        }
    }

    @Published private(set) var columns: [ColumnModel]
    @Published private(set) var projectOptions: [TaskBoardProjectionPayload.ProjectOption]
    @Published private(set) var selectedProjectID: String?
    @Published private(set) var errorMessage: String?

    private let loadProjection: @Sendable (String?) throws -> TaskBoardProjectionPayload
    private let moveTaskBridge: @Sendable (String, TaskBoardColumnID) throws
        -> TaskBoardProjectionPayload

    init(
        loadProjection: @escaping @Sendable (String?) throws
            -> TaskBoardProjectionPayload = liveLoadProjection,
        moveTask: @escaping @Sendable (String, TaskBoardColumnID) throws
            -> TaskBoardProjectionPayload = liveMoveTask,
    ) {
        self.loadProjection = loadProjection
        moveTaskBridge = moveTask
        columns = TaskBoardColumnID.allCases.map { ColumnModel(column: $0, tasks: []) }
        projectOptions = []
        selectedProjectID = nil
        errorMessage = nil
    }

    func reload() throws {
        try apply(loadProjection(selectedProjectID))
    }

    func selectProject(_ projectID: String?) throws {
        selectedProjectID = projectID
        try reload()
    }

    func moveTask(taskID: String, to column: TaskBoardColumnID) throws {
        let currentSelection = selectedProjectID
        try apply(moveTaskBridge(taskID, column))
        if currentSelection != selectedProjectID {
            selectedProjectID = currentSelection
            try reload()
        }
    }

    func present(error: Error) {
        errorMessage = String(describing: error)
    }

    private func apply(_ projection: TaskBoardProjectionPayload) {
        selectedProjectID = projection.selectedProjectID
        projectOptions = projection.projects
        errorMessage = nil
        columns = TaskBoardColumnID.allCases.map { column in
            let tasks = projection.columns.first(where: { $0.column == column })?.tasks ?? []
            return ColumnModel(column: column, tasks: tasks)
        }
    }
}

private enum TaskBoardBridgeError: Error, Equatable {
    case invalidPayload
    case invalidProjection
    case operationFailed(String)
}

private struct TaskBoardBridgeErrorPayload: Decodable {
    let error: String?
}

private struct TaskMoveBridgeResponse: Decodable {
    let task: TaskBoardProjectionPayload.TaskCard
}

private struct TaskBoardCStringBox {
    private let storage: [CChar]

    init(_ string: String) {
        storage = Array(string.utf8CString)
    }

    func withPointer<T>(_ body: (UnsafePointer<CChar>) throws -> T) rethrows -> T {
        try storage.withUnsafeBufferPointer { buffer in
            try body(buffer.baseAddress!)
        }
    }
}

private func taskBoardStringPayloadData(_ payload: HcString) throws -> Data {
    defer { hc_string_free(payload) }

    guard let pointer = payload.ptr, let json = String(validatingCString: pointer) else {
        throw TaskBoardBridgeError.invalidPayload
    }

    if
        let data = json.data(using: .utf8),
        let response = try? JSONDecoder().decode(TaskBoardBridgeErrorPayload.self, from: data),
        let error = response.error
    {
        throw TaskBoardBridgeError.operationFailed(error)
    }

    return Data(json.utf8)
}

private func decodeTaskBoardProjection(from data: Data) throws -> TaskBoardProjectionPayload {
    guard let projection = try? JSONDecoder().decode(TaskBoardProjectionPayload.self, from: data)
    else {
        throw TaskBoardBridgeError.invalidProjection
    }

    return projection
}

private func liveLoadProjection(projectID: String?) throws -> TaskBoardProjectionPayload {
    let payload: Data = if let projectID, !projectID.isEmpty {
        try TaskBoardCStringBox(projectID).withPointer { pointer in
            try taskBoardStringPayloadData(hc_task_board_json(pointer))
        }
    } else {
        try taskBoardStringPayloadData(hc_task_board_json(nil))
    }

    return try decodeTaskBoardProjection(from: payload)
}

private func liveMoveTask(taskID: String,
                          column: TaskBoardColumnID) throws -> TaskBoardProjectionPayload
{
    let task = TaskBoardCStringBox(taskID)
    let targetColumn = TaskBoardCStringBox(column.rawValue)
    let movedData = try task.withPointer { taskPointer in
        try targetColumn.withPointer { columnPointer in
            try taskBoardStringPayloadData(hc_task_move_json(taskPointer, columnPointer))
        }
    }
    _ = try JSONDecoder().decode(TaskMoveBridgeResponse.self, from: movedData)
    return try liveLoadProjection(projectID: nil)
}
