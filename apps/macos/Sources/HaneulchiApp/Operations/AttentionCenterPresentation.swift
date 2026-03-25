import Foundation

enum AttentionCenterPresentation {
    enum Group: String, CaseIterable, Equatable {
        case manualTakeover = "manual_takeover"
        case failed
        case degraded
        case unread

        var displayTitle: String {
            switch self {
            case .manualTakeover:
                "Manual Takeover"
            case .failed:
                "Failed"
            case .degraded:
                "Degraded"
            case .unread:
                "Unread"
            }
        }

        var badgeState: HaneulchiStatusBadge.State {
            switch self {
            case .manualTakeover:
                .manualTakeover
            case .failed:
                .blocked
            case .degraded:
                .degraded
            case .unread:
                .waitingInput
            }
        }

        var badgeLabel: String {
            switch self {
            case .manualTakeover:
                "MANUAL"
            case .failed:
                "FAILED"
            case .degraded:
                "DEGRADED"
            case .unread:
                "UNREAD"
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
