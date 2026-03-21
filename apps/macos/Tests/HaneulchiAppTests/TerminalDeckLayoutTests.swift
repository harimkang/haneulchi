import Testing
@testable import HaneulchiApp

@Test("deck layout can split a focused pane horizontally")
func deckSplitsFocusedPaneHorizontally() {
    var layout = TerminalDeckLayout.singleDemo
    let focused = layout.focusedPaneID

    layout.splitFocusedPane(axis: .horizontal)

    #expect(layout.paneIDs.count == 2)
    #expect(layout.focusedPaneID != focused)
}

@Test("deck layout moves focus in presentation order")
func deckMovesFocusInPresentationOrder() {
    var layout = TerminalDeckLayout.singleDemo

    layout.splitFocusedPane(axis: .vertical)
    let initiallyFocused = layout.focusedPaneID

    layout.moveFocusForward()

    #expect(layout.focusedPaneID != initiallyFocused)
}

@Test("deck layout updates the root split ratio")
func deckUpdatesRootSplitRatio() {
    var layout = TerminalDeckLayout.singleDemo

    layout.splitFocusedPane(axis: .vertical)
    let rootSplitID = try! #require(layout.rootSplitID)

    layout.setSplitRatio(0.7, for: rootSplitID)

    #expect(layout.rootSplitRatio == 0.7)
}

@Test("splitting a live pane preserves a live surface")
func splittingLivePanePreservesLiveSurface() {
    var layout = TerminalDeckLayout.singleLiveDemo

    layout.splitFocusedPane(axis: .horizontal)

    #expect(layout.paneIDs.count == 2)
    #expect(layout.focusedSurface?.isLive == true)
}
