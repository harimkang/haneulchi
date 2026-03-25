import Foundation

enum WorkflowPresenceState: Equatable, Sendable {
    case none
    case present
    case unreadable
}

struct WorkflowPresenceProbe: Sendable {
    let probe: @Sendable (String?) async -> WorkflowPresenceState

    static func mock(_ state: WorkflowPresenceState) -> Self {
        Self(probe: { _ in state })
    }

    static let live = Self(
        probe: { rootPath in
            guard let rootPath else {
                return .none
            }

            let fileURL = URL(fileURLWithPath: rootPath).appendingPathComponent("WORKFLOW.md")
            guard FileManager.default.fileExists(atPath: fileURL.path) else {
                return .none
            }

            return FileManager.default.isReadableFile(atPath: fileURL.path) ? .present : .unreadable
        },
    )
}
