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
    @Published private(set) var pendingProjectFocusFilePath: String?
    @Published private(set) var isWorkflowDrawerPresented = false
    @Published private(set) var workflowStatus: WorkflowStatusPayload?
    @Published private(set) var settingsStatusViewModel: SettingsStatusViewModel?
    @Published private(set) var isTaskContextDrawerPresented = false
    @Published private(set) var taskContextDrawerModel: TaskDrawerModel?
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
        pendingProjectFocusFilePath: String? = nil,
        isWorkflowDrawerPresented: Bool = false,
        workflowStatus: WorkflowStatusPayload? = nil,
        settingsStatusViewModel: SettingsStatusViewModel? = nil,
        isTaskContextDrawerPresented: Bool = false,
        taskContextDrawerModel: TaskDrawerModel? = nil,
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
        self.pendingProjectFocusFilePath = pendingProjectFocusFilePath
        self.isWorkflowDrawerPresented = isWorkflowDrawerPresented
        self.workflowStatus = workflowStatus
        self.settingsStatusViewModel = settingsStatusViewModel
        self.isTaskContextDrawerPresented = isTaskContextDrawerPresented
        self.taskContextDrawerModel = taskContextDrawerModel
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
        readinessRunner: ReadinessProbeRunner = .live,
        coreBridge: CoreBridge? = nil
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
                readinessRunner: readinessRunner,
                coreBridge: coreBridge
            )

        if model.coreBridge == nil, let coreBridge {
            let bridgedModel = AppShellModel(
                entrySurface: model.entrySurface,
                selectedRoute: model.selectedRoute,
                selectedProject: model.selectedProject,
                recentProjects: model.recentProjects,
                readinessReport: model.readinessReport,
                projectStore: projectStore,
                restoreStore: restoreStore,
                preferencesStore: preferencesStore,
                readinessRunner: readinessRunner,
                coreBridge: coreBridge,
                shellSnapshot: model.shellSnapshot,
                pendingProjectFocusFilePath: model.pendingProjectFocusFilePath,
                isWorkflowDrawerPresented: model.isWorkflowDrawerPresented,
                workflowStatus: model.workflowStatus,
                settingsStatusViewModel: model.settingsStatusViewModel,
                isTaskContextDrawerPresented: model.isTaskContextDrawerPresented,
                taskContextDrawerModel: model.taskContextDrawerModel,
                isNewSessionSheetPresented: model.isNewSessionSheetPresented,
                newSessionSheetViewModel: model.newSessionSheetViewModel,
                isCommandPalettePresented: model.isCommandPalettePresented,
                commandPaletteViewModel: model.commandPaletteViewModel,
                transientNotice: model.transientNotice
            )
            bridgedModel.refreshStartupReadiness(using: readinessRunner)
            Task { await bridgedModel.refreshShellSnapshot() }
            return bridgedModel
        }

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
        if selectedRoute == .settings {
            settingsStatusViewModel = makeSettingsStatusViewModel()
        }
        Task {
            await refreshShellSnapshot()
        }
    }

    func setSelectedRoute(_ route: Route) {
        selectedRoute = route
        try? preferencesStore.save(.init(lastActiveRoute: route))
    }

    func refreshShellSnapshot() async {
        let localSnapshot = try? await snapshotSource.load(
            activeRoute: selectedRoute,
            selectedProject: selectedProject,
            readinessReport: readinessReport,
            recentProjects: recentProjects
        )
        if let coreBridge, let bridgeSnapshot = try? coreBridge.stateSnapshot() {
            let merged = mergedSnapshot(local: localSnapshot, bridge: bridgeSnapshot)
            shellSnapshot = merged
            syncWorkflowStatus(from: merged)
            return
        }

        shellSnapshot = localSnapshot
    }

    func perform(_ action: AppShellAction) async {
        switch action {
        case let .selectRoute(route):
            setSelectedRoute(route)
        case .openSettings:
            if let selectedProject, let coreBridge, let payload = try? coreBridge.workflowValidate(selectedProject.rootPath) {
                workflowStatus = try? JSONDecoder().decode(WorkflowStatusPayload.self, from: payload)
            }
            settingsStatusViewModel = makeSettingsStatusViewModel()
            setSelectedRoute(.settings)
        case .presentWorkflowDrawer:
            if let selectedProject, let coreBridge, let payload = try? coreBridge.workflowValidate(selectedProject.rootPath) {
                workflowStatus = try? JSONDecoder().decode(WorkflowStatusPayload.self, from: payload)
            }
            isWorkflowDrawerPresented = true
        case .dismissWorkflowDrawer:
            isWorkflowDrawerPresented = false
        case .presentTaskContextDrawer:
            taskContextDrawerModel = makeTaskContextDrawerModel()
            isTaskContextDrawerPresented = true
        case .dismissTaskContextDrawer:
            isTaskContextDrawerPresented = false
        case .reloadWorkflow:
            guard let selectedProject, let coreBridge else {
                return
            }
            if let payload = try? coreBridge.workflowReload(selectedProject.rootPath) {
                workflowStatus = try? JSONDecoder().decode(WorkflowStatusPayload.self, from: payload)
            }
            if selectedRoute == .settings {
                settingsStatusViewModel = makeSettingsStatusViewModel()
            }
            if isTaskContextDrawerPresented {
                taskContextDrawerModel = makeTaskContextDrawerModel()
            }
        case .presentNewSessionSheet:
            let resolvedWorkflowSummary = selectedProject.flatMap { project in
                fetchWorkflowStatus(for: project)?.workflow.map {
                    WorkflowLaunchSummary(
                        name: $0.name ?? project.name,
                        strategy: $0.strategy ?? "worktree",
                        baseRoot: $0.baseRoot ?? ".",
                        reviewChecklist: $0.reviewChecklist,
                        allowedAgents: $0.allowedAgents
                    )
                }
            }
            newSessionSheetViewModel = NewSessionSheetViewModel(
                selectedProjectRoot: selectedProject?.rootPath,
                selectedTaskID: taskContextDrawerModel?.taskID,
                registry: presetRegistry,
                workflowSummary: resolvedWorkflowSummary,
                provisionIsolatedWorkspace: { [coreBridge] projectRoot, taskID in
                    guard let coreBridge else {
                        throw NewSessionSheetError.isolatedProvisionUnavailable
                    }
                    return try coreBridge.provisionTaskWorkspace(
                        projectRoot,
                        taskID,
                        resolvedWorkflowSummary?.baseRoot
                    )
                }
            )
            isNewSessionSheetPresented = true
        case .dismissNewSessionSheet:
            isNewSessionSheetPresented = false
            newSessionSheetViewModel = nil
        case let .launchSession(descriptor):
            do {
                let bootstrappedDescriptor = try bootstrapIfNeeded(descriptor)
                var bundles = try restoreStore.load()
                bundles.append(bootstrappedDescriptor.restoreBundle)
                try restoreStore.save(bundles)
                setSelectedRoute(.projectFocus)
                transientNotice = "Session launched: \(bootstrappedDescriptor.title)"
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
            pendingProjectFocusFilePath = path
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

    private func fetchWorkflowStatus(for project: LauncherProject) -> WorkflowStatusPayload? {
        guard let coreBridge, let payload = try? coreBridge.workflowValidate(project.rootPath) else {
            return nil
        }

        return try? JSONDecoder().decode(WorkflowStatusPayload.self, from: payload)
    }

    private func makeSettingsStatusViewModel() -> SettingsStatusViewModel {
        SettingsStatusViewModel(
            report: readinessReport,
            workflowStatus: workflowStatus,
            presetRegistry: presetRegistry,
            runtimeInfo: try? coreBridge?.runtimeInfo()
        )
    }

    private func syncWorkflowStatus(from snapshot: AppShellSnapshot) {
        guard let workflow = snapshot.workflow else {
            workflowStatus = nil
            if selectedRoute == .settings {
                settingsStatusViewModel = makeSettingsStatusViewModel()
            }
            if isTaskContextDrawerPresented {
                taskContextDrawerModel = makeTaskContextDrawerModel()
            }
            return
        }

        let summary = workflowStatus?.path == workflow.path ? workflowStatus?.workflow : nil
        workflowStatus = WorkflowStatusPayload(
            state: workflow.state,
            path: workflow.path,
            lastGoodHash: workflow.lastGoodHash,
            lastReloadAt: workflow.lastReloadAt,
            lastError: workflow.lastError,
            workflow: summary
        )

        if selectedRoute == .settings {
            settingsStatusViewModel = makeSettingsStatusViewModel()
        }
        if isTaskContextDrawerPresented {
            taskContextDrawerModel = makeTaskContextDrawerModel()
        }
    }

    private func makeTaskContextDrawerModel() -> TaskDrawerModel? {
        guard let snapshot = shellSnapshot else {
            return nil
        }

        let fetchedWorkflowStatus = selectedProject.flatMap(fetchWorkflowStatus)
        let resolvedWorkflowStatus = workflowStatus ?? fetchedWorkflowStatus
        let mergedWorkflowStatus = resolvedWorkflowStatus.map { resolved in
            WorkflowStatusPayload(
                state: resolved.state,
                path: resolved.path,
                lastGoodHash: resolved.lastGoodHash ?? fetchedWorkflowStatus?.lastGoodHash,
                lastReloadAt: resolved.lastReloadAt ?? fetchedWorkflowStatus?.lastReloadAt,
                lastError: resolved.lastError ?? fetchedWorkflowStatus?.lastError,
                workflow: resolved.workflow ?? fetchedWorkflowStatus?.workflow
            )
        }

        return TaskDrawerModel.resolve(from: snapshot, workflowStatus: mergedWorkflowStatus)
    }

    private func bootstrapIfNeeded(_ descriptor: SessionLaunchDescriptor) throws -> SessionLaunchDescriptor {
        guard descriptor.mode == .isolated, let selectedProject else {
            return descriptor
        }

        let workflowStatus = workflowStatus ?? fetchWorkflowStatus(for: selectedProject)
        guard let workspaceRoot = descriptor.workspaceRoot else {
            return descriptor
        }

        let workspaceURL = URL(fileURLWithPath: workspaceRoot, isDirectory: true)
        try FileManager.default.createDirectory(at: workspaceURL, withIntermediateDirectories: true)

        let baseRoot = workflowStatus?.workflow?.baseRoot ?? "."
        let sessionCwdURL: URL
        if baseRoot == "." {
            sessionCwdURL = workspaceURL
        } else {
            sessionCwdURL = workspaceURL.appendingPathComponent(baseRoot, isDirectory: true)
            try FileManager.default.createDirectory(at: sessionCwdURL, withIntermediateDirectories: true)
        }

        try runHookIfPresent(workflowStatus?.workflow?.hookRuns["after_create"], cwd: sessionCwdURL)
        try writeRenderedPrompt(
            workflowStatus: workflowStatus,
            project: selectedProject,
            sessionCwdURL: sessionCwdURL
        )
        try runHookIfPresent(workflowStatus?.workflow?.hookRuns["before_run"], cwd: sessionCwdURL)

        return SessionLaunchDescriptor(
            mode: descriptor.mode,
            title: descriptor.title,
            presetID: descriptor.presetID,
            restoreBundle: .genericShell(at: sessionCwdURL.path),
            workspaceRoot: workspaceURL.path,
            workflowSummary: descriptor.workflowSummary
        )
    }

    private func runHookIfPresent(_ hookPath: String?, cwd: URL) throws {
        guard let hookPath, !hookPath.isEmpty else {
            return
        }

        let process = Process()
        process.currentDirectoryURL = cwd
        process.executableURL = URL(fileURLWithPath: hookPath)
        try process.run()
        process.waitUntilExit()
        guard process.terminationStatus == 0 else {
            throw CoreBridgeError.operationFailed("workflow_hook_failed")
        }
    }

    private func writeRenderedPrompt(
        workflowStatus: WorkflowStatusPayload?,
        project: LauncherProject,
        sessionCwdURL: URL
    ) throws {
        let template = workflowStatus?.workflow?.templateBody ?? "Project: {{project.name}}"
        let rendered = template
            .replacingOccurrences(of: "{{project.name}}", with: project.name)
            .replacingOccurrences(of: "{{project.repo_root}}", with: project.rootPath)
            .replacingOccurrences(of: "{{workflow.name}}", with: workflowStatus?.workflow?.name ?? "")
        try rendered.write(
            to: sessionCwdURL.appendingPathComponent("prompt.rendered.md"),
            atomically: true,
            encoding: .utf8
        )
    }

    private func mergedSnapshot(local: AppShellSnapshot?, bridge: AppShellSnapshot) -> AppShellSnapshot {
        guard let local else {
            return bridge
        }

        return AppShellSnapshot(
            meta: bridge.meta,
            ops: bridge.sessions.isEmpty ? local.ops : bridge.ops,
            app: .init(
                activeRoute: selectedRoute,
                focusedSessionID: bridge.app.focusedSessionID ?? local.app.focusedSessionID,
                degradedFlags: Array(Set(local.app.degradedFlags + bridge.app.degradedFlags))
            ),
            projects: bridge.projects.isEmpty ? local.projects : bridge.projects,
            sessions: bridge.sessions.isEmpty ? local.sessions : bridge.sessions,
            attention: bridge.attention.isEmpty ? local.attention : bridge.attention,
            retryQueue: bridge.retryQueue.isEmpty ? local.retryQueue : bridge.retryQueue,
            warnings: local.warnings,
            workflow: bridge.workflow ?? local.workflow,
            tracker: bridge.tracker ?? local.tracker
        )
    }
}
