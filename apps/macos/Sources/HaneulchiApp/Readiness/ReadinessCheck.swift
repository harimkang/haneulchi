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

struct ReadinessCheck: Equatable, Sendable {
    let name: ReadinessCheckName
    let status: ReadinessCheckStatus
    let headline: String
    let detail: String
    let nextAction: String?
}
