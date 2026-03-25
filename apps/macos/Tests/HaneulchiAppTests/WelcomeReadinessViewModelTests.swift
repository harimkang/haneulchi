@testable import HaneulchiApp
import Testing

@Test("first-run empty state offers a demo entry point and onboarding copy")
func welcomeReadinessViewModelShowsFirstRunStartState() {
    let model = WelcomeReadinessViewModel(
        entryReason: .firstRun,
        recentProjectsCount: 0,
        selectedProject: nil,
        report: nil,
        supportsDemoWorkspace: true,
        launcherNotice: nil,
    )

    #expect(model.headerTitle == "Start a workspace")
    #expect(model
        .helperText ==
        "Open the demo workspace or add a folder. Generic shell remains available when presets are incomplete.")
    #expect(model.showsDemoWorkspaceAction == true)
    #expect(model.canRetry == false)
}

@Test("first-run no-project state keeps the demo shortcut even when recent projects exist")
func welcomeReadinessViewModelKeepsDemoShortcutForNoProjectState() {
    let model = WelcomeReadinessViewModel(
        entryReason: .firstRun,
        recentProjectsCount: 2,
        selectedProject: nil,
        report: nil,
        supportsDemoWorkspace: true,
        launcherNotice: nil,
    )

    #expect(model.showsDemoWorkspaceAction == true)
}

@Test("degraded recovery keeps recovery copy and hides the demo shortcut")
func welcomeReadinessViewModelShowsRecoveryState() {
    let project = LauncherProject(
        projectID: "proj_demo",
        name: "demo",
        rootPath: "/tmp/demo",
        lastOpenedAt: .now,
    )

    let model = WelcomeReadinessViewModel(
        entryReason: .degradedRecovery,
        recentProjectsCount: 1,
        selectedProject: project,
        report: nil,
        supportsDemoWorkspace: true,
        launcherNotice: nil,
    )

    #expect(model.headerTitle == "Recover this workspace")
    #expect(model.showsDemoWorkspaceAction == false)
    #expect(model.canRetry == true)
}

@Test("launcher surfaces a notice when demo workspace preparation fails")
func welcomeReadinessViewModelCarriesLauncherNotice() {
    let model = WelcomeReadinessViewModel(
        entryReason: .firstRun,
        recentProjectsCount: 0,
        selectedProject: nil,
        report: nil,
        supportsDemoWorkspace: true,
        launcherNotice: "Demo workspace could not be prepared. Add a folder or try again.",
    )

    #expect(model
        .launcherNotice == "Demo workspace could not be prepared. Add a folder or try again.")
}

@Test("launcher enables continue only when a project is selected and shell probe is not blocked")
func welcomeReadinessViewModelComputesPrimaryActionState() {
    let report = ReadinessReport(
        project: .init(
            projectID: "proj_demo",
            name: "demo",
            rootPath: "/tmp/demo",
            lastOpenedAt: .now,
        ),
        checks: [
            .init(
                name: .shell,
                status: .ready,
                headline: "Shell ready",
                detail: "zsh available",
                nextAction: nil,
            ),
        ],
    )

    let model = WelcomeReadinessViewModel(
        entryReason: .firstRun,
        recentProjectsCount: 1,
        selectedProject: .init(
            projectID: "proj_demo",
            name: "demo",
            rootPath: "/tmp/demo",
            lastOpenedAt: .now,
        ),
        report: report,
        supportsDemoWorkspace: true,
        launcherNotice: nil,
    )

    #expect(model.canContinue == true)
    #expect(model.primaryActionTitle == "Continue with Generic Shell")
}

@Test("saved project with informational gaps still presents the normal project summary copy")
func welcomeReadinessViewModelKeepsProjectSummaryForInformationalGaps() {
    let project = LauncherProject(
        projectID: "proj_demo",
        name: "demo",
        rootPath: "/tmp/demo",
        lastOpenedAt: .now,
    )
    let report = ReadinessReport(
        project: project,
        checks: [
            .init(
                name: .shell,
                status: .ready,
                headline: "Shell ready",
                detail: "/bin/zsh",
                nextAction: nil,
            ),
            .init(
                name: .shellIntegration,
                status: .degraded,
                headline: "Shell integration not installed",
                detail: "Command markers are not configured yet.",
                nextAction: "Open Settings",
            ),
            .init(
                name: .workflow,
                status: .degraded,
                headline: "Workflow contract not found",
                detail: "Future launches can still use a generic shell.",
                nextAction: "Continue with Generic Shell",
            ),
        ],
    )

    let model = WelcomeReadinessViewModel(
        entryReason: .firstRun,
        recentProjectsCount: 1,
        selectedProject: project,
        report: report,
        supportsDemoWorkspace: true,
        launcherNotice: nil,
    )

    #expect(model.headerTitle == "demo")
    #expect(model.helperText == "/tmp/demo")
    #expect(model.showsDemoWorkspaceAction == false)
}

@Test("open settings targets the documented settings route")
func welcomeReadinessViewModelOpensSettingsRoute() {
    let model = WelcomeReadinessViewModel(
        entryReason: .firstRun,
        recentProjectsCount: 0,
        selectedProject: nil,
        report: nil,
        supportsDemoWorkspace: true,
        launcherNotice: nil,
    )
    #expect(model.settingsTargetRoute == .settings)
}
