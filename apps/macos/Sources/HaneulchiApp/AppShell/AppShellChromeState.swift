import Foundation

struct AppShellChromeState: Equatable {
    struct Chip: Equatable, Identifiable {
        let title: String
        let tone: WarningFlag?

        var id: String { title }
    }

    struct RailItem: Equatable, Identifiable {
        let route: Route
        let title: String
        let badgeText: String?
        let shortcutLabel: String?

        var id: Route { route }
    }

    struct StripItem: Equatable, Identifiable {
        let title: String
        let detail: String?

        var id: String { title }
    }

    let topBarTitle: String
    let topBarChips: [Chip]
    let leftRailItems: [RailItem]
    let bottomStripItems: [StripItem]
    let transientNotice: String?

    init(
        snapshot: AppShellSnapshot,
        selectedProjectName: String?,
        transientNotice: String? = nil
    ) {
        self.topBarTitle = selectedProjectName ?? snapshot.projects.first?.name ?? "Haneulchi"
        self.topBarChips = Self.makeTopBarChips(from: snapshot)
        self.leftRailItems = Self.makeLeftRailItems(from: snapshot)
        self.bottomStripItems = Self.makeBottomStripItems(from: snapshot)
        self.transientNotice = transientNotice
    }

    private static func makeTopBarChips(from snapshot: AppShellSnapshot) -> [Chip] {
        var chips: [Chip] = []

        for flag in snapshot.app.degradedFlags {
            chips.append(.init(title: flag.rawValue, tone: flag))
        }

        if snapshot.ops.workflowHealth != .none {
            chips.append(.init(title: snapshot.ops.workflowHealth.rawValue, tone: nil))
        }

        if chips.isEmpty {
            chips.append(.init(title: snapshot.app.activeRoute.title, tone: nil))
        }

        return chips
    }

    private static func makeLeftRailItems(from snapshot: AppShellSnapshot) -> [RailItem] {
        Route.primaryCases.map { route in
            let badgeText: String?

            switch route {
            case .projectFocus:
                badgeText = snapshot.sessions.isEmpty ? nil : "\(snapshot.sessions.count)"
            case .attentionCenter:
                badgeText = snapshot.attention.isEmpty ? nil : "\(snapshot.attention.count)"
            default:
                badgeText = nil
            }

            return .init(
                route: route,
                title: route.title,
                badgeText: badgeText,
                shortcutLabel: route.shortcutLabel
            )
        }
    }

    private static func makeBottomStripItems(from snapshot: AppShellSnapshot) -> [StripItem] {
        var items: [StripItem] = []

        items.append(.init(
            title: "\(snapshot.sessions.count) sessions",
            detail: snapshot.projects.first?.name
        ))
        items.append(.init(
            title: "route \(snapshot.app.activeRoute.rawValue)",
            detail: nil
        ))
        items.append(.init(
            title: "snapshot \(snapshot.meta.snapshotRev)",
            detail: "runtime \(snapshot.meta.runtimeRev)"
        ))

        return items
    }
}
