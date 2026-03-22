import Foundation

struct AutomationPanelViewModel: Equatable, Sendable {
    let cadenceLabel: String
    let slotLabel: String
    let workflowHealth: String
    let trackerHealth: String
    let queueLabel: String
    let paused: Bool

    init(snapshot: AppShellSnapshot) {
        cadenceLabel = "\(snapshot.ops.cadenceMs)ms"
        slotLabel = "\(snapshot.ops.runningSlots)/\(snapshot.ops.maxSlots)"
        workflowHealth = snapshot.ops.workflowHealth.rawValue
        trackerHealth = snapshot.ops.trackerHealth
        queueLabel = "\(snapshot.ops.retryQueueCount) retry · \(snapshot.ops.queuedClaimCount) claimed"
        paused = snapshot.ops.paused
    }
}
