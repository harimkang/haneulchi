import Foundation

struct ControlTowerViewModel: Equatable, Sendable {
    struct HeatStrip: Equatable, Sendable {
        let running: Int
        let waitingInput: Int
        let reviewReady: Int
        let blocked: Int
    }

    struct ProjectCard: Equatable, Identifiable, Sendable {
        let projectID: String
        let title: String
        let statusLabel: String
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

            return ProjectCard(
                projectID: project.projectID,
                title: project.name,
                statusLabel: statusLabel,
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
