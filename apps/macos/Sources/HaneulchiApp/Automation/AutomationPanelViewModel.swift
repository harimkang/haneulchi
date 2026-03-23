import Foundation

struct AutomationPanelViewModel: Equatable, Sendable {
    let cadenceLabel: String
    let lastTickLabel: String
    let lastReconcileLabel: String
    let nextTickLabel: String
    let slotLabel: String
    let workflowHealth: String
    let trackerHealth: String
    let queueLabel: String
    let paused: Bool

    init(snapshot: AppShellSnapshot) {
        cadenceLabel = "\(snapshot.ops.cadenceMs)ms"
        lastTickLabel = snapshot.ops.lastTickAt ?? "none"
        lastReconcileLabel = snapshot.ops.lastReconcileAt ?? "none"
        nextTickLabel = snapshot.ops.lastTickAt.map { _ in "scheduled" } ?? "pending"
        slotLabel = "\(snapshot.ops.runningSlots)/\(snapshot.ops.maxSlots)"
        workflowHealth = snapshot.ops.workflowHealth.rawValue
        trackerHealth = snapshot.ops.trackerHealth
        queueLabel = "\(snapshot.ops.retryQueueCount) retry · \(snapshot.ops.queuedClaimCount) claimed"
        paused = snapshot.ops.paused
    }
}
