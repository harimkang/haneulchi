import Foundation
import Testing
@testable import HaneulchiApp

@Test("inventory search store returns real inventory rows from restore metadata and filesystem roots")
func inventorySearchStoreReturnsRealRows() async throws {
    let root = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString, isDirectory: true)
    try FileManager.default.createDirectory(at: root, withIntermediateDirectories: true)

    defer {
        try? FileManager.default.removeItem(at: root)
    }

    let restoreStore = TerminalSessionRestoreStore.inMemory
    try restoreStore.save([.genericShell(at: root.path)])

    let rows = try await InventorySearchProjectionStore(restoreStore: restoreStore).load(
        selectedProjectRoot: root.path
    )

    #expect(rows.contains(where: { $0.rootPath == root.path }))
}
