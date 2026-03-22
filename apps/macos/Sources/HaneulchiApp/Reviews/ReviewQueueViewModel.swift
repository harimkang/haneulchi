import Foundation
import HCCoreFFI

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
        let warnings: [String]
        let evidenceManifestPath: String?

        enum CodingKeys: String, CodingKey {
            case taskID = "task_id"
            case projectID = "project_id"
            case title
            case summary
            case touchedFiles = "touched_files"
            case diffSummary = "diff_summary"
            case testsSummary = "tests_summary"
            case commandSummary = "command_summary"
            case warnings
            case evidenceManifestPath = "evidence_manifest_path"
        }

        var id: String { taskID }
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

    let loadProjection: @Sendable () throws -> ReviewQueueProjectionPayload

    init(
        loadProjection: @escaping @Sendable () throws -> ReviewQueueProjectionPayload = liveLoadProjection
    ) {
        self.loadProjection = loadProjection
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
    }

    func select(taskID: String) {
        selectedTaskID = taskID
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
