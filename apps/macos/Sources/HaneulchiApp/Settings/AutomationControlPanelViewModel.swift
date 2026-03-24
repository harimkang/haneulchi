import Foundation

struct AutomationControlPanelViewModel: Equatable, Sendable {
    let orchestratorSummary: String
    let workflowSummary: String
    let apiSummary: String
    let cliSummary: String
    let trackerSummary: String
    let actions: [String]

    init(snapshot: AppShellSnapshot, runtimeInfo: TerminalBackendDescriptor?) {
        orchestratorSummary =
            "\(snapshot.ops.runningSlots)/\(snapshot.ops.maxSlots) slots · \(snapshot.ops.retryQueueCount) retry · cadence \(snapshot.ops.cadenceMs)ms"
        workflowSummary =
            "\(snapshot.workflow?.state.rawValue ?? "none") · \(snapshot.workflow?.path ?? "no workflow")"
        apiSummary = runtimeInfo.map {
            "local API via \($0.transport)"
        } ?? "local API unavailable"
        cliSummary = "CLI uses the same snapshot contract"
        trackerSummary = snapshot.tracker?.health ?? "unknown"
        actions = ["Refresh", "Reconcile", "Reload Workflow", "Export Snapshot"]
    }
}
