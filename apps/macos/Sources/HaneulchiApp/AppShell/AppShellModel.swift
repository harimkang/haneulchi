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
    @Published private(set) var selectedRoute: Route
    @Published private(set) var selectedProject: LauncherProject?
    @Published private(set) var recentProjects: [LauncherProject]
    @Published private(set) var readinessReport: ReadinessReport?
    @Published private(set) var shellSnapshot: AppShellSnapshot?
    @Published private(set) var isNewSessionSheetPresented = false
    @Published private(set) var newSessionSheetViewModel: NewSessionSheetViewModel?
    @Published private(set) var isCommandPalettePresented = false
    @Published private(set) var commandPaletteViewModel: CommandPaletteViewModel?
    @Published private(set) var transientNotice: String?
    private(set) var startupReadinessTask: Task<Void, Never>?

    private let projectStore: ProjectLauncherStore
    private let restoreStore: TerminalSessionRestoreStore
    private let preferencesStore: AppShellPreferencesStore
    private let snapshotSource: LocalAppShellSnapshotSource
    private let readinessRunner: ReadinessProbeRunner
    private let projectFileIndex: ProjectFileIndex
    private let taskSearchProjectionStore: TaskSearchProjectionStore
    private let inventorySearchProjectionStore: InventorySearchProjectionStore
    private let presetRegistry: PresetRegistry
    private let coreBridge: CoreBridge?

    init(
        entrySurface: EntrySurface,
        selectedRoute: Route,
        selectedProject: LauncherProject?,
        recentProjects: [LauncherProject],
        readinessReport: ReadinessReport?,
        projectStore: ProjectLauncherStore,
        restoreStore: TerminalSessionRestoreStore,
        preferencesStore: AppShellPreferencesStore,
        snapshotSource: LocalAppShellSnapshotSource? = nil,
        readinessRunner: ReadinessProbeRunner = .live,
        projectFileIndex: ProjectFileIndex = ProjectFileIndex(),
        taskSearchProjectionStore: TaskSearchProjectionStore = .liveDefault,
        inventorySearchProjectionStore: InventorySearchProjectionStore? = nil,
        presetRegistry: PresetRegistry? = nil,
        coreBridge: CoreBridge? = nil,
        shellSnapshot: AppShellSnapshot? = nil,
        isNewSessionSheetPresented: Bool = false,
        newSessionSheetViewModel: NewSessionSheetViewModel? = nil,
        isCommandPalettePresented: Bool = false,
        commandPaletteViewModel: CommandPaletteViewModel? = nil,
        transientNotice: String? = nil
    ) {
        self.entrySurface = entrySurface
        self.selectedRoute = selectedRoute
        self.selectedProject = selectedProject
        self.recentProjects = recentProjects
        self.readinessReport = readinessReport
        self.shellSnapshot = shellSnapshot
        self.isNewSessionSheetPresented = isNewSessionSheetPresented
        self.newSessionSheetViewModel = newSessionSheetViewModel
        self.isCommandPalettePresented = isCommandPalettePresented
        self.commandPaletteViewModel = commandPaletteViewModel
        self.transientNotice = transientNotice
        self.projectStore = projectStore
        self.restoreStore = restoreStore
        self.preferencesStore = preferencesStore
        self.snapshotSource = snapshotSource ?? LocalAppShellSnapshotSource(restoreStore: restoreStore)
        self.readinessRunner = readinessRunner
        self.projectFileIndex = projectFileIndex
        self.taskSearchProjectionStore = taskSearchProjectionStore
        self.inventorySearchProjectionStore = inventorySearchProjectionStore ?? InventorySearchProjectionStore(restoreStore: restoreStore)
        self.presetRegistry = presetRegistry ?? (try? PresetRegistry.loadDefault()) ?? PresetRegistry(presets: [])
        self.coreBridge = coreBridge
    }

    static func bootstrap(
        projectStore: ProjectLauncherStore,
        restoreStore: TerminalSessionRestoreStore,
        preferencesStore: AppShellPreferencesStore,
        readinessRunner: ReadinessProbeRunner = .live
    ) throws -> AppShellModel {
        let selectedProject = try projectStore.loadLastSelectedProject()
        let recentProjects = try projectStore.loadRecentProjects()
        let preferences = try preferencesStore.load()
        return AppShellModel(
            entrySurface: selectedProject == nil ? .welcome(.firstRun) : .shell,
            selectedRoute: selectedProject == nil ? .projectFocus : preferences.lastActiveRoute,
            selectedProject: selectedProject,
            recentProjects: recentProjects,
            readinessReport: nil,
            projectStore: projectStore,
            restoreStore: restoreStore,
            preferencesStore: preferencesStore,
            readinessRunner: readinessRunner
        )
    }

    static func liveDefault(
        projectStore: ProjectLauncherStore = .liveDefault,
        restoreStore: TerminalSessionRestoreStore = .liveDefault,
        preferencesStore: AppShellPreferencesStore = .liveDefault,
        readinessRunner: ReadinessProbeRunner = .live
    ) -> AppShellModel {
        let model = (try? bootstrap(
            projectStore: projectStore,
            restoreStore: restoreStore,
            preferencesStore: preferencesStore,
            readinessRunner: readinessRunner
        ))
            ?? AppShellModel(
                entrySurface: .welcome(.firstRun),
                selectedRoute: .projectFocus,
                selectedProject: nil,
                recentProjects: [],
                readinessReport: nil,
                projectStore: projectStore,
                restoreStore: restoreStore,
                preferencesStore: preferencesStore,
                readinessRunner: readinessRunner
            )

        model.refreshStartupReadiness(using: readinessRunner)
        Task {
            await model.refreshShellSnapshot()
        }
        return model
    }

    static func bootstrap(
        projectStore: ProjectLauncherStore,
        restoreStore: TerminalSessionRestoreStore,
        preferencesStore: AppShellPreferencesStore,
        readinessRunner: ReadinessProbeRunner = .live,
        initialReport: ReadinessReport
    ) async throws -> AppShellModel {
        let selectedProject = try projectStore.loadLastSelectedProject()
        let preferences = try preferencesStore.load()

        if selectedProject != nil, initialReport.requiresRecoverySurface {
            return AppShellModel(
                entrySurface: .welcome(.degradedRecovery),
                selectedRoute: preferences.lastActiveRoute,
                selectedProject: selectedProject,
                recentProjects: try projectStore.loadRecentProjects(),
                readinessReport: initialReport,
                projectStore: projectStore,
                restoreStore: restoreStore,
                preferencesStore: preferencesStore,
                readinessRunner: readinessRunner
            )
        }

        return try bootstrap(
            projectStore: projectStore,
            restoreStore: restoreStore,
            preferencesStore: preferencesStore,
            readinessRunner: readinessRunner
        )
    }

    func selectProject(_ project: LauncherProject) throws {
        try projectStore.recordOpen(project)
        try projectStore.saveLastSelectedProject(project)
        selectedProject = project
        recentProjects = try projectStore.loadRecentProjects()
        Task {
            await refreshShellSnapshot()
        }
    }

    func updateReadinessReport(_ report: ReadinessReport?) {
        readinessReport = report
        Task {
            await refreshShellSnapshot()
        }
    }

    func setSelectedRoute(_ route: Route) {
        selectedRoute = route
        try? preferencesStore.save(.init(lastActiveRoute: route))
    }

    func refreshShellSnapshot() async {
        if let coreBridge, let snapshot = try? coreBridge.stateSnapshot() {
            shellSnapshot = snapshot
            return
        }

        shellSnapshot = try? await snapshotSource.load(
            activeRoute: selectedRoute,
            selectedProject: selectedProject,
            readinessReport: readinessReport,
            recentProjects: recentProjects
        )
    }

    func perform(_ action: AppShellAction) async {
        switch action {
        case let .selectRoute(route):
            setSelectedRoute(route)
        case .openSettings:
            setSelectedRoute(.settings)
        case .presentNewSessionSheet:
            newSessionSheetViewModel = NewSessionSheetViewModel(
                selectedProjectRoot: selectedProject?.rootPath,
                registry: presetRegistry,
                workflowSummary: nil
            )
            isNewSessionSheetPresented = true
        case .dismissNewSessionSheet:
            isNewSessionSheetPresented = false
            newSessionSheetViewModel = nil
        case let .launchSession(descriptor):
            do {
                var bundles = try restoreStore.load()
                bundles.append(descriptor.restoreBundle)
                try restoreStore.save(bundles)
                setSelectedRoute(.projectFocus)
                transientNotice = "Session launched: \(descriptor.title)"
                isNewSessionSheetPresented = false
                newSessionSheetViewModel = nil
            } catch {
                transientNotice = "Session could not be launched. Check restore storage and try again."
            }
        case .toggleCommandPalette:
            if isCommandPalettePresented {
                isCommandPalettePresented = false
                commandPaletteViewModel = nil
            } else {
                if shellSnapshot == nil {
                    await refreshShellSnapshot()
                }
                await refreshCommandPaletteViewModel()
                isCommandPalettePresented = true
            }
        case .dismissCommandPalette:
            isCommandPalettePresented = false
            commandPaletteViewModel = nil
        case let .queueFileSelection(path):
            setSelectedRoute(.projectFocus)
            transientNotice = "File queued for Project Focus: \(path)"
        case let .createTaskDraft(title):
            do {
                let row = try taskSearchProjectionStore.createDraft(title, selectedProject?.projectID)
                setSelectedRoute(.taskBoard)
                transientNotice = "Task draft created: \(row.title)"
            } catch {
                transientNotice = "Task draft could not be created. Check local task storage and try again."
            }
        case let .jumpToSession(sessionID):
            setSelectedRoute(.projectFocus)
            if let coreBridge {
                do {
                    try coreBridge.focusSession(sessionID)
                    transientNotice = "Focused session \(sessionID)"
                } catch {
                    transientNotice = "Focus requested for session \(sessionID)"
                }
            } else {
                transientNotice = "Focus requested for session \(sessionID)"
            }
        case .jumpToLatestUnread:
            if shellSnapshot == nil {
                await refreshShellSnapshot()
            }
            if let latestAttention = shellSnapshot?.attention.first {
                setSelectedRoute(latestAttention.targetRoute)
                transientNotice = "Jumped to latest unread: \(latestAttention.headline)"
            }
        case .retryReadiness:
            await retryStartupReadiness()
        }

        if action != .dismissCommandPalette && action != .toggleCommandPalette {
            await refreshShellSnapshot()
        }

        if isCommandPalettePresented {
            await refreshCommandPaletteViewModel()
        }
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

                Task {
                    await self.refreshShellSnapshot()
                }
            }
        }
    }

    func presentShell() {
        entrySurface = .shell
    }

    func refreshCommandPaletteViewModel() async {
        let snapshot = shellSnapshot ?? AppShellSnapshot.empty(
            activeRoute: selectedRoute,
            selectedProject: selectedProject
        )
        let files: [ProjectFileIndex.Entry]
        if let rootPath = selectedProject?.rootPath {
            files = (try? await projectFileIndex.index(rootPath: rootPath)) ?? []
        } else {
            files = []
        }

        let tasks = (try? taskSearchProjectionStore.search("")) ?? []
        let inventory = (try? await inventorySearchProjectionStore.load(selectedProjectRoot: selectedProject?.rootPath)) ?? []

        commandPaletteViewModel = CommandPaletteViewModel(
            catalog: .build(
                snapshot: snapshot,
                files: files,
                tasks: tasks,
                inventory: inventory
            )
        )
    }

    private func retryStartupReadiness() async {
        guard let selectedProject else {
            return
        }

        let report = try? await readinessRunner.run(for: selectedProject)
        updateReadinessReport(report)
    }
}
