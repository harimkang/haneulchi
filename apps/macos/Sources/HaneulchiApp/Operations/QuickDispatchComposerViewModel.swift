import Foundation

struct QuickDispatchComposerViewModel: Equatable, Sendable {
    struct Target: Equatable, Identifiable, Sendable {
        let id: String
        let title: String
        let subtitle: String
        let disabledReason: String?
        let isNewSession: Bool
    }

    let origin: Route
    let targets: [Target]
    var selectedTargetID: String?
    var messageText: String

    init(snapshot: AppShellSnapshot, origin: Route) {
        self.origin = origin
        let existingTargets = snapshot.sessions
            .filter { $0.mode == .structuredAdapter || $0.adapterKind != nil }
            .map { session in
                Target(
                    id: session.sessionID,
                    title: session.title,
                    subtitle: [session.providerID, session.modelID].compactMap { $0 }.joined(separator: " · "),
                    disabledReason: session.dispatchState == .dispatchable ? nil : session.dispatchReason,
                    isNewSession: false
                )
            }
        let newSessionTargets = Set(snapshot.sessions.compactMap(\.adapterKind))
            .sorted()
            .map { adapterKind in
                Target(
                    id: "new:\(adapterKind)",
                    title: "New \(adapterKind) session",
                    subtitle: "Create a new adapter session",
                    disabledReason: nil,
                    isNewSession: true
                )
            }

        self.targets = existingTargets + newSessionTargets
        self.selectedTargetID = existingTargets.first?.id
        self.messageText = ""
    }

    var selectedTarget: Target? {
        targets.first(where: { $0.id == selectedTargetID })
    }

    var sendEnabled: Bool {
        guard let selectedTarget, !messageText.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty else {
            return false
        }
        return selectedTarget.disabledReason == nil
    }

    var sendDisabledReason: String? {
        guard !messageText.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty else {
            return "message_required"
        }
        return selectedTarget?.disabledReason
    }

    mutating func selectTarget(id: String) {
        selectedTargetID = id
    }
}
