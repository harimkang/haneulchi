import Testing
@testable import HaneulchiApp

@Test("launcher enables continue only when a project is selected and shell probe is not blocked")
func welcomeReadinessViewModelComputesPrimaryActionState() {
    let report = ReadinessReport(
        project: .init(projectID: "proj_demo", name: "demo", rootPath: "/tmp/demo", lastOpenedAt: .now),
        checks: [
            .init(name: .shell, status: .ready, headline: "Shell ready", detail: "zsh available", nextAction: nil),
        ]
    )

    let model = WelcomeReadinessViewModel(
        selectedProject: .init(projectID: "proj_demo", name: "demo", rootPath: "/tmp/demo", lastOpenedAt: .now),
        report: report
    )

    #expect(model.canContinue == true)
    #expect(model.primaryActionTitle == "Continue with Generic Shell")
}

@Test("open settings targets the documented settings route")
func welcomeReadinessViewModelOpensSettingsRoute() {
    let model = WelcomeReadinessViewModel(selectedProject: nil, report: nil)
    #expect(model.settingsTargetRoute == .settings)
}
