import Foundation

enum AppShellAction: Equatable, Sendable {
    case selectRoute(Route)
    case openSettings
    case presentWorkflowDrawer
    case dismissWorkflowDrawer
    case reloadWorkflow
    case presentNewSessionSheet
    case dismissNewSessionSheet
    case launchSession(SessionLaunchDescriptor)
    case toggleCommandPalette
    case dismissCommandPalette
    case retryReadiness
    case queueFileSelection(String)
    case createTaskDraft(String)
    case jumpToSession(String)
    case jumpToLatestUnread
}
