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
                summary: "Task Board now reads the Rust-owned board projection, supports project filtering, and lets operators move tasks between the fixed six columns.",
                nextActionTitle: "Open Task Board"
            )
        case .reviewQueue:
            return .init(
                route: route,
                title: route.title,
                summary: "Review Queue now reads Rust-owned review-ready evidence summaries so touched files, diff signals, test results, and warnings stay aligned with the task projection.",
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
    let settingsStatusViewModel: SettingsStatusViewModel
    let queuedProjectFocusFilePath: String?
    let onAction: (AppShellAction) -> Void

    var body: some View {
        switch route {
        case .projectFocus:
            ProjectFocusView(
                model: projectFocusModel,
                snapshot: snapshot,
                queuedFilePath: queuedProjectFocusFilePath,
                onAction: onAction
            )
        case .settings:
            SettingsView(viewModel: settingsStatusViewModel)
        case .controlTower:
            ControlTowerPlaceholderView(
                descriptor: .placeholder(for: .controlTower, snapshot: snapshot),
                snapshot: snapshot
            )
        case .taskBoard:
            TaskBoardPlaceholderView(descriptor: .placeholder(for: .taskBoard, snapshot: snapshot))
        case .reviewQueue:
            ReviewQueuePlaceholderView(descriptor: .placeholder(for: .reviewQueue, snapshot: snapshot))
        case .attentionCenter:
            AttentionCenterPlaceholderView(descriptor: .placeholder(for: .attentionCenter, snapshot: snapshot))
        }
    }
}
