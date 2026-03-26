import Foundation

struct ControlTowerViewModel: Equatable, Sendable {
    struct HeatStrip: Equatable, Sendable {
        let running: Int
        let waitingInput: Int
        let reviewReady: Int
        let blocked: Int
    }

    struct ProjectCard: Equatable, Identifiable, Sendable {
        struct OverviewMetric: Equatable, Sendable {
            let label: String
            let value: String
        }

        let projectID: String
        let title: String
        let iconName: String
        let statusLabel: String
        let sessionCount: Int
        let attentionCount: Int
        let sessionCountLabel: String
        let attentionCountLabel: String
        let latestSummary: String?
        let latestCommentary: String?
        let heatStrip: HeatStrip

        var id: String {
            projectID
        }

        var statusBadgeState: HaneulchiStatusBadge.State {
            switch statusLabel.lowercased() {
            case "attention", "error", "blocked":
                .blocked
            case "running", "active":
                .active
            case "review", "review_ready":
                .reviewReady
            case "waiting", "waiting_input":
                .waitingInput
            case "degraded":
                .degraded
            case "done", "complete":
                .done
            default:
                .idle
            }
        }

        var primaryMeta: [String] {
            [sessionCountLabel, attentionCountLabel] + spotlightMeta
        }

        var spotlightLabel: String? {
            spotlightMeta.first
        }

        var overviewMetrics: [OverviewMetric] {
            [
                .init(label: "Sessions", value: "\(sessionCount)"),
                .init(label: "Alerts", value: "\(attentionCount)"),
            ]
        }

        var accent: HaneulchiSignalAccent {
            if heatStrip.blocked > 0 {
                return .error
            }
            if heatStrip.waitingInput > 0 {
                return .warning
            }
            if heatStrip.reviewReady > 0 {
                return .reviewReady
            }
            switch statusLabel.lowercased() {
            case "attention", "waiting", "waiting_input", "degraded":
                return .warning
            case "error", "blocked":
                return .error
            case "running", "active":
                return .success
            case "review", "review_ready":
                return .reviewReady
            default:
                return .neutral
            }
        }

        private var spotlightMeta: [String] {
            if heatStrip.reviewReady > 0 {
                return ["\(heatStrip.reviewReady) review ready"]
            }
            if heatStrip.waitingInput > 0 {
                return ["\(heatStrip.waitingInput) waiting input"]
            }
            if heatStrip.blocked > 0 {
                return ["\(heatStrip.blocked) blocked"]
            }
            if heatStrip.running > 0 {
                return ["\(heatStrip.running) running"]
            }
            return []
        }
    }

    struct AttentionItem: Equatable, Identifiable, Sendable {
        let id: String
        let headline: String
        let summary: String?
        let targetRoute: Route
        let targetSessionID: String?
    }

    struct RecentArtifactItem: Equatable, Identifiable, Sendable {
        let taskID: String
        let projectID: String
        let summary: String
        let targetRoute: Route
        let manifestPath: String?

        var id: String {
            "\(projectID):\(taskID)"
        }
    }

    let opsModel: AutomationPanelViewModel
    let projectCards: [ProjectCard]
    let attentionItems: [AttentionItem]
    let recentArtifacts: [RecentArtifactItem]

    init(snapshot: AppShellSnapshot) {
        opsModel = AutomationPanelViewModel(snapshot: snapshot)

        projectCards = snapshot.projects.map { project in
            let sessions = snapshot.sessions.filter { $0.projectID == project.projectID }
            let latestSession = sessions.max { lhs, rhs in
                (lhs.lastActivityAt ?? "") < (rhs.lastActivityAt ?? "")
            }
            let heatStrip = HeatStrip(
                running: sessions.count(where: { $0.runtimeState == .running }),
                waitingInput: sessions.count(where: { $0.runtimeState == .waitingInput }),
                reviewReady: sessions.count(where: { $0.runtimeState == .reviewReady }),
                blocked: sessions
                    .count(where: { $0.runtimeState == .blocked || $0.runtimeState == .error }),
            )
            let statusLabel = project.attentionCount > 0 ? "attention" : project.status.rawValue

            let iconName: String
            let lowerName = project.name.lowercased()
            if lowerName.contains("auth") { iconName = "lock.shield" }
            else if lowerName.contains("api") { iconName = "network" }
            else if lowerName.contains("ios") || lowerName
                .contains("mobile") { iconName = "iphone" }
            else if lowerName.contains("infra") || lowerName
                .contains("k8s") { iconName = "server.rack" }
            else { iconName = "folder" }

            return ProjectCard(
                projectID: project.projectID,
                title: project.name,
                iconName: iconName,
                statusLabel: statusLabel,
                sessionCount: project.sessionCount,
                attentionCount: project.attentionCount,
                sessionCountLabel: "\(project.sessionCount) sessions",
                attentionCountLabel: "\(project.attentionCount) items",
                latestSummary: latestSession?.latestSummary,
                latestCommentary: latestSession?.latestCommentary,
                heatStrip: heatStrip,
            )
        }

        attentionItems = snapshot.attention.map { item in
            AttentionItem(
                id: item.id,
                headline: item.headline,
                summary: item.summary,
                targetRoute: item.targetRoute,
                targetSessionID: item.targetSessionID,
            )
        }

        recentArtifacts = snapshot.recentArtifacts.map { item in
            RecentArtifactItem(
                taskID: item.taskID,
                projectID: item.projectID,
                summary: item.summary,
                targetRoute: item.jumpTarget == "review_queue" ? .reviewQueue : .projectFocus,
                manifestPath: item.manifestPath,
            )
        }
    }
}
