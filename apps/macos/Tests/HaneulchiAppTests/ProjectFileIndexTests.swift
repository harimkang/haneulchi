import Foundation
import Testing
@testable import HaneulchiApp

@Test("project file index returns root-scoped file paths and skips ignored directories")
func projectFileIndexSkipsIgnoredDirectories() async throws {
    let root = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString, isDirectory: true)
    try FileManager.default.createDirectory(at: root, withIntermediateDirectories: true)
    try "ok".write(to: root.appendingPathComponent("README.md"), atomically: true, encoding: .utf8)
    try FileManager.default.createDirectory(at: root.appendingPathComponent(".git"), withIntermediateDirectories: true)
    try "nope".write(to: root.appendingPathComponent(".git/config"), atomically: true, encoding: .utf8)

    defer {
        try? FileManager.default.removeItem(at: root)
    }

    let results = try await ProjectFileIndex().index(rootPath: root.path)

    #expect(results.contains(where: { $0.relativePath == "README.md" }))
    #expect(!results.contains(where: { $0.relativePath.contains(".git") }))
}
