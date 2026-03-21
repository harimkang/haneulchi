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

@Test("selected project without a restore bundle boots a live shell rooted at the project path")
func projectFocusBootstrapFallsBackToSelectedProjectRoot() throws {
    let model = try ProjectFocusView.Model.bootstrap(
        selectedProjectRoot: "/tmp/auth-service",
        restoreStore: .inMemory
    )

    #expect(model.deck.layout.focusedSurface?.liveBundle?.launch.currentDirectory == "/tmp/auth-service")
}

@Test("selected project root overrides a stale restore bundle from another repo")
func selectedProjectRootOverridesRestoreBundle() throws {
    let store = TerminalSessionRestoreStore.inMemory
    try store.save([.genericShell(at: "/tmp/stale-repo")])

    let model = try ProjectFocusView.Model.bootstrap(
        selectedProjectRoot: "/tmp/auth-service",
        restoreStore: store
    )

    #expect(model.deck.layout.focusedSurface?.liveBundle?.launch.currentDirectory == "/tmp/auth-service")
}

@Test("live project focus layouts can retarget focus deterministically")
func liveProjectFocusLayoutCanRetargetFocus() {
    var layout = TerminalDeckLayout.singleLiveDemo
    layout.splitFocusedPane(axis: .vertical)
    let originalPane = layout.paneIDs[0]

    layout.focusPane(originalPane)

    #expect(layout.focusedPaneID == originalPane)
    #expect(layout.focusedSurface?.isLive == true)
}
