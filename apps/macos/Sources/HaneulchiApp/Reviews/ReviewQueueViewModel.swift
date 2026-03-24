import Foundation
import HCCoreFFI

enum ReviewDecisionCommand: String, Sendable {
    case accept
    case requestChanges = "request_changes"
    case manualContinue = "manual_continue"
    case followUp = "follow_up"
}

struct ReviewQueueProjectionPayload: Decodable, Equatable, Sendable {
    struct Item: Decodable, Equatable, Identifiable, Sendable {
        let taskID: String
        let projectID: String
        let title: String
        let summary: String
        let touchedFiles: [String]
        let diffSummary: String?
        let testsSummary: String?
        let commandSummary: String?
        let hookSummary: String?
        let evidenceSummary: String?
        let checklistSummary: String?
        let warnings: [String]
        let evidenceManifestPath: String?
        let ciRunURL: String?
        let prURL: String?
        let timeline: [TaskTimelineEntry]

        enum CodingKeys: String, CodingKey {
            case taskID = "task_id"
            case projectID = "project_id"
            case title
            case summary
            case touchedFiles = "touched_files"
            case diffSummary = "diff_summary"
            case testsSummary = "tests_summary"
            case commandSummary = "command_summary"
            case hookSummary = "hook_summary"
            case evidenceSummary = "evidence_summary"
            case checklistSummary = "checklist_summary"
            case warnings
            case evidenceManifestPath = "evidence_manifest_path"
            case ciRunURL = "ci_run_url"
            case prURL = "pr_url"
            case timeline
        }

        var id: String { taskID }

        init(
            taskID: String,
            projectID: String,
            title: String,
            summary: String,
            touchedFiles: [String],
            diffSummary: String?,
            testsSummary: String?,
            commandSummary: String?,
            hookSummary: String? = nil,
            evidenceSummary: String? = nil,
            checklistSummary: String? = nil,
            warnings: [String],
            evidenceManifestPath: String?,
            ciRunURL: String? = nil,
            prURL: String? = nil,
            timeline: [TaskTimelineEntry] = []
        ) {
            self.taskID = taskID
            self.projectID = projectID
            self.title = title
            self.summary = summary
            self.touchedFiles = touchedFiles
            self.diffSummary = diffSummary
            self.testsSummary = testsSummary
            self.commandSummary = commandSummary
            self.hookSummary = hookSummary
            self.evidenceSummary = evidenceSummary
            self.checklistSummary = checklistSummary
            self.warnings = warnings
            self.evidenceManifestPath = evidenceManifestPath
            self.ciRunURL = ciRunURL
            self.prURL = prURL
            self.timeline = timeline
        }
    }

    let items: [Item]
    let degradedReason: String?

    enum CodingKeys: String, CodingKey {
        case items
        case degradedReason = "degraded_reason"
    }
}

@MainActor
final class ReviewQueueViewModel: ObservableObject {
    @Published private(set) var items: [ReviewQueueProjectionPayload.Item] = []
    @Published private(set) var selectedTaskID: String?
    @Published private(set) var degradedReason: String?
    @Published private(set) var actionError: String?

    let loadProjection: @Sendable () throws -> ReviewQueueProjectionPayload
    let applyDecision: @Sendable (String, ReviewDecisionCommand) throws -> Void

    init(
        loadProjection: @escaping @Sendable () throws -> ReviewQueueProjectionPayload = liveLoadProjection,
        applyDecision: @escaping @Sendable (String, ReviewDecisionCommand) throws -> Void = liveApplyDecision
    ) {
        self.loadProjection = loadProjection
        self.applyDecision = applyDecision
    }

    var selectedItem: ReviewQueueProjectionPayload.Item? {
        items.first(where: { $0.taskID == selectedTaskID }) ?? items.first
    }

    var emptyStateMessage: String {
        "No review-ready tasks."
    }

    func reload() throws {
        let projection = try loadProjection()
        items = projection.items
        degradedReason = projection.degradedReason
        selectedTaskID = projection.items.first?.taskID
        actionError = nil
    }

    func select(taskID: String) {
        selectedTaskID = taskID
    }

    func apply(_ command: ReviewDecisionCommand) throws {
        guard let taskID = selectedItem?.taskID else {
            return
        }
        do {
            try applyDecision(taskID, command)
            try reload()
        } catch {
            actionError = String(describing: error)
            throw error
        }
    }
}

private enum ReviewQueueBridgeError: Error, Equatable {
    case invalidPayload
    case invalidProjection
    case operationFailed(String)
}

private struct ReviewQueueBridgeErrorPayload: Decodable {
    let error: String?
}

private struct ReviewQueueCStringBox {
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

private func reviewQueueStringPayloadData(_ payload: HcString) throws -> Data {
    defer { hc_string_free(payload) }

    guard let pointer = payload.ptr, let json = String(validatingCString: pointer) else {
        throw ReviewQueueBridgeError.invalidPayload
    }

    if
        let data = json.data(using: .utf8),
        let response = try? JSONDecoder().decode(ReviewQueueBridgeErrorPayload.self, from: data),
        let error = response.error
    {
        throw ReviewQueueBridgeError.operationFailed(error)
    }

    return Data(json.utf8)
}

private func liveLoadProjection() throws -> ReviewQueueProjectionPayload {
    let payload = try reviewQueueStringPayloadData(hc_review_queue_json())
    guard let projection = try? JSONDecoder().decode(ReviewQueueProjectionPayload.self, from: payload) else {
        throw ReviewQueueBridgeError.invalidProjection
    }
    return projection
}

private func liveApplyDecision(taskID: String, command: ReviewDecisionCommand) throws {
    let taskCString = ReviewQueueCStringBox(taskID)
    let decisionCString = ReviewQueueCStringBox(command.rawValue)
    _ = try taskCString.withPointer { taskPointer in
        try decisionCString.withPointer { decisionPointer in
            try reviewQueueStringPayloadData(hc_review_decision_json(taskPointer, decisionPointer))
        }
    }
}
