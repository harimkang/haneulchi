import Foundation

struct AttentionCenterViewModel {
    struct Item: Equatable, Identifiable, Sendable {
        let id: String
        let headline: String
        let stateLabel: String
        let summary: String?
        let severity: WarningFlag
        let targetRoute: Route
        let targetSessionID: String?
    }

    let items: [Item]
    private let openTargetAction: (AppShellAction) -> Void
    private let resolveAttentionAction: (String) -> Void
    private let dismissAttentionAction: (String) -> Void
    private let snoozeAttentionAction: (String) -> Void

    init(
        snapshot: AppShellSnapshot,
        openTarget: @escaping (AppShellAction) -> Void,
        resolveAttention: @escaping (String) -> Void,
        dismissAttention: @escaping (String) -> Void,
        snoozeAttention: @escaping (String) -> Void
    ) {
        let rank: (String, WarningFlag) -> Int = { stateLabel, severity in
            if stateLabel == "manual takeover" {
                return 0
            }
            if stateLabel == "dispatch_failed" {
                return 1
            }
            switch severity {
            case .failed:
                return 2
            case .degraded:
                return 3
            case .unread:
                return 4
            }
        }

        let sessionItems = snapshot.sessions.compactMap { session -> Item? in
            if session.manualControlState == .takeover {
                return Item(
                    id: session.sessionID,
                    headline: session.title,
                    stateLabel: "manual takeover",
                    summary: session.latestSummary ?? "Operator takeover active.",
                    severity: .failed,
                    targetRoute: .projectFocus,
                    targetSessionID: session.sessionID
                )
            }
            if session.dispatchState == .dispatchFailed {
                return Item(
                    id: session.sessionID,
                    headline: session.title,
                    stateLabel: "dispatch_failed",
                    summary: [session.latestSummary, session.dispatchReason]
                        .compactMap { $0 }
                        .joined(separator: " · "),
                    severity: .failed,
                    targetRoute: .projectFocus,
                    targetSessionID: session.sessionID
                )
            }
            return nil
        }

        let attentionItems = snapshot.attention.map { attention in
            Item(
                id: attention.attentionID,
                headline: attention.headline,
                stateLabel: attention.severity.rawValue,
                summary: attention.summary,
                severity: attention.severity,
                targetRoute: attention.targetRoute,
                targetSessionID: attention.targetSessionID
            )
        }

        self.items = (sessionItems + attentionItems)
            .sorted { lhs, rhs in
                rank(lhs.stateLabel, lhs.severity) < rank(rhs.stateLabel, rhs.severity)
            }
        self.openTargetAction = openTarget
        self.resolveAttentionAction = resolveAttention
        self.dismissAttentionAction = dismissAttention
        self.snoozeAttentionAction = snoozeAttention
    }

    func open(_ item: Item) {
        openTargetAction(
            item.targetSessionID.map(AppShellAction.jumpToSession)
                ?? .selectRoute(item.targetRoute)
        )
    }

    func resolve(_ item: Item) {
        resolveAttentionAction(item.id)
    }

    func dismiss(_ item: Item) {
        dismissAttentionAction(item.id)
    }

    func snooze(_ item: Item) {
        snoozeAttentionAction(item.id)
    }
}
