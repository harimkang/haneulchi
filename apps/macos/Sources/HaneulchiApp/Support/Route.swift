enum Route: String, CaseIterable, Hashable, Identifiable {
    case projectFocus
    case controlTower
    case taskBoard
    case review
    case attention
    case settings

    var id: String { rawValue }

    var title: String {
        switch self {
        case .projectFocus:
            "Project Focus"
        case .controlTower:
            "Control Tower"
        case .taskBoard:
            "Task Board"
        case .review:
            "Review"
        case .attention:
            "Attention"
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
        case .review:
            "checkmark.circle"
        case .attention:
            "bell"
        case .settings:
            "gearshape"
        }
    }
}
