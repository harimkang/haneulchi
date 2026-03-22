import Foundation
import SwiftUI

struct AppShellView: View {
    @StateObject private var shellModel: AppShellModel
    @State private var projectFocusModel = AppShellView.bootstrapProjectFocusModel()
    @State private var launcherNotice: String?
    private let projectFolderPicker: ProjectFolderPicker
    private let demoWorkspaceScaffold: DemoWorkspaceScaffold

    init(
        model: @autoclosure @escaping () -> AppShellModel = AppShellModel.liveDefault(),
        projectFolderPicker: ProjectFolderPicker = .live,
        demoWorkspaceScaffold: DemoWorkspaceScaffold = .liveDefault
    ) {
        let resolvedModel = model()
        _shellModel = StateObject(wrappedValue: resolvedModel)
        self.projectFolderPicker = projectFolderPicker
        self.demoWorkspaceScaffold = demoWorkspaceScaffold
    }

    var body: some View {
        Group {
            switch shellModel.entrySurface {
            case .welcome:
                welcomePlaceholder
            case .shell:
                shellLayout
            }
        }
        .task(id: shellModel.selectedProject?.projectID) {
            await MainActor.run {
                guard shellModel.entrySurface == .shell else {
                    return
                }

                projectFocusModel = AppShellView.bootstrapProjectFocusModel(
                    selectedProjectRoot: shellModel.selectedProject?.rootPath
                )
            }
        }
        .overlay(alignment: .top) {
            if
                shellModel.entrySurface == .shell,
                shellModel.isCommandPalettePresented,
                let viewModel = shellModel.commandPaletteViewModel
            {
                CommandPaletteOverlay(viewModel: viewModel) { action in
                    Task {
                        await shellModel.perform(action)
                        await shellModel.perform(.dismissCommandPalette)
                    }
                } onClose: {
                    Task {
                        await shellModel.perform(.dismissCommandPalette)
                    }
                }
            }
        }
        .sheet(isPresented: Binding(
            get: { shellModel.isNewSessionSheetPresented },
            set: { presented in
                if !presented {
                    Task {
                        await shellModel.perform(.dismissNewSessionSheet)
                    }
                }
            }
        )) {
            if let viewModel = shellModel.newSessionSheetViewModel {
                NewSessionSheetView(viewModel: viewModel) { descriptor in
                    Task {
                        await shellModel.perform(.launchSession(descriptor))
                    }
                }
            }
        }
        .sheet(isPresented: Binding(
            get: { shellModel.isWorkflowDrawerPresented },
            set: { presented in
                if !presented {
                    Task {
                        await shellModel.perform(.dismissWorkflowDrawer)
                    }
                }
            }
        )) {
            WorkflowDrawerView(status: shellModel.workflowStatus) {
                Task {
                    await shellModel.perform(.reloadWorkflow)
                }
            }
        }
    }

    private var shellLayout: some View {
        let snapshot = shellModel.shellSnapshot ?? AppShellSnapshot.empty(
            activeRoute: shellModel.selectedRoute,
            selectedProject: shellModel.selectedProject
        )
        let chrome = AppShellChromeState(
            snapshot: snapshot,
            selectedProjectName: shellModel.selectedProject?.name,
            transientNotice: shellModel.transientNotice
        )

        return AppShellChromeView(
            chrome: chrome,
            destination: shellModel.selectedRoute,
            onAction: performAction
        ) {
            RouteDestinationView(
                route: shellModel.selectedRoute,
                snapshot: snapshot,
                projectFocusModel: projectFocusModel,
                settingsStatusViewModel: shellModel.settingsStatusViewModel ?? .empty,
                queuedProjectFocusFilePath: shellModel.pendingProjectFocusFilePath,
                onAction: performAction
            )
        }
    }

    private var welcomePlaceholder: some View {
        WelcomeReadinessView(
            entryReason: launcherEntryReason,
            recentProjects: shellModel.recentProjects,
            selectedProject: shellModel.selectedProject,
            report: shellModel.readinessReport,
            supportsDemoWorkspace: true,
            launcherNotice: launcherNotice,
            addFolder: addFolder,
            openDemoWorkspace: openDemoWorkspace,
            reopenProject: reopenProject,
            continueWithGenericShell: continueWithGenericShell,
            openSettings: openSettings,
            retry: retryReadiness
        )
    }

    private static func bootstrapProjectFocusModel(
        selectedProjectRoot: String? = nil,
        restoreStore: TerminalSessionRestoreStore = .liveDefault
    ) -> ProjectFocusView.Model {
        (try? ProjectFocusView.Model.bootstrap(
            selectedProjectRoot: selectedProjectRoot,
            restoreStore: restoreStore
        )) ?? .demo
    }

    private func continueWithGenericShell() {
        launcherNotice = nil
        projectFocusModel = AppShellView.bootstrapProjectFocusModel(
            selectedProjectRoot: shellModel.selectedProject?.rootPath
        )
        shellModel.setSelectedRoute(.projectFocus)
        shellModel.presentShell()
    }

    private func openSettings() {
        launcherNotice = nil
        Task {
            await shellModel.perform(.openSettings)
            await MainActor.run {
                shellModel.presentShell()
            }
        }
    }

    private func retryReadiness() {
        guard let selectedProject = shellModel.selectedProject else {
            return
        }

        Task {
            let report = try? await ReadinessProbeRunner.live.run(for: selectedProject)
            await MainActor.run {
                shellModel.updateReadinessReport(report)
            }
        }
    }

    private func openDemoWorkspace() {
        guard let project = try? demoWorkspaceScaffold.materialize() else {
            launcherNotice = "Demo workspace could not be prepared. Add a folder or try again."
            return
        }

        selectProjectAndRefreshReadiness(project)
    }

    private func reopenProject(_ project: LauncherProject) {
        selectProjectAndRefreshReadiness(project)
    }

    private func addFolder() {
        guard let url = projectFolderPicker.pickFolder() else {
            return
        }

        let project = LauncherProject(
            projectID: UUID().uuidString,
            name: url.lastPathComponent,
            rootPath: url.path,
            lastOpenedAt: .now
        )
        selectProjectAndRefreshReadiness(project)
    }

    private func selectProjectAndRefreshReadiness(_ project: LauncherProject) {
        launcherNotice = nil
        try? shellModel.selectProject(project)
        Task {
            let report = try? await ReadinessProbeRunner.live.run(for: project)
            await MainActor.run {
                shellModel.updateReadinessReport(report)
            }
        }
    }

    private func performAction(_ action: AppShellAction) {
        Task {
            await shellModel.perform(action)
        }
    }

    private var launcherEntryReason: AppShellModel.LauncherEntryReason {
        switch shellModel.entrySurface {
        case let .welcome(reason):
            reason
        case .shell:
            .firstRun
        }
    }
}

#Preview {
    AppShellView()
}
