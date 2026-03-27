import Foundation

struct AttentionCenterViewModel {
    struct Item: Equatable, Identifiable, Sendable {
        let id: String
        let headline: String
        let stateLabel: String
        let summary: String?
        let severity: WarningFlag
        let targetRouteTitle: String
        let openAction: AppShellAction
        let attentionActionID: String?
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
        snoozeAttention: @escaping (String) -> Void,
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
                    targetRouteTitle: Route.projectFocus.title,
                    openAction: .jumpToSession(session.sessionID),
                    attentionActionID: nil,
                )
            }
            if session.dispatchState == .dispatchFailed {
                return Item(
                    id: session.sessionID,
                    headline: session.title,
                    stateLabel: "dispatch_failed",
                    summary: [session.latestSummary, session.dispatchReason]
                        .compactMap(\.self)
                        .joined(separator: " · "),
                    severity: .failed,
                    targetRouteTitle: Route.projectFocus.title,
                    openAction: .jumpToSession(session.sessionID),
                    attentionActionID: "attention-dispatch-\(session.sessionID)",
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
                targetRouteTitle: attention.targetRoute.title,
                openAction: Self.openAction(for: attention),
                attentionActionID: attention.attentionID,
            )
        }

        items = (sessionItems + attentionItems)
            .sorted { lhs, rhs in
                rank(lhs.stateLabel, lhs.severity) < rank(rhs.stateLabel, rhs.severity)
            }
        openTargetAction = openTarget
        resolveAttentionAction = resolveAttention
        dismissAttentionAction = dismissAttention
        snoozeAttentionAction = snoozeAttention
    }

    func open(_ item: Item) {
        openTargetAction(item.openAction)
    }

    func resolve(_ item: Item) {
        guard let attentionActionID = item.attentionActionID else {
            return
        }

        resolveAttentionAction(attentionActionID)
    }

    func dismiss(_ item: Item) {
        guard let attentionActionID = item.attentionActionID else {
            return
        }

        dismissAttentionAction(attentionActionID)
    }

    func snooze(_ item: Item) {
        guard let attentionActionID = item.attentionActionID else {
            return
        }

        snoozeAttentionAction(attentionActionID)
    }

    private static func openAction(for attention: AppShellSnapshot
        .AttentionSummary) -> AppShellAction
    {
        switch attention.actionHint {
        case "reload_workflow":
            return .presentWorkflowDrawer
        case "open_review":
            return .selectRoute(.reviewQueue)
        case "focus_session":
            if let targetSessionID = attention.targetSessionID {
                return .jumpToSession(targetSessionID)
            }
        default:
            break
        }

        return attention.targetSessionID.map(AppShellAction.jumpToSession)
            ?? .selectRoute(attention.targetRoute)
    }
}
