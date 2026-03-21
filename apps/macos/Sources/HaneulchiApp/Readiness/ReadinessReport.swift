import Foundation

struct ReadinessReport: Equatable, Sendable {
    let project: LauncherProject?
    let checks: [ReadinessCheck]

    var canContinueWithGenericShell: Bool {
        project != nil && checks.first(where: { $0.name == .shell })?.status != .blocked
    }

    var requiresRecoverySurface: Bool {
        checks.contains { $0.status != .ready }
    }

    func check(named name: ReadinessCheckName) -> ReadinessCheck? {
        checks.first(where: { $0.name == name })
    }
}
