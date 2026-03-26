@testable import HaneulchiApp
import Testing

@MainActor
@Test("preview fixtures provide populated models for shell routes and supporting surfaces")
func previewFixturesProvidePopulatedModels() {
    let shellModel = HaneulchiPreviewFixtures.shellModel(
        route: .controlTower,
        showCommandPalette: true,
        showQuickDispatch: true,
        showInventory: true,
    )

    #expect(shellModel.entrySurface == .shell)
    #expect(shellModel.selectedRoute == .controlTower)
    #expect(shellModel.shellSnapshot?.projects.isEmpty == false)
    #expect(shellModel.shellSnapshot?.sessions.isEmpty == false)
    #expect(shellModel.settingsStatusViewModel != nil)
    #expect(shellModel.quickDispatchComposer != nil)
    #expect(shellModel.inventoryViewModel != nil)
    #expect(shellModel.commandPaletteViewModel != nil)
}

@Test("preview fixtures provide non-empty task board and review queue projections")
func previewFixturesProvideProjectionContent() {
    let taskBoard = HaneulchiPreviewFixtures.taskBoardProjection()
    let reviewQueue = HaneulchiPreviewFixtures.reviewQueueProjection()

    #expect(taskBoard.columns.isEmpty == false)
    #expect(taskBoard.columns.contains(where: { !$0.tasks.isEmpty }))
    #expect(reviewQueue.items.isEmpty == false)
    #expect(reviewQueue.degradedReason != nil)
}
