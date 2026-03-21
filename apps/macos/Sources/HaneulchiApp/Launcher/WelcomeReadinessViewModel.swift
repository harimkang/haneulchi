import Foundation

struct WelcomeReadinessViewModel: Equatable {
    let selectedProject: LauncherProject?
    let report: ReadinessReport?

    var canContinue: Bool {
        report?.canContinueWithGenericShell == true
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
}
