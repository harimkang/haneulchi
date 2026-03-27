import Foundation

enum HaneulchiChromeAction: Equatable {
    case commandPalette
    case notifications
    case refresh
    case reconcile
    case reload
    case focusPane
    case find
    case paste
    case dispatch
    case splitHorizontal
    case splitVertical
    case resolve
    case dismiss
    case snooze

    var symbolName: String {
        switch self {
        case .commandPalette:
            "magnifyingglass"
        case .notifications:
            "bell"
        case .refresh:
            "arrow.clockwise"
        case .reconcile:
            "arrow.triangle.2.circlepath"
        case .reload:
            "arrow.clockwise.circle"
        case .focusPane:
            "scope"
        case .find:
            "text.magnifyingglass"
        case .paste:
            "doc.on.clipboard"
        case .dispatch:
            "paperplane"
        case .splitHorizontal:
            "rectangle.split.2x1"
        case .splitVertical:
            "rectangle.split.1x2"
        case .resolve:
            "checkmark"
        case .dismiss:
            "xmark"
        case .snooze:
            "bell.slash"
        }
    }

    var accessibilityLabel: String {
        switch self {
        case .commandPalette:
            "Open Command Palette"
        case .notifications:
            "Open Notifications"
        case .refresh:
            "Refresh Snapshot"
        case .reconcile:
            "Reconcile Automation"
        case .reload:
            "Reload Workflow"
        case .focusPane:
            "Focus Terminal Pane"
        case .find:
            "Find in Terminal Pane"
        case .paste:
            "Paste Clipboard"
        case .dispatch:
            "Open Quick Dispatch"
        case .splitHorizontal:
            "Split Horizontally"
        case .splitVertical:
            "Split Vertically"
        case .resolve:
            "Resolve Attention Item"
        case .dismiss:
            "Dismiss Attention Item"
        case .snooze:
            "Snooze Attention Item"
        }
    }
}
