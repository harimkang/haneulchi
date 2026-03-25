import Foundation

enum ProjectFocusPreviewMode: Equatable, Sendable {
    case empty
    case text
    case markdown
    case json
    case yaml
    case image
}

enum InspectorSection: String, Equatable, Sendable, CaseIterable {
    case commentary
    case task
    case activity
    case evidence
    case git
    case diff
    case quickActions = "quick_actions"
}

struct ProjectFocusWorkspaceState: Equatable, Sendable {
    let projectRoot: String?
    var layoutPreset: ProjectFocusLayoutPreset
    var fileEntries: [ProjectFileIndex.Entry]
    var searchQuery: String
    var selectedFilePath: String?
    var previewMode: ProjectFocusPreviewMode
    var previewText: String?
    var isEditing: Bool
    var editingText: String
    var activeInspectorSection: InspectorSection

    init(projectRoot: String?) {
        self.projectRoot = projectRoot
        layoutPreset = .fullTerminal
        fileEntries = []
        searchQuery = ""
        selectedFilePath = nil
        previewMode = .empty
        previewText = nil
        isEditing = false
        editingText = ""
        activeInspectorSection = .commentary
    }

    var filteredEntries: [ProjectFileIndex.Entry] {
        guard !searchQuery.isEmpty else {
            return fileEntries
        }
        return fileEntries.filter {
            $0.relativePath.localizedCaseInsensitiveContains(searchQuery)
                || $0.absolutePath.localizedCaseInsensitiveContains(searchQuery)
        }
    }

    mutating func openFile(_ path: String) {
        selectedFilePath = path
        previewMode = detectPreviewMode(path: path)
        previewText = loadPreviewText(path: path, mode: previewMode)
        editingText = previewText ?? ""
        isEditing = false
    }

    mutating func enterQuickEdit() {
        guard selectedFilePath != nil else {
            return
        }
        isEditing = true
        editingText = previewText ?? ""
    }

    mutating func saveQuickEdit() throws {
        guard let selectedFilePath else {
            return
        }
        try editingText.write(toFile: selectedFilePath, atomically: true, encoding: .utf8)
        previewText = editingText
        isEditing = false
    }

    private func detectPreviewMode(path: String) -> ProjectFocusPreviewMode {
        let ext = URL(fileURLWithPath: path).pathExtension.lowercased()
        switch ext {
        case "md", "markdown":
            return .markdown
        case "json":
            return .json
        case "yaml", "yml":
            return .yaml
        case "png", "jpg", "jpeg", "gif", "webp":
            return .image
        case "":
            return .text
        default:
            return .text
        }
    }

    private func loadPreviewText(path: String, mode: ProjectFocusPreviewMode) -> String? {
        guard mode != .image else {
            return nil
        }
        return try? String(contentsOfFile: path, encoding: .utf8)
    }
}
