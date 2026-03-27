import AppKit
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
    @Published private(set) var isNotificationDrawerPresented = false
    @Published private(set) var pendingProjectFocusFilePath: String?
    @Published private(set) var isWorkflowDrawerPresented = false
    @Published private(set) var workflowStatus: WorkflowStatusPayload?
    @Published private(set) var settingsStatusViewModel: SettingsStatusViewModel?
    @Published private(set) var isTaskContextDrawerPresented = false
    @Published private(set) var taskContextDrawerModel: TaskDrawerModel?
    @Published private(set) var quickDispatchComposer: QuickDispatchComposerViewModel?
    @Published private(set) var quickDispatchOrigin: Route?
    @Published private(set) var quickDispatchTaskID: String?
    @Published private(set) var projectFocusRefreshToken = 0
    @Published private(set) var projectFocusTerminalFocusToken = 0
    @Published private(set) var isNewSessionSheetPresented = false
    @Published private(set) var newSessionSheetViewModel: NewSessionSheetViewModel?
    @Published private(set) var isCommandPalettePresented = false
    @Published private(set) var commandPaletteViewModel: CommandPaletteViewModel?
    @Published private(set) var transientNotice: String?
    @Published private(set) var isInventoryPresented = false
    @Published private(set) var inventoryViewModel: WorktreeInventoryViewModel?
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
    private var pendingQuickDispatchReplay: PendingQuickDispatchReplay?

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
        isNotificationDrawerPresented: Bool = false,
        pendingProjectFocusFilePath: String? = nil,
        isWorkflowDrawerPresented: Bool = false,
        workflowStatus: WorkflowStatusPayload? = nil,
        settingsStatusViewModel: SettingsStatusViewModel? = nil,
        isTaskContextDrawerPresented: Bool = false,
        taskContextDrawerModel: TaskDrawerModel? = nil,
        quickDispatchComposer: QuickDispatchComposerViewModel? = nil,
        quickDispatchOrigin: Route? = nil,
        quickDispatchTaskID: String? = nil,
        projectFocusRefreshToken: Int = 0,
        projectFocusTerminalFocusToken: Int = 0,
        isNewSessionSheetPresented: Bool = false,
        newSessionSheetViewModel: NewSessionSheetViewModel? = nil,
        isCommandPalettePresented: Bool = false,
        commandPaletteViewModel: CommandPaletteViewModel? = nil,
        transientNotice: String? = nil,
        isInventoryPresented: Bool = false,
        inventoryViewModel: WorktreeInventoryViewModel? = nil,
    ) {
        self.entrySurface = entrySurface
        self.selectedRoute = selectedRoute
        self.selectedProject = selectedProject
        self.recentProjects = recentProjects
        self.readinessReport = readinessReport
        self.shellSnapshot = shellSnapshot
        self.isNotificationDrawerPresented = isNotificationDrawerPresented
        self.pendingProjectFocusFilePath = pendingProjectFocusFilePath
        self.isWorkflowDrawerPresented = isWorkflowDrawerPresented
        self.workflowStatus = workflowStatus
        self.settingsStatusViewModel = settingsStatusViewModel
        self.isTaskContextDrawerPresented = isTaskContextDrawerPresented
        self.taskContextDrawerModel = taskContextDrawerModel
        self.quickDispatchComposer = quickDispatchComposer
        self.quickDispatchOrigin = quickDispatchOrigin
        self.quickDispatchTaskID = quickDispatchTaskID
        self.projectFocusRefreshToken = projectFocusRefreshToken
        self.projectFocusTerminalFocusToken = projectFocusTerminalFocusToken
        self.isNewSessionSheetPresented = isNewSessionSheetPresented
        self.newSessionSheetViewModel = newSessionSheetViewModel
        self.isCommandPalettePresented = isCommandPalettePresented
        self.commandPaletteViewModel = commandPaletteViewModel
        self.transientNotice = transientNotice
        self.isInventoryPresented = isInventoryPresented
        self.inventoryViewModel = inventoryViewModel
        self.projectStore = projectStore
        self.restoreStore = restoreStore
        self.preferencesStore = preferencesStore
        self
            .snapshotSource = snapshotSource ??
            LocalAppShellSnapshotSource(restoreStore: restoreStore)
        self.readinessRunner = readinessRunner
        self.projectFileIndex = projectFileIndex
        self.taskSearchProjectionStore = taskSearchProjectionStore
        if let inventorySearchProjectionStore {
            self.inventorySearchProjectionStore = inventorySearchProjectionStore
        } else if let coreBridge {
            self.inventorySearchProjectionStore = InventorySearchProjectionStore(
                inventoryList: { projectID in
                    try coreBridge.inventoryList(projectID)
                },
            )
        } else {
            self
                .inventorySearchProjectionStore =
                InventorySearchProjectionStore(restoreStore: restoreStore)
        }
        self
            .presetRegistry = presetRegistry ?? (try? PresetRegistry.loadDefault()) ??
            PresetRegistry(presets: [])
        self.coreBridge = coreBridge
    }

    static func bootstrap(
        projectStore: ProjectLauncherStore,
        restoreStore: TerminalSessionRestoreStore,
        preferencesStore: AppShellPreferencesStore,
        readinessRunner: ReadinessProbeRunner = .live,
        coreBridge: CoreBridge? = nil,
    ) throws -> AppShellModel {
        let selectedProject = try projectStore.loadLastSelectedProject()
        let recentProjects = try projectStore.loadRecentProjects()

        // Prefer Rust-backed app state over JSON preferences
        let activeRoute: Route
        if selectedProject != nil,
           let bridge = coreBridge,
           let appState = try? bridge.loadAppState(),
           let route = Route(rawValue: appState.activeRoute)
        {
            activeRoute = route
        } else {
            let preferences = try preferencesStore.load()
            activeRoute = selectedProject == nil ? .projectFocus : preferences.lastActiveRoute
        }

        return AppShellModel(
            entrySurface: selectedProject == nil ? .welcome(.firstRun) : .shell,
            selectedRoute: activeRoute,
            selectedProject: selectedProject,
            recentProjects: recentProjects,
            readinessReport: nil,
            projectStore: projectStore,
            restoreStore: restoreStore,
            preferencesStore: preferencesStore,
            readinessRunner: readinessRunner,
            coreBridge: coreBridge,
        )
    }

    static func liveDefault(
        projectStore: ProjectLauncherStore = .liveDefault,
        restoreStore: TerminalSessionRestoreStore = .liveDefault,
        preferencesStore: AppShellPreferencesStore = .liveDefault,
        readinessRunner: ReadinessProbeRunner = .live,
        coreBridge: CoreBridge? = nil,
    ) -> AppShellModel {
        let model = (try? bootstrap(
            projectStore: projectStore,
            restoreStore: restoreStore,
            preferencesStore: preferencesStore,
            readinessRunner: readinessRunner,
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
                coreBridge: coreBridge,
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
                isNotificationDrawerPresented: model.isNotificationDrawerPresented,
                pendingProjectFocusFilePath: model.pendingProjectFocusFilePath,
                isWorkflowDrawerPresented: model.isWorkflowDrawerPresented,
                workflowStatus: model.workflowStatus,
                settingsStatusViewModel: model.settingsStatusViewModel,
                isTaskContextDrawerPresented: model.isTaskContextDrawerPresented,
                taskContextDrawerModel: model.taskContextDrawerModel,
                quickDispatchComposer: model.quickDispatchComposer,
                quickDispatchOrigin: model.quickDispatchOrigin,
                quickDispatchTaskID: model.quickDispatchTaskID,
                projectFocusRefreshToken: model.projectFocusRefreshToken,
                projectFocusTerminalFocusToken: model.projectFocusTerminalFocusToken,
                isNewSessionSheetPresented: model.isNewSessionSheetPresented,
                newSessionSheetViewModel: model.newSessionSheetViewModel,
                isCommandPalettePresented: model.isCommandPalettePresented,
                commandPaletteViewModel: model.commandPaletteViewModel,
                transientNotice: model.transientNotice,
                isInventoryPresented: model.isInventoryPresented,
                inventoryViewModel: model.inventoryViewModel,
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
        coreBridge: CoreBridge? = nil,
        initialReport: ReadinessReport,
    ) async throws -> AppShellModel {
        let selectedProject = try projectStore.loadLastSelectedProject()
        let preferences = try preferencesStore.load()

        if selectedProject != nil, initialReport.requiresRecoverySurface {
            return try AppShellModel(
                entrySurface: .welcome(.degradedRecovery),
                selectedRoute: preferences.lastActiveRoute,
                selectedProject: selectedProject,
                recentProjects: projectStore.loadRecentProjects(),
                readinessReport: initialReport,
                projectStore: projectStore,
                restoreStore: restoreStore,
                preferencesStore: preferencesStore,
                readinessRunner: readinessRunner,
                coreBridge: coreBridge,
            )
        }

        return try bootstrap(
            projectStore: projectStore,
            restoreStore: restoreStore,
            preferencesStore: preferencesStore,
            readinessRunner: readinessRunner,
            coreBridge: coreBridge,
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
        persistCurrentRoute()
    }

    private func persistCurrentRoute() {
        guard let bridge = coreBridge else { return }
        let projectId = selectedProject?.projectID
        let sessionID = shellSnapshot?.app.focusedSessionID ?? taskContextDrawerModel?.sessionID
        try? bridge.saveAppState(selectedRoute.rawValue, projectId, sessionID)
    }

    func refreshShellSnapshot() async {
        let localSnapshot = try? await snapshotSource.load(
            activeRoute: selectedRoute,
            selectedProject: selectedProject,
            readinessReport: readinessReport,
            recentProjects: recentProjects,
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
        case .toggleNotificationDrawer:
            let wasPresented = isNotificationDrawerPresented
            isNotificationDrawerPresented.toggle()
            if wasPresented, !isNotificationDrawerPresented {
                requestProjectFocusTerminalRefocus()
            }
        case .dismissNotificationDrawer:
            isNotificationDrawerPresented = false
            requestProjectFocusTerminalRefocus()
        case .refreshShellSnapshot:
            break
        case .openSettings:
            if let selectedProject, let coreBridge,
               let payload = try? coreBridge.workflowValidate(selectedProject.rootPath)
            {
                workflowStatus = try? JSONDecoder().decode(
                    WorkflowStatusPayload.self,
                    from: payload,
                )
            }
            settingsStatusViewModel = makeSettingsStatusViewModel()
            setSelectedRoute(.settings)
        case .presentWorkflowDrawer:
            if let selectedProject, let coreBridge,
               let payload = try? coreBridge.workflowValidate(selectedProject.rootPath)
            {
                workflowStatus = try? JSONDecoder().decode(
                    WorkflowStatusPayload.self,
                    from: payload,
                )
            }
            isWorkflowDrawerPresented = true
        case .dismissWorkflowDrawer:
            isWorkflowDrawerPresented = false
            requestProjectFocusTerminalRefocus()
        case .presentTaskContextDrawer:
            taskContextDrawerModel = makeTaskContextDrawerModel()
            isTaskContextDrawerPresented = true
        case .dismissTaskContextDrawer:
            isTaskContextDrawerPresented = false
            taskContextDrawerModel = nil
            requestProjectFocusTerminalRefocus()
        case .reconcileAutomation:
            do {
                try coreBridge?.reconcileAutomation()
                transientNotice = "Reconcile requested."
            } catch {
                transientNotice = "Reconcile unavailable."
            }
        case .reloadWorkflow:
            guard let selectedProject, let coreBridge else {
                return
            }
            if let payload = try? coreBridge.workflowReload(selectedProject.rootPath) {
                workflowStatus = try? JSONDecoder().decode(
                    WorkflowStatusPayload.self,
                    from: payload,
                )
            }
            if selectedRoute == .settings {
                settingsStatusViewModel = makeSettingsStatusViewModel()
            }
            if isTaskContextDrawerPresented {
                taskContextDrawerModel = makeTaskContextDrawerModel()
            }
        case let .resolveAttention(attentionID):
            try? coreBridge?.resolveAttention(attentionID)
            transientNotice = "Resolved attention: \(attentionID)"
        case let .dismissAttention(attentionID):
            try? coreBridge?.dismissAttention(attentionID)
            transientNotice = "Dismissed attention: \(attentionID)"
        case let .snoozeAttention(attentionID):
            try? coreBridge?.snoozeAttention(attentionID)
            transientNotice = "Snoozed attention: \(attentionID)"
        case let .presentQuickDispatch(origin):
            if let snapshot = shellSnapshot {
                quickDispatchComposer = QuickDispatchComposerViewModel(
                    snapshot: snapshot,
                    origin: origin,
                )
                quickDispatchOrigin = origin
                quickDispatchTaskID = resolveQuickDispatchTaskID(origin: origin, snapshot: snapshot)
            }
        case .dismissQuickDispatch:
            quickDispatchComposer = nil
            quickDispatchOrigin = nil
            quickDispatchTaskID = nil
            requestProjectFocusTerminalRefocus()
        case let .submitQuickDispatch(targetID, message):
            if let adapterKind = targetID.split(separator: ":", maxSplits: 1).dropFirst().first {
                let taskID = quickDispatchTaskID
                quickDispatchComposer = nil
                quickDispatchOrigin = nil
                quickDispatchTaskID = nil
                requestProjectFocusTerminalRefocus()
                pendingQuickDispatchReplay = PendingQuickDispatchReplay(
                    taskID: taskID,
                    message: message,
                )
                presentNewSessionSheet(prefillPresetID: presetID(for: String(adapterKind)))
                transientNotice = "Open a new \(adapterKind) session, then continue quick dispatch."
            } else {
                pendingQuickDispatchReplay = nil
                try? coreBridge?.dispatchSend(targetID, quickDispatchTaskID, message)
                transientNotice = "Dispatch sent to \(targetID)"
                quickDispatchComposer = nil
                quickDispatchOrigin = nil
                quickDispatchTaskID = nil
                requestProjectFocusTerminalRefocus()
            }
        case let .terminalSessionReady(sessionID):
            if let pendingQuickDispatchReplay {
                try? coreBridge?.dispatchSend(
                    sessionID,
                    pendingQuickDispatchReplay.taskID,
                    pendingQuickDispatchReplay.message,
                )
                self.pendingQuickDispatchReplay = nil
                transientNotice = "Dispatch sent to \(sessionID)"
            }
        case let .dispatchSend(targetSessionID, taskID, message):
            pendingQuickDispatchReplay = nil
            try? coreBridge?.dispatchSend(targetSessionID, taskID ?? quickDispatchTaskID, message)
            transientNotice = "Dispatch sent to \(targetSessionID)"
            quickDispatchComposer = nil
            quickDispatchOrigin = nil
            quickDispatchTaskID = nil
            requestProjectFocusTerminalRefocus()
        case .exportSnapshot:
            if let coreBridge, let payload = try? coreBridge.stateSnapshotJSON() {
                let exportURL = snapshotExportURL()
                try? FileManager.default.createDirectory(
                    at: exportURL.deletingLastPathComponent(),
                    withIntermediateDirectories: true,
                )
                try? payload.write(to: exportURL, atomically: true, encoding: .utf8)
                transientNotice = "Snapshot exported to \(exportURL.lastPathComponent)"
            } else {
                transientNotice = "Snapshot export unavailable."
            }
        case .presentNewSessionSheet:
            presentNewSessionSheet(prefillPresetID: nil)
        case .dismissNewSessionSheet:
            isNewSessionSheetPresented = false
            newSessionSheetViewModel = nil
            pendingQuickDispatchReplay = nil
            requestProjectFocusTerminalRefocus()
        case let .launchSession(descriptor):
            do {
                let bootstrappedDescriptor = try bootstrapIfNeeded(descriptor)
                var bundles = try restoreStore.load()
                bundles.removeAll(where: { $0 == bootstrappedDescriptor.restoreBundle })
                bundles.insert(bootstrappedDescriptor.restoreBundle, at: 0)
                try restoreStore.save(bundles)
                projectFocusRefreshToken &+= 1
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
                requestProjectFocusTerminalRefocus()
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
            requestProjectFocusTerminalRefocus()
        case let .queueFileSelection(path):
            setSelectedRoute(.projectFocus)
            pendingProjectFocusFilePath = path
            transientNotice = "File queued for Project Focus: \(path)"
        case let .createTaskDraft(title):
            do {
                let row = try taskSearchProjectionStore.createDraft(
                    title,
                    selectedProject?.projectID,
                )
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
                if let sessionID = latestAttention.targetSessionID {
                    setSelectedRoute(.projectFocus)
                    if let coreBridge {
                        do {
                            try coreBridge.focusSession(sessionID)
                        } catch {
                            transientNotice = "Focus requested for session \(sessionID)"
                        }
                    }
                } else {
                    setSelectedRoute(latestAttention.targetRoute)
                }
                transientNotice = "Jumped to latest unread: \(latestAttention.headline)"
            }
        case .retryReadiness:
            await retryStartupReadiness()
        case let .triggerRecovery(issueCode):
            await handleRecovery(issueCode: issueCode)
        case .presentInventory:
            isInventoryPresented = true
            inventoryViewModel = await loadInventoryViewModel()
        case .dismissInventory:
            isInventoryPresented = false
            requestProjectFocusTerminalRefocus()
        case let .openInventoryFinder(path):
            NSWorkspace.shared.selectFile(nil, inFileViewerRootedAtPath: path)
        case let .openInventorySession(taskID, worktreeId):
            guard !taskID.isEmpty || !worktreeId.isEmpty else {
                transientNotice = "Session not available for this worktree."
                return
            }
            if let sessionID = shellSnapshot?.sessions.first(where: { $0.taskID == taskID })?
                .sessionID
            {
                await perform(.jumpToSession(sessionID))
            } else {
                transientNotice = "Session not available for task \(taskID)."
            }
        case let .openInventoryTask(taskID):
            guard !taskID.isEmpty else {
                transientNotice = "Task not available for this worktree."
                return
            }
            taskContextDrawerModel = makeTaskContextDrawerModel(targetTaskID: taskID)
            isTaskContextDrawerPresented = taskContextDrawerModel != nil
            transientNotice = taskContextDrawerModel == nil
                ? "Task context unavailable for \(taskID)."
                : "Opened task \(taskID)"
        }

        if action != .dismissCommandPalette, action != .toggleCommandPalette {
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

    func startLocalControlServerIfNeeded() async {
        do {
            try coreBridge?.startLocalControlServer()
        } catch {
            transientNotice = localControlServerStartupNotice(for: error)
        }
    }

    func presentShell() {
        entrySurface = .shell
    }

    func refreshCommandPaletteViewModel() async {
        let snapshot = shellSnapshot ?? AppShellSnapshot.empty(
            activeRoute: selectedRoute,
            selectedProject: selectedProject,
        )
        let files: [ProjectFileIndex.Entry] = if let rootPath = selectedProject?.rootPath {
            await (try? projectFileIndex.index(rootPath: rootPath)) ?? []
        } else {
            []
        }

        let tasks = (try? taskSearchProjectionStore.search("")) ?? []
        let inventory = await (try? inventorySearchProjectionStore.load(
            selectedProjectID: selectedProject?.projectID,
            selectedProjectRoot: selectedProject?.rootPath,
        )) ?? []

        commandPaletteViewModel = CommandPaletteViewModel(
            catalog: .build(
                snapshot: snapshot,
                files: files,
                tasks: tasks,
                inventory: inventory,
            ),
        )
    }

    private func loadInventoryViewModel() async -> WorktreeInventoryViewModel? {
        guard let coreBridge, let selectedProject else {
            return WorktreeInventoryViewModel(rows: [])
        }
        guard let payloads = try? coreBridge.inventoryList(selectedProject.projectID) else {
            return WorktreeInventoryViewModel(rows: [])
        }
        let rows = payloads.compactMap { payload -> WorktreeInventoryViewModel.Row? in
            guard let disposition = WorktreeInventoryViewModel
                .Disposition(rawValue: payload.disposition)
            else {
                return nil
            }
            return WorktreeInventoryViewModel.Row(
                worktreeId: payload.worktreeId,
                taskID: payload.taskId,
                path: payload.path,
                projectName: payload.projectName.isEmpty ? selectedProject.name : payload
                    .projectName,
                branch: payload.branch,
                disposition: disposition,
                isPinned: payload.isPinned,
                isDegraded: payload.isDegraded,
                sizeBytes: payload.sizeBytes,
                lastAccessedAt: payload.lastAccessedAt,
            )
        }
        return WorktreeInventoryViewModel(rows: rows)
    }

    private func retryStartupReadiness() async {
        guard let selectedProject else {
            return
        }

        let report = try? await readinessRunner.run(for: selectedProject)
        updateReadinessReport(report)
    }

    private func requestProjectFocusTerminalRefocus() {
        guard entrySurface == .shell, selectedRoute == .projectFocus else {
            return
        }

        projectFocusTerminalFocusToken &+= 1
    }

    private func fetchWorkflowStatus(for project: LauncherProject) -> WorkflowStatusPayload? {
        guard let coreBridge,
              let payload = try? coreBridge.workflowValidate(project.rootPath)
        else {
            return nil
        }

        guard let decoded = try? JSONDecoder().decode(WorkflowStatusPayload.self, from: payload)
        else {
            return nil
        }

        if workflowStatus?.path == decoded.path {
            return WorkflowStatusPayload(
                state: decoded.state,
                path: decoded.path,
                lastGoodHash: decoded.lastGoodHash,
                lastReloadAt: decoded.lastReloadAt,
                lastError: decoded.lastError,
                lastBootstrap: workflowStatus?.lastBootstrap,
                workflow: decoded.workflow,
            )
        }

        return decoded
    }

    private func makeSettingsStatusViewModel() -> SettingsStatusViewModel {
        let termSettings = try? coreBridge?.terminalSettings()
        let runtimeSummary = try? coreBridge?.runtimeInfoSummary()
        let context = RecoveryContextPayload(
            workflowHealth: shellSnapshot?.ops.workflowHealth.rawValue ?? "unknown",
            staleClaims: [],
        )
        let degradedIssues = (try? coreBridge?.listDegradedIssues(context)) ?? []
        return SettingsStatusViewModel(
            report: readinessReport,
            workflowStatus: workflowStatus,
            presetRegistry: presetRegistry,
            runtimeInfo: try? coreBridge?.runtimeInfo(),
            snapshot: shellSnapshot,
            terminalSettings: termSettings ?? nil,
            runtimeInfoSummary: runtimeSummary,
            degradedIssues: degradedIssues,
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
            workflow: summary,
        )

        if selectedRoute == .settings {
            settingsStatusViewModel = makeSettingsStatusViewModel()
        }
        if isTaskContextDrawerPresented {
            taskContextDrawerModel = makeTaskContextDrawerModel()
        }
    }

    private func makeTaskContextDrawerModel(targetTaskID: String? = nil) -> TaskDrawerModel? {
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
                workflow: resolved.workflow ?? fetchedWorkflowStatus?.workflow,
            )
        }

        return TaskDrawerModel.resolve(
            from: snapshot,
            workflowStatus: mergedWorkflowStatus,
            targetTaskID: targetTaskID,
        )
    }

    func recoverableSessions() -> [RecoverableSessionPayload] {
        guard let coreBridge, let selectedProject else {
            return []
        }
        return (try? coreBridge.listRecoverableSessions(selectedProject.projectID)) ?? []
    }

    private func inventoryRow(worktreeID: String) -> WorktreeInventoryViewModel.Row? {
        inventoryViewModel?.inUseRows.first(where: { $0.worktreeId == worktreeID })
            ?? inventoryViewModel?.recoverableRows.first(where: { $0.worktreeId == worktreeID })
            ?? inventoryViewModel?.safeToDeleteRows.first(where: { $0.worktreeId == worktreeID })
            ?? inventoryViewModel?.staleRows.first(where: { $0.worktreeId == worktreeID })
    }

    private func handleRecovery(issueCode: String) async {
        guard let coreBridge else {
            transientNotice = "Recovery unavailable: \(issueCode)"
            return
        }

        if issueCode.hasPrefix("pin:") {
            let parts = issueCode.split(separator: ":", omittingEmptySubsequences: false)
            guard parts.count >= 3 else {
                transientNotice = "Pin request was invalid."
                return
            }
            let worktreeID = String(parts[1])
            let desiredPinned = String(parts[2]).lowercased() == "true"
            do {
                try coreBridge.setWorktreePinned(worktreeID, desiredPinned)
                transientNotice = desiredPinned ? "Pinned \(worktreeID)" : "Unpinned \(worktreeID)"
                inventoryViewModel = await loadInventoryViewModel()
            } catch {
                transientNotice = "Pin request failed for \(worktreeID)."
            }
            return
        }

        if issueCode.hasPrefix("clean:") {
            let worktreeID = String(issueCode.dropFirst("clean:".count))
            let nextState = switch inventoryRow(worktreeID: worktreeID)?.disposition {
            case .safeToDelete:
                "stale"
            case .stale:
                "stale"
            default:
                "safe_to_delete"
            }
            do {
                try coreBridge.updateWorktreeLifecycle(worktreeID, nextState)
                transientNotice = "Cleanup staged for \(worktreeID)"
                inventoryViewModel = await loadInventoryViewModel()
            } catch {
                transientNotice = "Cleanup request failed for \(worktreeID)."
            }
            return
        }

        if issueCode.hasPrefix("recover:") {
            let worktreeID = String(issueCode.dropFirst("recover:".count))
            if let row = inventoryRow(worktreeID: worktreeID), !row.taskID.isEmpty {
                await perform(.openInventoryTask(taskID: row.taskID))
            } else {
                transientNotice = "Recovery context unavailable for \(worktreeID)."
            }
            return
        }

        switch issueCode {
        case "invalid_workflow_reload":
            await perform(.reloadWorkflow)
            setSelectedRoute(.settings)
            transientNotice = "Workflow reload requested."
        case "stale_claim_reconcile":
            await perform(.reconcileAutomation)
        case "keychain_ref_missing", "preset_missing":
            await perform(.openSettings)
            transientNotice = "Open Settings to resolve \(issueCode)."
        case "missing_project_path", "crashed_restore":
            entrySurface = .welcome(.degradedRecovery)
            transientNotice = "Recovery launcher opened for \(issueCode)."
        case "worktree_unreachable":
            await perform(.presentInventory)
            transientNotice = "Inventory opened for worktree recovery."
        default:
            transientNotice = "Recovery requested for issue: \(issueCode)"
        }
    }

    private func presentNewSessionSheet(prefillPresetID: String?) {
        let resolvedWorkflowSummary = selectedProject.flatMap { project in
            fetchWorkflowStatus(for: project)?.workflow.map {
                WorkflowLaunchSummary(
                    name: $0.name ?? project.name,
                    strategy: $0.strategy ?? "worktree",
                    baseRoot: $0.baseRoot ?? ".",
                    reviewChecklist: $0.reviewChecklist,
                    allowedAgents: $0.allowedAgents,
                )
            }
        }
        newSessionSheetViewModel = NewSessionSheetViewModel(
            selectedProjectRoot: selectedProject?.rootPath,
            selectedTaskID: taskContextDrawerModel?.taskID,
            registry: presetRegistry,
            workflowSummary: resolvedWorkflowSummary,
            preferredPresetID: prefillPresetID,
            provisionIsolatedWorkspace: { [coreBridge] projectRoot, taskID in
                guard let coreBridge else {
                    throw NewSessionSheetError.isolatedProvisionUnavailable
                }
                return try coreBridge.provisionTaskWorkspace(
                    projectRoot,
                    taskID,
                    resolvedWorkflowSummary?.baseRoot,
                )
            },
            resolveSecretEnv: { [coreBridge] in
                // TODO(sprint-6): pass project_id as scope_filter once project context is threaded through launch
                try coreBridge?.resolveLaunchEnvironment() ?? [:]
            },
        )
        isNewSessionSheetPresented = true
    }

    private func resolveQuickDispatchTaskID(origin: Route, snapshot: AppShellSnapshot) -> String? {
        switch origin {
        case .projectFocus, .taskBoard:
            if let taskID = taskContextDrawerModel?.taskID {
                return taskID
            }
            return focusedSession(from: snapshot)?.taskID
        case .controlTower, .reviewQueue, .attentionCenter, .settings:
            return nil
        }
    }

    private func focusedSession(from snapshot: AppShellSnapshot) -> AppShellSnapshot
        .SessionSummary?
    {
        snapshot.sessions.first(where: {
            $0.focusState == .focused || snapshot.app.focusedSessionID == $0.sessionID
        })
    }

    private func localControlServerStartupNotice(for error: Error) -> String {
        guard case let CoreBridgeError.operationFailed(message) = error else {
            return "Local control API unavailable."
        }

        if message.hasPrefix("socket_already_owned:") {
            let socketPath = String(message.dropFirst("socket_already_owned:".count))
            return "Local control API already owned by another Haneulchi instance at \(socketPath)."
        }

        if message == "socket_already_owned" {
            return "Local control API already owned by another Haneulchi instance."
        }

        return "Local control API unavailable: \(message)"
    }

    private func presetID(for adapterKind: String) -> String? {
        let normalized = adapterKind.lowercased()
        if normalized.contains("claude") {
            return presetRegistry.preset(id: "claude") != nil ? "claude" : nil
        }
        if normalized.contains("codex") {
            return presetRegistry.preset(id: "codex") != nil ? "codex" : nil
        }
        if normalized.contains("gemini") {
            return presetRegistry.preset(id: "gemini") != nil ? "gemini" : nil
        }
        return presetRegistry.presets.first(where: { preset in
            normalized.contains(preset.id.lowercased()) || preset.id.lowercased()
                .contains(normalized)
        })?.id
    }

    private struct PendingQuickDispatchReplay {
        let taskID: String?
        let message: String
    }

    private func bootstrapIfNeeded(_ descriptor: SessionLaunchDescriptor) throws
        -> SessionLaunchDescriptor
    {
        guard descriptor.mode == .isolated, let selectedProject, let coreBridge else {
            return descriptor
        }
        guard let workspaceRoot = descriptor.workspaceRoot else {
            return descriptor
        }
        let taskID = URL(fileURLWithPath: workspaceRoot, isDirectory: true).lastPathComponent
        let bootstrap = try coreBridge.prepareIsolatedLaunch(
            selectedProject.rootPath,
            selectedProject.name,
            taskID,
            descriptor.title,
            workspaceRoot,
        )
        if let current = workflowStatus {
            workflowStatus = WorkflowStatusPayload(
                state: current.state,
                path: current.path,
                lastGoodHash: current.lastGoodHash,
                lastReloadAt: current.lastReloadAt,
                lastError: current.lastError,
                lastBootstrap: bootstrap,
                workflow: current.workflow,
            )
        } else {
            workflowStatus = WorkflowStatusPayload(
                state: .none,
                path: selectedProject.rootPath + "/WORKFLOW.md",
                lastGoodHash: bootstrap.lastKnownGoodHash,
                lastReloadAt: nil,
                lastError: nil,
                lastBootstrap: bootstrap,
                workflow: nil,
            )
        }
        let sessionCwdURL = URL(fileURLWithPath: bootstrap.sessionCwd, isDirectory: true)

        return SessionLaunchDescriptor(
            mode: descriptor.mode,
            title: descriptor.title,
            presetID: descriptor.presetID,
            restoreBundle: .genericShell(at: sessionCwdURL.path),
            workspaceRoot: workspaceRoot,
            workflowSummary: descriptor.workflowSummary,
        )
    }

    private func mergedSnapshot(local: AppShellSnapshot?,
                                bridge: AppShellSnapshot) -> AppShellSnapshot
    {
        guard let local else {
            return bridge
        }

        return AppShellSnapshot(
            meta: bridge.meta,
            ops: bridge.ops,
            app: .init(
                activeRoute: selectedRoute,
                focusedSessionID: bridge.app.focusedSessionID ?? local.app.focusedSessionID,
                degradedFlags: Array(Set(local.app.degradedFlags + bridge.app.degradedFlags)),
            ),
            projects: bridge.projects.isEmpty ? local.projects : bridge.projects,
            sessions: bridge.sessions.isEmpty ? local.sessions : bridge.sessions,
            attention: bridge.attention.isEmpty ? local.attention : bridge.attention,
            retryQueue: bridge.retryQueue.isEmpty ? local.retryQueue : bridge.retryQueue,
            warnings: local.warnings,
            workflow: bridge.workflow ?? local.workflow,
            tracker: bridge.tracker ?? local.tracker,
        )
    }

    private func snapshotExportURL() -> URL {
        if let override = ProcessInfo.processInfo.environment["HC_EXPORT_SNAPSHOT_PATH"] {
            return URL(fileURLWithPath: override)
        }

        return FileManager.default.homeDirectoryForCurrentUser
            .appendingPathComponent(
                "Library/Application Support/Haneulchi/evidence",
                isDirectory: true,
            )
            .appendingPathComponent("exported-snapshot.json")
    }
}
