import Foundation

struct SettingsStatusViewModel: Equatable, Sendable {
    struct ReadinessRow: Equatable, Sendable, Identifiable {
        let headline: String
        let detail: String
        let statusLabel: String
        let nextAction: String?

        var id: String { headline }
    }

    struct PresetRow: Equatable, Sendable, Identifiable {
        let id: String
        let title: String
        let statusLabel: String
        let detail: String
        let requiresShellIntegration: Bool
    }

    struct WorkflowRow: Equatable, Sendable {
        let title: String
        let path: String
        let statusLabel: String
        let detail: String
        let lastError: String?
    }

    struct AutomationRow: Equatable, Sendable, Identifiable {
        enum RowID: String, Sendable {
            case localAPI = "local_api"
            case cli = "cli"
            case workflowWatch = "workflow_watch"
            case workflowDefaults = "workflow_defaults"
        }

        let id: RowID
        let title: String
        let statusLabel: String
        let detail: String
        let nextAction: String?
    }

    let readinessRows: [ReadinessRow]
    let shellIntegrationRow: ReadinessRow?
    let workflowRow: WorkflowRow?
    let presetRows: [PresetRow]
    let automationRows: [AutomationRow]

    static let empty = Self(
        report: nil,
        workflowStatus: nil,
        presetRegistry: .init(presets: []),
        runtimeInfo: nil
    )

    init(
        report: ReadinessReport?,
        workflowStatus: WorkflowStatusPayload?,
        presetRegistry: PresetRegistry,
        runtimeInfo: TerminalBackendDescriptor?
    ) {
        let checks = report?.checks ?? []
        readinessRows = checks
            .filter { ![.shellIntegration, .presetBinaries, .workflow].contains($0.name) }
            .map(Self.readinessRow)

        shellIntegrationRow = checks
            .first(where: { $0.name == .shellIntegration })
            .map(Self.readinessRow)

        if let workflowStatus {
            let title = workflowStatus.workflow?.name ?? "Workflow Contract"
            var detailParts = [workflowStatus.path]
            if let strategy = workflowStatus.workflow?.strategy, let baseRoot = workflowStatus.workflow?.baseRoot {
                detailParts.append("strategy: \(strategy) · base root: \(baseRoot)")
            }
            if let lastGoodHash = workflowStatus.lastGoodHash {
                detailParts.append("last good: \(lastGoodHash)")
            }
            workflowRow = WorkflowRow(
                title: title,
                path: workflowStatus.path,
                statusLabel: workflowStatus.state.rawValue,
                detail: detailParts.joined(separator: "\n"),
                lastError: workflowStatus.lastError
            )
        } else {
            workflowRow = nil
        }

        presetRows = presetRegistry.presets
            .sorted { $0.id < $1.id }
            .map { preset in
                let capabilitySummary = preset.capabilityFlags.isEmpty
                    ? "no capability flags"
                    : preset.capabilityFlags.joined(separator: ", ")

                return PresetRow(
                    id: preset.id,
                    title: preset.title,
                    statusLabel: preset.installState.rawValue,
                    detail: capabilitySummary,
                    requiresShellIntegration: preset.requiresShellIntegration
                )
            }

        automationRows = [
            AutomationRow(
                id: .localAPI,
                title: "Local API Boundary",
                statusLabel: runtimeInfo == nil ? "deferred" : "available",
                detail: runtimeInfo.map {
                    "Rust core connected via \($0.transport). Local-only diagnostics remain the trust boundary."
                } ?? "Local control API diagnostics are not surfaced in this build yet.",
                nextAction: runtimeInfo == nil ? "Open Workflow Contract" : nil
            ),
            AutomationRow(
                id: .cli,
                title: "hc CLI",
                statusLabel: "deferred",
                detail: "CLI install path and version are not surfaced in this build yet.",
                nextAction: "Open Workflow Contract"
            ),
            AutomationRow(
                id: .workflowWatch,
                title: "Workflow Watch",
                statusLabel: "deferred",
                detail: workflowStatus.map {
                    "Workflow contract loaded from \($0.path). Watch diagnostics will surface here once runtime watch state is exported."
                } ?? "Workflow watch state is not available until a repo workflow is loaded.",
                nextAction: workflowStatus == nil ? "Open Workflow Contract" : nil
            ),
            AutomationRow(
                id: .workflowDefaults,
                title: "Cadence / Slots / Retry Defaults",
                statusLabel: "deferred",
                detail: "Workflow cadence, max slots, and retry-cap defaults are not surfaced in this build yet.",
                nextAction: "Open Workflow Contract"
            ),
        ]
    }

    private static func readinessRow(_ check: ReadinessCheck) -> ReadinessRow {
        ReadinessRow(
            headline: check.headline,
            detail: check.detail,
            statusLabel: check.status.label,
            nextAction: check.nextAction
        )
    }
}

private extension ReadinessCheckStatus {
    var label: String {
        switch self {
        case .ready:
            "ready"
        case .degraded:
            "degraded"
        case .blocked:
            "blocked"
        }
    }
}
