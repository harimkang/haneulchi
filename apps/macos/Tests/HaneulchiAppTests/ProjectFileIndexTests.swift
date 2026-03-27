import Foundation
@testable import HaneulchiAppUI
import Testing

private actor FileIndexVisitCounter {
    private(set) var count = 0

    func increment() {
        count += 1
    }
}

@Test("project file index returns root-scoped file paths and skips ignored directories")
func projectFileIndexSkipsIgnoredDirectories() async throws {
    let root = FileManager.default.temporaryDirectory.appendingPathComponent(
        UUID().uuidString,
        isDirectory: true,
    )
    try FileManager.default.createDirectory(at: root, withIntermediateDirectories: true)
    try "ok".write(to: root.appendingPathComponent("README.md"), atomically: true, encoding: .utf8)
    try FileManager.default.createDirectory(
        at: root.appendingPathComponent(".git"),
        withIntermediateDirectories: true,
    )
    try "nope".write(
        to: root.appendingPathComponent(".git/config"),
        atomically: true,
        encoding: .utf8,
    )

    defer {
        try? FileManager.default.removeItem(at: root)
    }

    let results = try await ProjectFileIndex().index(rootPath: root.path)

    #expect(results.contains(where: { $0.relativePath == "README.md" }))
    #expect(!results.contains(where: { $0.relativePath.contains(".git") }))
}

@Test("project file index stops scanning early when the task is cancelled")
func projectFileIndexStopsWhenCancelled() async throws {
    let root = FileManager.default.temporaryDirectory.appendingPathComponent(
        UUID().uuidString,
        isDirectory: true,
    )
    try FileManager.default.createDirectory(at: root, withIntermediateDirectories: true)

    for index in 0 ..< 200 {
        try "ok".write(
            to: root.appendingPathComponent("File\(index).txt"),
            atomically: true,
            encoding: .utf8,
        )
    }

    defer {
        try? FileManager.default.removeItem(at: root)
    }

    let counter = FileIndexVisitCounter()
    let fileIndex = ProjectFileIndex(
        onEntryVisit: {
            await counter.increment()
            try await Task.sleep(for: .milliseconds(1))
        },
    )

    let task = Task {
        try await fileIndex.index(rootPath: root.path, limit: 300)
    }

    while await counter.count == 0 {
        await Task.yield()
    }

    task.cancel()

    await #expect(throws: CancellationError.self) {
        _ = try await task.value
    }

    try await Task.sleep(for: .milliseconds(20))

    #expect(await counter.count < 200)
}
