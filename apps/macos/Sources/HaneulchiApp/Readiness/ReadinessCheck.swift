import Foundation

enum ReadinessCheckName: String, CaseIterable, Sendable {
    case shell
    case loginShellPath
    case git
    case shellIntegration
    case presetBinaries
    case keychain
    case workflow
}

enum ReadinessCheckStatus: Sendable {
    case ready
    case degraded
    case blocked
}

enum ReadinessStartupImpact: Sendable {
    case informational
    case recoveryRequired
}

struct ReadinessCheck: Equatable, Sendable {
    let name: ReadinessCheckName
    let status: ReadinessCheckStatus
    let headline: String
    let detail: String
    let nextAction: String?

    var startupImpact: ReadinessStartupImpact {
        switch (name, status) {
        case (.shell, .blocked):
            .recoveryRequired
        default:
            .informational
        }
    }
}
