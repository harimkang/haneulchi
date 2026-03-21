import Testing
@testable import HaneulchiApp

@Test("project focus demo boots with one hosted surface and split disabled")
func projectFocusDemoSurfaceContract() {
    let model = ProjectFocusView.Model.demo

    #expect(model.deck.layout.paneIDs.count == 1)
    #expect(model.deck.layout.rootSplitID == nil)
    #expect(model.deck.showsSplitControls == false)
    #expect(model.deck.layout.focusedSurface?.fixtureName == "hello-world.ansi")
}

@Test("runtime project focus model switches the focused surface to live session mode")
func runtimeProjectFocusUsesLiveSurface() {
    let model = ProjectFocusView.Model.runtimeDemo

    #expect(model.deck.layout.focusedSurface?.isLive == true)
    #expect(model.deck.layout.focusedSurface?.fixtureName == nil)
}

@Test("bootstrap project focus model only uses a live restore when one exists")
func bootstrapProjectFocusModelUsesRestoreBundleIfPresent() throws {
    let emptyStore = TerminalSessionRestoreStore.inMemory
    #expect(try ProjectFocusView.Model.bootstrap(restoreStore: emptyStore) == .demo)

    let restoredStore = TerminalSessionRestoreStore.inMemory
    try restoredStore.save([.demo])
    let restored = try ProjectFocusView.Model.bootstrap(restoreStore: restoredStore)

    #expect(restored.deck.layout.focusedSurface?.isLive == true)
}
