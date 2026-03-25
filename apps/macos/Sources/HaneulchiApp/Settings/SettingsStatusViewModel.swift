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

    struct TerminalSettingsRow: Equatable, Sendable {
        let shell: String
        let defaultCols: Int
        let defaultRows: Int
        let scrollbackLines: Int
        let fontName: String
        let theme: String
        let cursorStyle: String
    }

    struct APIRuntimeInfoRow: Equatable, Sendable {
        let socketPath: String?
        let transport: String
        let status: String
    }

    struct DegradedIssueRow: Equatable, Sendable, Identifiable {
        let issueCode: String
        let details: String
        var id: String { issueCode }
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
    let controlPanel: AutomationControlPanelViewModel?
    let terminalSettingsRow: TerminalSettingsRow?
    let apiRuntimeInfoRow: APIRuntimeInfoRow?
    let degradedIssueRows: [DegradedIssueRow]

    static let empty = Self(
        report: nil,
        workflowStatus: nil,
        presetRegistry: .init(presets: []),
        runtimeInfo: nil,
        snapshot: nil,
        terminalSettings: nil,
        runtimeInfoSummary: nil,
        degradedIssues: []
    )

    init(
        report: ReadinessReport?,
        workflowStatus: WorkflowStatusPayload?,
        presetRegistry: PresetRegistry,
        runtimeInfo: TerminalBackendDescriptor?,
        snapshot: AppShellSnapshot?,
        terminalSettings: TerminalSettingsPayload? = nil,
        runtimeInfoSummary: RuntimeInfoSummaryPayload? = nil,
        degradedIssues: [DegradedIssuePayload] = []
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
                statusLabel: snapshot == nil ? "deferred" : "available",
                detail: snapshot == nil
                    ? "CLI install path and version are not surfaced in this build yet."
                    : "CLI parity uses the same local snapshot contract as the Swift shell.",
                nextAction: snapshot == nil ? "Open Workflow Contract" : nil
            ),
            AutomationRow(
                id: .workflowWatch,
                title: "Workflow Watch",
                statusLabel: snapshot?.workflow?.state.rawValue
                    ?? workflowStatus?.state.rawValue
                    ?? "deferred",
                detail: snapshot?.workflow.map {
                    var parts = ["workflow: \($0.path)"]
                    if let lastReloadAt = $0.lastReloadAt {
                        parts.append("last reload: \(lastReloadAt)")
                    }
                    if let lastError = $0.lastError {
                        parts.append("error: \(lastError)")
                    }
                    return parts.joined(separator: " · ")
                }
                    ?? workflowStatus.map {
                        "Workflow contract loaded from \($0.path). Watch diagnostics will surface here once runtime watch state is exported."
                    }
                    ?? "Workflow watch state is not available until a repo workflow is loaded.",
                nextAction: snapshot == nil && workflowStatus == nil ? "Open Workflow Contract" : nil
            ),
            AutomationRow(
                id: .workflowDefaults,
                title: "Cadence / Slots / Retry Defaults",
                statusLabel: snapshot == nil ? "deferred" : "available",
                detail: snapshot.map {
                    "\($0.ops.cadenceMs)ms cadence · \($0.ops.runningSlots)/\($0.ops.maxSlots) slots · \($0.ops.retryQueueCount) retry · workflow \($0.workflow?.state.rawValue ?? "none") · tracker \($0.tracker?.health ?? "unknown")"
                } ?? "Workflow cadence, max slots, and retry-cap defaults are not surfaced in this build yet.",
                nextAction: snapshot == nil ? "Open Workflow Contract" : nil
            ),
        ]
        controlPanel = snapshot.map { AutomationControlPanelViewModel(snapshot: $0, runtimeInfo: runtimeInfo) }

        terminalSettingsRow = terminalSettings.map {
            TerminalSettingsRow(
                shell: $0.shell,
                defaultCols: $0.defaultCols,
                defaultRows: $0.defaultRows,
                scrollbackLines: $0.scrollbackLines,
                fontName: $0.fontName,
                theme: $0.theme,
                cursorStyle: $0.cursorStyle
            )
        }

        apiRuntimeInfoRow = runtimeInfoSummary.map {
            APIRuntimeInfoRow(
                socketPath: $0.socketPath,
                transport: $0.transport,
                status: $0.status
            )
        }

        degradedIssueRows = degradedIssues.map {
            DegradedIssueRow(issueCode: $0.issueCode, details: $0.details)
        }
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
