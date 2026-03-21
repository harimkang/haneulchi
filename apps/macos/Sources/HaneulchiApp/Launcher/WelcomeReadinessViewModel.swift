import Foundation

struct WelcomeReadinessViewModel: Equatable {
    let entryReason: AppShellModel.LauncherEntryReason
    let recentProjectsCount: Int
    let selectedProject: LauncherProject?
    let report: ReadinessReport?
    let supportsDemoWorkspace: Bool
    let launcherNotice: String?

    var headerTitle: String {
        switch (entryReason, selectedProject) {
        case (.degradedRecovery, _):
            "Recover this workspace"
        case (.firstRun, .some(let project)):
            project.name
        case (.firstRun, nil):
            "Start a workspace"
        }
    }

    var helperText: String {
        switch (entryReason, selectedProject) {
        case (.degradedRecovery, _):
            "Review the current readiness state, then continue with a generic shell or adjust settings."
        case (.firstRun, .some(let project)):
            project.rootPath
        case (.firstRun, nil):
            "Open the demo workspace or add a folder. Generic shell remains available when presets are incomplete."
        }
    }

    var showsDemoWorkspaceAction: Bool {
        entryReason == .firstRun &&
            selectedProject == nil &&
            supportsDemoWorkspace
    }

    var canContinue: Bool {
        report?.canContinueWithGenericShell == true
    }

    var canRetry: Bool {
        selectedProject != nil
    }

    var primaryActionTitle: String {
        "Continue with Generic Shell"
    }

    var settingsTargetRoute: Route {
        .settings
    }

    var readinessChecks: [ReadinessCheck] {
        report?.checks ?? []
    }

    init(
        entryReason: AppShellModel.LauncherEntryReason,
        recentProjectsCount: Int,
        selectedProject: LauncherProject?,
        report: ReadinessReport?,
        supportsDemoWorkspace: Bool,
        launcherNotice: String?
    ) {
        self.entryReason = entryReason
        self.recentProjectsCount = recentProjectsCount
        self.selectedProject = selectedProject
        self.report = report
        self.supportsDemoWorkspace = supportsDemoWorkspace
        self.launcherNotice = launcherNotice
    }

    init(selectedProject: LauncherProject?, report: ReadinessReport?) {
        self.init(
            entryReason: .firstRun,
            recentProjectsCount: 0,
            selectedProject: selectedProject,
            report: report,
            supportsDemoWorkspace: false,
            launcherNotice: nil
        )
    }
}
