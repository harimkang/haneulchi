import Foundation
import Testing
@testable import HaneulchiApp

@Test("task search store returns real task-summary rows rather than a placeholder section")
func taskSearchStoreReturnsPersistedRows() throws {
    let store = TaskSearchProjectionStore.inMemory

    try store.upsert([
        .init(
            taskID: "task_01",
            projectID: "proj_demo",
            title: "Wire app shell",
            state: .ready,
            automationMode: .manual,
            linkedSessionID: nil
        )
    ])

    let rows = try store.search("wire")

    #expect(rows.map(\.taskID) == ["task_01"])
}

@Test("file-backed task search store falls back instead of crashing when the sqlite path is invalid")
func fileBackedTaskStoreFallsBackWhenPathIsInvalid() throws {
    let invalidRoot = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString, isDirectory: true)
    try FileManager.default.createDirectory(at: invalidRoot, withIntermediateDirectories: true)
    defer {
        try? FileManager.default.removeItem(at: invalidRoot)
    }

    let store = TaskSearchProjectionStore.fileBacked(fileURL: invalidRoot)

    let rowsToInsert: [TaskSearchProjectionStore.Row] = [
        .init(
            taskID: "task_01",
            projectID: "proj_demo",
            title: "Fallback task",
            state: .inbox,
            automationMode: .manual,
            linkedSessionID: nil
        )
    ]
    try store.upsert(rowsToInsert)

    let rows = try store.search("fallback")
    #expect(rows.count == 1)
}
