import AppKit
import Foundation

struct ProjectFolderPicker {
    let pickFolder: @MainActor () -> URL?

    static let live = Self(
        pickFolder: {
            let panel = NSOpenPanel()
            panel.canChooseFiles = false
            panel.canChooseDirectories = true
            panel.allowsMultipleSelection = false
            panel.prompt = "Open Project"
            return panel.runModal() == .OK ? panel.url : nil
        },
    )
}
