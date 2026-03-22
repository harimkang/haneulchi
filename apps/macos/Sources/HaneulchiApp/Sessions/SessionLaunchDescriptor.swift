import Foundation

enum SessionLaunchMode: String, Equatable, Sendable {
    case generic
    case preset
    case isolated
}

struct WorkflowLaunchSummary: Equatable, Sendable {
    let name: String
    let strategy: String
    let baseRoot: String
    let reviewChecklist: [String]
    let allowedAgents: [String]
}

struct SessionLaunchDescriptor: Equatable, Sendable {
    let mode: SessionLaunchMode
    let title: String
    let presetID: String?
    let restoreBundle: TerminalRestoreBundle
    let workspaceRoot: String?
    let workflowSummary: WorkflowLaunchSummary?
}
