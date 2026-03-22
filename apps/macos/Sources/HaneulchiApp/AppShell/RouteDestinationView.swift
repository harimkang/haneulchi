import SwiftUI

struct RouteDestinationDescriptor: Equatable {
    let route: Route
    let title: String
    let summary: String
    let nextActionTitle: String

    static func placeholder(for route: Route, snapshot: AppShellSnapshot) -> Self {
        switch route {
        case .controlTower:
            return .init(
                route: route,
                title: route.title,
                summary: "\(snapshot.projects.count) projects, \(snapshot.sessions.count) sessions, \(snapshot.attention.count) attention items.",
                nextActionTitle: "Open Control Tower"
            )
        case .taskBoard:
            return .init(
                route: route,
                title: route.title,
                summary: "Task board route is wired. Full board data and editing land with MVP2-026 and MVP2-027.",
                nextActionTitle: "Open Task Board"
            )
        case .reviewQueue:
            return .init(
                route: route,
                title: route.title,
                summary: "Review queue route is wired. Evidence list and review actions land with MVP2-030 and MVP2-031.",
                nextActionTitle: "Open Review Queue"
            )
        case .attentionCenter:
            return .init(
                route: route,
                title: route.title,
                summary: "\(snapshot.attention.count) attention items, \(snapshot.warnings.count) warnings.",
                nextActionTitle: "Open Attention Center"
            )
        case .projectFocus, .settings:
            return .init(
                route: route,
                title: route.title,
                summary: route.title,
                nextActionTitle: route.title
            )
        }
    }
}

struct RouteDestinationView: View {
    let route: Route
    let snapshot: AppShellSnapshot
    let projectFocusModel: ProjectFocusView.Model
    let readinessReport: ReadinessReport?
    let onAction: (AppShellAction) -> Void

    var body: some View {
        switch route {
        case .projectFocus:
            ProjectFocusView(
                model: projectFocusModel,
                snapshot: snapshot,
                onAction: onAction
            )
        case .settings:
            SettingsView(report: readinessReport)
        case .controlTower:
            ControlTowerPlaceholderView(descriptor: .placeholder(for: .controlTower, snapshot: snapshot))
        case .taskBoard:
            TaskBoardPlaceholderView(descriptor: .placeholder(for: .taskBoard, snapshot: snapshot))
        case .reviewQueue:
            ReviewQueuePlaceholderView(descriptor: .placeholder(for: .reviewQueue, snapshot: snapshot))
        case .attentionCenter:
            AttentionCenterPlaceholderView(descriptor: .placeholder(for: .attentionCenter, snapshot: snapshot))
        }
    }
}
