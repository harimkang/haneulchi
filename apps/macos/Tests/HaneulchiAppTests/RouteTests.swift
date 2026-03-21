import Testing
@testable import HaneulchiApp

@Test("app shell exposes the documented primary routes")
func primaryRoutes() {
    #expect(Route.allCases.count == 5)
    #expect(Route.projectFocus.title == "Project Focus")
    #expect(Route.controlTower.title == "Control Tower")
    #expect(Route.taskBoard.title == "Task Board")
    #expect(Route.review.title == "Review")
    #expect(Route.attention.title == "Attention")
}
