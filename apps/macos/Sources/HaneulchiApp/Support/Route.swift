import Foundation

struct RouteShortcut: Equatable, Sendable {
    struct Modifiers: OptionSet, Equatable, Sendable {
        let rawValue: Int

        static let command = Self(rawValue: 1 << 0)
        static let shift = Self(rawValue: 1 << 1)
    }

    let key: String
    let modifiers: Modifiers

    init(_ key: String, modifiers: Modifiers = []) {
        self.key = key.lowercased()
        self.modifiers = modifiers
    }
}

enum Route: String, CaseIterable, Hashable, Identifiable, Codable, Sendable {
    case projectFocus = "project_focus"
    case controlTower = "control_tower"
    case taskBoard = "task_board"
    case reviewQueue = "review_queue"
    case attentionCenter = "attention_center"
    case settings = "settings"

    static let primaryCases: [Route] = [
        .projectFocus,
        .controlTower,
        .taskBoard,
        .reviewQueue,
        .attentionCenter,
    ]

    static let latestUnreadShortcut = RouteShortcut(
        "u",
        modifiers: [.command, .shift]
    )

    var id: String { rawValue }

    var title: String {
        switch self {
        case .projectFocus:
            "Project Focus"
        case .controlTower:
            "Control Tower"
        case .taskBoard:
            "Task Board"
        case .reviewQueue:
            "Review Queue"
        case .attentionCenter:
            "Attention Center"
        case .settings:
            "Settings"
        }
    }

    var symbolName: String {
        switch self {
        case .projectFocus:
            "terminal"
        case .controlTower:
            "square.grid.2x2"
        case .taskBoard:
            "checklist"
        case .reviewQueue:
            "checkmark.circle"
        case .attentionCenter:
            "bell"
        case .settings:
            "gearshape"
        }
    }

    var shortcutLabel: String? {
        switch self {
        case .projectFocus:
            "Cmd+1"
        case .controlTower:
            "Cmd+2"
        case .taskBoard:
            "Cmd+3"
        case .reviewQueue:
            "Cmd+4"
        case .attentionCenter:
            "Cmd+5"
        case .settings:
            nil
        }
    }

    var keyboardShortcut: RouteShortcut {
        switch self {
        case .projectFocus:
            .init("1", modifiers: [.command])
        case .controlTower:
            .init("2", modifiers: [.command])
        case .taskBoard:
            .init("3", modifiers: [.command])
        case .reviewQueue:
            .init("4", modifiers: [.command])
        case .attentionCenter:
            .init("5", modifiers: [.command])
        case .settings:
            .init(",", modifiers: [.command])
        }
    }
}
