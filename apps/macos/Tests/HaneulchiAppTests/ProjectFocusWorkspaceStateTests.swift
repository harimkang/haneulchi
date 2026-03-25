import Foundation
@testable import HaneulchiApp
import Testing

private func makeTempWorkspace() throws -> URL {
    let root = FileManager.default.temporaryDirectory
        .appendingPathComponent("haneulchi-workspace-\(UUID().uuidString)", isDirectory: true)
    try FileManager.default.createDirectory(at: root, withIntermediateDirectories: true)
    try "Hello".write(
        to: root.appendingPathComponent("README.md"),
        atomically: true,
        encoding: .utf8,
    )
    try "# Notes".write(
        to: root.appendingPathComponent("notes.markdown"),
        atomically: true,
        encoding: .utf8,
    )
    try #"{"ok":true}"#.write(
        to: root.appendingPathComponent("data.json"),
        atomically: true,
        encoding: .utf8,
    )
    try "name: demo".write(
        to: root.appendingPathComponent("config.yaml"),
        atomically: true,
        encoding: .utf8,
    )
    FileManager.default.createFile(
        atPath: root.appendingPathComponent("image.png").path,
        contents: Data([0x89, 0x50, 0x4E, 0x47]),
    )
    return root
}

@Test("workspace state tracks layout presets and filters project files")
func workspaceStateTracksPresetAndFiltering() throws {
    let root = try makeTempWorkspace()
    let entries = [
        ProjectFileIndex.Entry(
            relativePath: "README.md",
            absolutePath: root.appendingPathComponent("README.md").path,
        ),
        ProjectFileIndex.Entry(
            relativePath: "notes.markdown",
            absolutePath: root.appendingPathComponent("notes.markdown").path,
        ),
    ]

    var state = ProjectFocusWorkspaceState(projectRoot: root.path)
    state.layoutPreset = .explorerTerminalInspector
    state.fileEntries = entries
    state.searchQuery = "read"

    #expect(state.layoutPreset == .explorerTerminalInspector)
    #expect(state.filteredEntries.map(\.relativePath) == ["README.md"])
}

@Test("workspace state detects preview modes and supports quick edit save")
func workspaceStateSupportsPreviewAndQuickEdit() throws {
    let root = try makeTempWorkspace()
    let readme = root.appendingPathComponent("README.md").path
    let image = root.appendingPathComponent("image.png").path
    let dataJSON = root.appendingPathComponent("data.json").path

    var state = ProjectFocusWorkspaceState(projectRoot: root.path)
    state.openFile(readme)

    #expect(state.previewMode == .markdown)
    #expect(state.previewText?.contains("Hello") == true)

    state.enterQuickEdit()
    state.editingText = "Updated"
    try state.saveQuickEdit()
    #expect(try String(contentsOfFile: readme, encoding: .utf8) == "Updated")
    #expect(state.isEditing == false)

    state.openFile(image)
    #expect(state.previewMode == .image)

    state.openFile(dataJSON)
    #expect(state.previewMode == .json)
}
