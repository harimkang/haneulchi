@testable import HaneulchiApp
import Testing

@Test("app shell exposes the documented primary routes")
func primaryRoutes() {
    #expect(Route.allCases.count == 6)
    #expect(Route.primaryCases == [
        .projectFocus,
        .controlTower,
        .taskBoard,
        .reviewQueue,
        .attentionCenter,
    ])
    #expect(Route.projectFocus.rawValue == "project_focus")
    #expect(Route.projectFocus.title == "Project Focus")
    #expect(Route.controlTower.title == "Control Tower")
    #expect(Route.taskBoard.title == "Task Board")
    #expect(Route.reviewQueue.title == "Review Queue")
    #expect(Route.attentionCenter.title == "Attention Center")
    #expect(Route.settings.title == "Settings")
    #expect(Route.attentionCenter.shortcutLabel == "Cmd+5")
    #expect(Route.projectFocus.keyboardShortcut == .init("1", modifiers: [.command]))
    #expect(Route.controlTower.keyboardShortcut == .init("2", modifiers: [.command]))
    #expect(Route.taskBoard.keyboardShortcut == .init("3", modifiers: [.command]))
    #expect(Route.reviewQueue.keyboardShortcut == .init("4", modifiers: [.command]))
    #expect(Route.attentionCenter.keyboardShortcut == .init("5", modifiers: [.command]))
    #expect(Route.latestUnreadShortcut == .init("u", modifiers: [.command, .shift]))
}
