import Foundation

struct PresetBinaryCatalog: Sendable {
    let commands: [String]

    static let launcherDefault = Self(commands: [
        "claude",
        "codex",
        "gemini",
        "yazi",
        "lazygit",
    ])
}
