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

    var stripMetrics: [HaneulchiMonolithMetric] {
        [
            .init(id: "cadence", label: "cadence", value: cadenceLabel, accent: .neutral),
            .init(id: "last_tick", label: "last tick", value: lastTickLabel, accent: .neutral),
            .init(id: "next_tick", label: "next tick", value: nextTickLabel, accent: .neutral),
            .init(id: "reconcile", label: "reconcile", value: lastReconcileLabel, accent: .neutral),
            .init(id: "slots", label: "slots", value: slotLabel, accent: .success),
            .init(id: "workflow", label: "workflow", value: workflowHealth, accent: workflowAccent),
            .init(id: "tracker", label: "tracker", value: trackerHealth, accent: trackerAccent),
            .init(id: "queue", label: "queue", value: queueLabel, accent: queueAccent),
            .init(
                id: "paused",
                label: "paused",
                value: paused ? "yes" : "no",
                accent: paused ? .warning : .neutral,
            ),
        ]
    }

    var primaryStripMetrics: [HaneulchiMonolithMetric] {
        Array(stripMetrics.prefix(6))
    }

    var secondaryStripMetrics: [HaneulchiMonolithMetric] {
        Array(stripMetrics.suffix(3))
    }

    private var workflowAccent: HaneulchiSignalAccent {
        if workflowHealth == "ok" || workflowHealth == "workflow_ok" {
            return .success
        }
        return .warning
    }

    private var trackerAccent: HaneulchiSignalAccent {
        let lower = trackerHealth.lowercased()
        if lower.contains("degraded") || lower.contains("local_only") {
            return .warning
        }
        if lower.contains("error") || lower.contains("failed") {
            return .error
        }
        return .success
    }

    private var queueAccent: HaneulchiSignalAccent {
        if queueLabel.hasPrefix("0 retry") {
            return .neutral
        }
        return .warning
    }
}
