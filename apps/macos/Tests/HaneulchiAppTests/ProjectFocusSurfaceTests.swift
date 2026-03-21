import Testing
@testable import HaneulchiApp

@Test("project focus demo boots with one hosted surface and split disabled")
func projectFocusDemoSurfaceContract() {
    let model = ProjectFocusView.Model.demo

    #expect(model.deck.surfaces.count == 1)
    #expect(model.deck.showsSplitControls == false)
    #expect(model.deck.surfaces.first?.fixtureName == "hello-world.ansi")
}
