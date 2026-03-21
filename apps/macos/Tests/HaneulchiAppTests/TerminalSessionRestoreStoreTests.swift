import Foundation
import Testing
@testable import HaneulchiApp

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
