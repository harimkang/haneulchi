import Foundation

@MainActor
final class AppShellModel: ObservableObject {
    enum LauncherEntryReason: Equatable {
        case firstRun
        case degradedRecovery
    }

    enum EntrySurface: Equatable {
        case welcome(LauncherEntryReason)
        case shell
    }

    @Published private(set) var entrySurface: EntrySurface
    @Published private(set) var selectedProject: LauncherProject?
    @Published private(set) var recentProjects: [LauncherProject]
    @Published private(set) var readinessReport: ReadinessReport?
    private(set) var startupReadinessTask: Task<Void, Never>?

    private let projectStore: ProjectLauncherStore
    private let restoreStore: TerminalSessionRestoreStore

    init(
        entrySurface: EntrySurface,
        selectedProject: LauncherProject?,
        recentProjects: [LauncherProject],
        readinessReport: ReadinessReport?,
        projectStore: ProjectLauncherStore,
        restoreStore: TerminalSessionRestoreStore
    ) {
        self.entrySurface = entrySurface
        self.selectedProject = selectedProject
        self.recentProjects = recentProjects
        self.readinessReport = readinessReport
        self.projectStore = projectStore
        self.restoreStore = restoreStore
    }

    static func bootstrap(
        projectStore: ProjectLauncherStore,
        restoreStore: TerminalSessionRestoreStore
    ) throws -> AppShellModel {
        let selectedProject = try projectStore.loadLastSelectedProject()
        let recentProjects = try projectStore.loadRecentProjects()
        return AppShellModel(
            entrySurface: selectedProject == nil ? .welcome(.firstRun) : .shell,
            selectedProject: selectedProject,
            recentProjects: recentProjects,
            readinessReport: nil,
            projectStore: projectStore,
            restoreStore: restoreStore
        )
    }

    static func liveDefault(
        projectStore: ProjectLauncherStore = .liveDefault,
        restoreStore: TerminalSessionRestoreStore = .liveDefault,
        readinessRunner: ReadinessProbeRunner = .live
    ) -> AppShellModel {
        let model = (try? bootstrap(projectStore: projectStore, restoreStore: restoreStore))
            ?? AppShellModel(
                entrySurface: .welcome(.firstRun),
                selectedProject: nil,
                recentProjects: [],
                readinessReport: nil,
                projectStore: projectStore,
                restoreStore: restoreStore
            )

        model.refreshStartupReadiness(using: readinessRunner)
        return model
    }

    static func bootstrap(
        projectStore: ProjectLauncherStore,
        restoreStore: TerminalSessionRestoreStore,
        initialReport: ReadinessReport
    ) async throws -> AppShellModel {
        let selectedProject = try projectStore.loadLastSelectedProject()

        if selectedProject != nil, initialReport.requiresRecoverySurface {
            return AppShellModel(
                entrySurface: .welcome(.degradedRecovery),
                selectedProject: selectedProject,
                recentProjects: try projectStore.loadRecentProjects(),
                readinessReport: initialReport,
                projectStore: projectStore,
                restoreStore: restoreStore
            )
        }

        return try bootstrap(projectStore: projectStore, restoreStore: restoreStore)
    }

    func selectProject(_ project: LauncherProject) throws {
        try projectStore.recordOpen(project)
        try projectStore.saveLastSelectedProject(project)
        selectedProject = project
        recentProjects = try projectStore.loadRecentProjects()
    }

    func updateReadinessReport(_ report: ReadinessReport?) {
        readinessReport = report
    }

    func refreshStartupReadiness(using readinessRunner: ReadinessProbeRunner) {
        guard let selectedProject else {
            return
        }

        startupReadinessTask?.cancel()
        startupReadinessTask = Task { [weak self] in
            let report = try? await readinessRunner.run(for: selectedProject)
            await MainActor.run {
                guard let self else {
                    return
                }

                self.readinessReport = report

                if report?.requiresRecoverySurface == true {
                    self.entrySurface = .welcome(.degradedRecovery)
                }
            }
        }
    }

    func presentShell() {
        entrySurface = .shell
    }
}
