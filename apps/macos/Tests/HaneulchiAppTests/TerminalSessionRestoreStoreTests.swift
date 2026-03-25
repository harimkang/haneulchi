import Foundation
@testable import HaneulchiApp
import Testing

@Test("restore store round-trips a persisted terminal restore bundle")
func restoreStoreRoundTripsBundle() throws {
    let store = TerminalSessionRestoreStore.inMemory
    let bundle = TerminalRestoreBundle.demo

    try store.save([bundle])

    #expect(try store.load() == [bundle])
}

@Test("file-backed restore store round-trips bundles on disk")
func fileBackedRestoreStoreRoundTripsBundles() throws {
    let fileURL = FileManager.default.temporaryDirectory
        .appendingPathComponent(UUID().uuidString)
        .appendingPathExtension("json")
    let store = TerminalSessionRestoreStore.fileBacked(fileURL: fileURL)
    let bundle = TerminalRestoreBundle.demo

    try store.save([bundle])

    #expect(try store.load() == [bundle])
}

@Test("restore store round-trips multiple bundles through save and load")
func restoreStoreRoundTrips() throws {
    let store = TerminalSessionRestoreStore.inMemory

    // Initially empty
    #expect(try store.load().isEmpty)

    let bundles = [
        TerminalRestoreBundle.genericShell(at: "/tmp/project-a"),
        TerminalRestoreBundle.genericShell(at: "/tmp/project-b"),
    ]
    try store.save(bundles)

    let loaded = try store.load()
    #expect(loaded.count == 2)
    #expect(loaded[0].launch.currentDirectory == "/tmp/project-a")
    #expect(loaded[1].launch.currentDirectory == "/tmp/project-b")

    // Overwrite with a single bundle
    try store.save([TerminalRestoreBundle.demo])
    #expect(try store.load().count == 1)
}
