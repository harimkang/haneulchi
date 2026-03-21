import Foundation

enum CommandPaletteSectionKind: String, Equatable, Sendable {
    case commands
    case files
    case sessions
    case tasks
    case inventory
}

struct CommandPaletteItem: Equatable, Identifiable, Sendable {
    let id: String
    let section: CommandPaletteSectionKind
    let title: String
    let subtitle: String?
    let tokens: [String]
    let action: AppShellAction?
}

struct CommandPaletteSection: Equatable, Identifiable, Sendable {
    let kind: CommandPaletteSectionKind
    let items: [CommandPaletteItem]

    var id: String { kind.rawValue }
}
