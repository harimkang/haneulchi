import Foundation

enum AttentionCenterPresentation {
    enum Group: String, CaseIterable, Equatable {
        case manualTakeover = "manual_takeover"
        case failed = "failed"
        case degraded = "degraded"
        case unread = "unread"

        var displayTitle: String {
            switch self {
            case .manualTakeover:
                return "Manual Takeover"
            case .failed:
                return "Failed"
            case .degraded:
                return "Degraded"
            case .unread:
                return "Unread"
            }
        }

        var badgeState: HaneulchiStatusBadge.State {
            switch self {
            case .manualTakeover:
                return .manualTakeover
            case .failed:
                return .blocked
            case .degraded:
                return .degraded
            case .unread:
                return .waitingInput
            }
        }

        var badgeLabel: String {
            switch self {
            case .manualTakeover:
                return "MANUAL"
            case .failed:
                return "FAILED"
            case .degraded:
                return "DEGRADED"
            case .unread:
                return "UNREAD"
            }
        }
    }

    struct GroupedItems: Equatable {
        let group: Group
        let items: [AttentionCenterViewModel.Item]
    }

    static func grouped(_ items: [AttentionCenterViewModel.Item]) -> [GroupedItems] {
        var buckets: [Group: [AttentionCenterViewModel.Item]] = [:]
        for item in items {
            let group = group(for: item)
            buckets[group, default: []].append(item)
        }

        return Group.allCases.compactMap { group in
            guard let items = buckets[group], !items.isEmpty else {
                return nil
            }
            return GroupedItems(group: group, items: items)
        }
    }

    private static func group(for item: AttentionCenterViewModel.Item) -> Group {
        if item.stateLabel == "manual takeover" {
            return .manualTakeover
        }

        switch item.severity {
        case .failed:
            return .failed
        case .degraded:
            return .degraded
        case .unread:
            return .unread
        }
    }
}
