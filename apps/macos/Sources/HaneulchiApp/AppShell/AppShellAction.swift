import Foundation

enum AppShellAction: Equatable, Sendable {
    case selectRoute(Route)
    case toggleNotificationDrawer
    case dismissNotificationDrawer
    case refreshShellSnapshot
    case openSettings
    case presentWorkflowDrawer
    case dismissWorkflowDrawer
    case presentTaskContextDrawer
    case dismissTaskContextDrawer
    case reconcileAutomation
    case reloadWorkflow
    case resolveAttention(String)
    case dismissAttention(String)
    case snoozeAttention(String)
    case presentQuickDispatch(Route)
    case dismissQuickDispatch
    case submitQuickDispatch(targetID: String, message: String)
    case terminalSessionReady(String)
    case dispatchSend(targetSessionID: String, taskID: String?, message: String)
    case exportSnapshot
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
    case triggerRecovery(issueCode: String)
    case presentInventory
    case dismissInventory
    case openInventoryFinder(path: String)
    case openInventorySession(taskID: String, worktreeId: String)
    case openInventoryTask(taskID: String)
}
