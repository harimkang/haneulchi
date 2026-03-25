import Foundation

struct NotificationDrawerModel: Equatable, Sendable {
    struct Item: Equatable, Identifiable, Sendable {
        let id: String
        let title: String
        let stateLabel: String
        let summary: String
        let severity: WarningFlag
        let action: AppShellAction
    }

    let items: [Item]

    init(snapshot: AppShellSnapshot) {
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
                    title: session.title,
                    stateLabel: "manual takeover",
                    summary: session.latestSummary ?? "Operator takeover active.",
                    severity: .failed,
                    action: .jumpToSession(session.sessionID),
                )
            }
            if session.dispatchState == .dispatchFailed {
                let summary = [session.latestSummary, session.dispatchReason]
                    .compactMap(\.self)
                    .joined(separator: " · ")
                return Item(
                    id: session.sessionID,
                    title: session.title,
                    stateLabel: "dispatch_failed",
                    summary: summary.isEmpty ? "Dispatch failed." : summary,
                    severity: .failed,
                    action: .jumpToSession(session.sessionID),
                )
            }
            return nil
        }

        let attentionItems = snapshot.attention.map { attention in
            Item(
                id: attention.attentionID,
                title: attention.headline,
                stateLabel: attention.severity.rawValue,
                summary: attention.summary ?? attention.headline,
                severity: attention.severity,
                action: attention.targetSessionID.map(AppShellAction.jumpToSession)
                    ?? .selectRoute(attention.targetRoute),
            )
        }

        items = (sessionItems + attentionItems)
            .sorted { lhs, rhs in
                rank(lhs.stateLabel, lhs.severity) < rank(rhs.stateLabel, rhs.severity)
            }
    }
}
