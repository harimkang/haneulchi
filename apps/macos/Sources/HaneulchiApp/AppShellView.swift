import Foundation
import SwiftUI

struct AppShellView: View {
    @StateObject private var shellModel: AppShellModel
    @State private var selectedRoute: Route? = .projectFocus
    @State private var projectFocusModel = AppShellView.bootstrapProjectFocusModel()
    private let projectFolderPicker: ProjectFolderPicker

    init(
        model: @autoclosure @escaping () -> AppShellModel = AppShellModel.liveDefault(),
        projectFolderPicker: ProjectFolderPicker = .live
    ) {
        _shellModel = StateObject(wrappedValue: model())
        self.projectFolderPicker = projectFolderPicker
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
    }

    private var shellLayout: some View {
        NavigationSplitView {
            List(Route.allCases, selection: $selectedRoute) { route in
                Label(route.title, systemImage: route.symbolName)
                    .tag(route)
            }
            .navigationTitle("Haneulchi")
        } detail: {
            detailView
        }
    }

    @ViewBuilder
    private var detailView: some View {
        switch selectedRoute {
        case .projectFocus:
            ProjectFocusView(model: projectFocusModel)
        case .settings:
            SettingsView(report: shellModel.readinessReport)
        case .controlTower, .taskBoard, .review, .attention:
            placeholderDetail
        case nil:
            placeholderDetail
        }
    }

    private var welcomePlaceholder: some View {
        WelcomeReadinessView(
            recentProjects: shellModel.recentProjects,
            selectedProject: shellModel.selectedProject,
            report: shellModel.readinessReport,
            addFolder: addFolder,
            reopenProject: reopenProject,
            continueWithGenericShell: continueWithGenericShell,
            openSettings: openSettings,
            retry: retryReadiness
        )
    }

    private var placeholderDetail: some View {
        VStack(alignment: .leading, spacing: 16) {
            Text(selectedRoute?.title ?? "No Selection")
                .font(.largeTitle)
                .bold()
            Text("Initial shell scaffold aligned to Sprint 1 foundation.")
                .foregroundStyle(.secondary)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        .padding(24)
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
        projectFocusModel = AppShellView.bootstrapProjectFocusModel(
            selectedProjectRoot: shellModel.selectedProject?.rootPath
        )
        selectedRoute = .projectFocus
        shellModel.presentShell()
    }

    private func openSettings() {
        selectedRoute = .settings
        shellModel.presentShell()
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

    private func reopenProject(_ project: LauncherProject) {
        try? shellModel.selectProject(project)
        Task {
            let report = try? await ReadinessProbeRunner.live.run(for: project)
            await MainActor.run {
                shellModel.updateReadinessReport(report)
            }
        }
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
        try? shellModel.selectProject(project)
        Task {
            let report = try? await ReadinessProbeRunner.live.run(for: project)
            await MainActor.run {
                shellModel.updateReadinessReport(report)
            }
        }
    }
}

#Preview {
    AppShellView()
}
