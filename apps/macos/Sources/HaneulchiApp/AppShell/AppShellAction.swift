import Foundation

enum AppShellAction: Equatable, Sendable {
    case selectRoute(Route)
    case openSettings
    case toggleCommandPalette
    case dismissCommandPalette
    case retryReadiness
    case queueFileSelection(String)
    case createTaskDraft(String)
    case jumpToSession(String)
    case jumpToLatestUnread
}
