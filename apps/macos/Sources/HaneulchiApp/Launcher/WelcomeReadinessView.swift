import SwiftUI

struct WelcomeReadinessView: View {
    let entryReason: AppShellModel.LauncherEntryReason
    let recentProjects: [LauncherProject]
    let selectedProject: LauncherProject?
    let report: ReadinessReport?
    let supportsDemoWorkspace: Bool
    let addFolder: () -> Void
    let openDemoWorkspace: () -> Void
    let reopenProject: (LauncherProject) -> Void
    let continueWithGenericShell: () -> Void
    let openSettings: () -> Void
    let retry: () -> Void

    private var viewModel: WelcomeReadinessViewModel {
        WelcomeReadinessViewModel(
            entryReason: entryReason,
            recentProjectsCount: recentProjects.count,
            selectedProject: selectedProject,
            report: report,
            supportsDemoWorkspace: supportsDemoWorkspace
        )
    }

    var body: some View {
        HStack(alignment: .top, spacing: HaneulchiChrome.Spacing.panelGap) {
            recentProjectsPane
            readinessPane
        }
        .padding(HaneulchiChrome.Spacing.screenPadding)
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        .background(HaneulchiChrome.Colors.appBackground)
    }

    private var recentProjectsPane: some View {
        VStack(alignment: .leading, spacing: HaneulchiChrome.Spacing.itemGap) {
            Text("Welcome")
                .font(.largeTitle.weight(.bold))
            Text("Add a project or reopen a recent workspace. Start with a generic shell and add setup later.")
                .foregroundStyle(HaneulchiChrome.Colors.mutedText)

            Button("Add Folder", action: addFolder)
                .buttonStyle(.borderedProminent)

            if viewModel.showsDemoWorkspaceAction {
                VStack(alignment: .leading, spacing: 8) {
                    Text("Quick Start")
                        .font(.headline)
                    Text("Open the demo workspace to verify the launcher flow before selecting your own repository.")
                        .foregroundStyle(HaneulchiChrome.Colors.mutedText)
                    Button("Open Demo Workspace", action: openDemoWorkspace)
                        .buttonStyle(.bordered)
                }
            }

            VStack(alignment: .leading, spacing: 8) {
                Text("Recent Projects")
                    .font(.headline)
                if recentProjects.isEmpty {
                    Text("No recent projects yet. Add a folder or open the demo workspace to start.")
                        .foregroundStyle(HaneulchiChrome.Colors.mutedText)
                } else {
                    ForEach(recentProjects) { project in
                        Button {
                            reopenProject(project)
                        } label: {
                            VStack(alignment: .leading, spacing: 4) {
                                Text(project.name)
                                Text(project.rootPath)
                                    .font(.caption)
                                    .foregroundStyle(HaneulchiChrome.Colors.mutedText)
                            }
                            .frame(maxWidth: .infinity, alignment: .leading)
                        }
                        .buttonStyle(.plain)
                        .padding(.vertical, 6)
                    }
                }
            }
        }
        .padding(HaneulchiChrome.Spacing.panelPadding)
        .frame(maxWidth: 320, alignment: .topLeading)
        .background(HaneulchiChrome.Colors.secondaryPanel)
        .clipShape(RoundedRectangle(cornerRadius: 18))
    }

    private var readinessPane: some View {
        VStack(alignment: .leading, spacing: HaneulchiChrome.Spacing.itemGap) {
            Text(viewModel.headerTitle)
                .font(.title2.weight(.semibold))
            Text(viewModel.helperText)
                .foregroundStyle(HaneulchiChrome.Colors.mutedText)

            if viewModel.readinessChecks.isEmpty {
                Text("Select a workspace to inspect shell, git, preset, keychain, and workflow readiness.")
                    .foregroundStyle(HaneulchiChrome.Colors.mutedText)
            } else {
                ForEach(viewModel.readinessChecks, id: \.name) { check in
                    readinessRow(check)
                }
            }

            HStack(spacing: 12) {
                Button(viewModel.primaryActionTitle, action: continueWithGenericShell)
                    .buttonStyle(.borderedProminent)
                    .disabled(!viewModel.canContinue)
                Button("Open Settings", action: openSettings)
                    .buttonStyle(.bordered)
                Button("Retry", action: retry)
                    .buttonStyle(.borderless)
                    .disabled(!viewModel.canRetry)
            }
        }
        .padding(HaneulchiChrome.Spacing.panelPadding)
        .frame(maxWidth: .infinity, alignment: .topLeading)
        .background(HaneulchiChrome.Colors.primaryPanel)
        .clipShape(RoundedRectangle(cornerRadius: 18))
    }

    private func readinessRow(_ check: ReadinessCheck) -> some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack {
                Text(check.headline)
                    .font(.headline)
                Spacer()
                Text(statusLabel(check.status))
                    .font(.caption.weight(.semibold))
                    .foregroundStyle(statusColor(check.status))
            }
            Text(check.detail)
                .foregroundStyle(HaneulchiChrome.Colors.mutedText)
            if let nextAction = check.nextAction {
                Text("Next: \(nextAction)")
                    .font(.caption)
                    .foregroundStyle(HaneulchiChrome.Colors.mutedText)
            }
        }
        .padding(.vertical, 4)
    }

    private func statusLabel(_ status: ReadinessCheckStatus) -> String {
        switch status {
        case .ready:
            "READY"
        case .degraded:
            "DEGRADED"
        case .blocked:
            "BLOCKED"
        }
    }

    private func statusColor(_ status: ReadinessCheckStatus) -> Color {
        switch status {
        case .ready:
            HaneulchiChrome.Colors.ready
        case .degraded:
            HaneulchiChrome.Colors.warning
        case .blocked:
            HaneulchiChrome.Colors.blocked
        }
    }
}
