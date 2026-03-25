import Foundation

enum HaneulchiChromeAction: Equatable {
    case commandPalette
    case notifications
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
            return "magnifyingglass"
        case .notifications:
            return "bell"
        case .focusPane:
            return "scope"
        case .find:
            return "text.magnifyingglass"
        case .paste:
            return "doc.on.clipboard"
        case .dispatch:
            return "paperplane"
        case .splitHorizontal:
            return "rectangle.split.2x1"
        case .splitVertical:
            return "rectangle.split.1x2"
        case .resolve:
            return "checkmark"
        case .dismiss:
            return "xmark"
        case .snooze:
            return "bell.slash"
        }
    }

    var accessibilityLabel: String {
        switch self {
        case .commandPalette:
            return "Open Command Palette"
        case .notifications:
            return "Open Notifications"
        case .focusPane:
            return "Focus Terminal Pane"
        case .find:
            return "Find in Terminal Pane"
        case .paste:
            return "Paste Clipboard"
        case .dispatch:
            return "Open Quick Dispatch"
        case .splitHorizontal:
            return "Split Horizontally"
        case .splitVertical:
            return "Split Vertically"
        case .resolve:
            return "Resolve Attention Item"
        case .dismiss:
            return "Dismiss Attention Item"
        case .snooze:
            return "Snooze Attention Item"
        }
    }
}
