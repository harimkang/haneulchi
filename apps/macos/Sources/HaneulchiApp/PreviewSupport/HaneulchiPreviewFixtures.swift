import Foundation

enum HaneulchiPreviewFixtures {
    static let project = LauncherProject(
        projectID: "proj_demo_workspace",
        name: "Demo Workspace",
        rootPath: "/tmp/haneulchi-preview",
        lastOpenedAt: .now,
    )

    static let recentProjects: [LauncherProject] = [
        project,
        LauncherProject(
            projectID: "proj_auth_service",
            name: "Auth Service",
            rootPath: "/tmp/auth-service",
            lastOpenedAt: .now.addingTimeInterval(-7200),
        ),
    ]

    static let workflowStatus = WorkflowStatusPayload(
        state: .ok,
        path: "/tmp/haneulchi-preview/WORKFLOW.md",
        lastGoodHash: "sha256:preview-last-good",
        lastReloadAt: "2026-03-26T07:15:00Z",
        lastError: nil,
        lastBootstrap: .init(
            workspaceRoot: "/tmp/haneulchi-preview/.haneulchi/task_104",
            baseRoot: ".",
            sessionCwd: "/tmp/haneulchi-preview/.haneulchi/task_104",
            renderedPromptPath: "/tmp/haneulchi-preview/.haneulchi/prompts/task_104.md",
            phaseSequence: ["resolve", "hydrate", "launch"],
            hookPhaseResults: [
                .init(
                    phase: "hydrate",
                    commandPath: "/usr/bin/env",
                    exitCode: 0,
                    stdout: "ok",
                    stderr: "",
                    succeeded: true,
                ),
            ],
            outcomeCode: "launched",
            warningCodes: [],
            claimReleased: false,
            launchExitCode: 0,
            lastKnownGoodHash: "sha256:preview-last-good",
        ),
        workflow: .init(
            name: "Demo Workflow",
            strategy: "worktree",
            baseRoot: ".",
            requireReview: true,
            maxRuntimeMinutes: 45,
            unsafeOverridePolicy: nil,
            reviewChecklist: ["Run tests", "Inspect diff", "Summarize outcome"],
            allowedAgents: ["codex", "automation-bot"],
            hooks: ["hydrate", "launch"],
            hookRuns: ["hydrate": "ok", "launch": "ok"],
            templateBody: "Focus on the requested task and leave reviewable outcomes.",
        ),
    )

    static let readinessReport = ReadinessReport(
        project: project,
        checks: [
            .init(
                name: .shell,
                status: .ready,
                headline: "Shell available",
                detail: "Login shell resolved successfully.",
                nextAction: nil,
            ),
            .init(
                name: .git,
                status: .ready,
                headline: "Git ready",
                detail: "Repository state is readable.",
                nextAction: nil,
            ),
            .init(
                name: .shellIntegration,
                status: .degraded,
                headline: "Shell integration not installed",
                detail: "Install shell integration to improve live commentary and path hydration.",
                nextAction: "Open Settings",
            ),
            .init(
                name: .presetBinaries,
                status: .ready,
                headline: "Preset binaries available",
                detail: "Codex and generic shell presets are installed.",
                nextAction: nil,
            ),
            .init(
                name: .workflow,
                status: .ready,
                headline: "Workflow contract loaded",
                detail: "Preview workflow is available for launches.",
                nextAction: nil,
            ),
        ],
    )

    static func snapshot(activeRoute: Route = .projectFocus) -> AppShellSnapshot {
        AppShellSnapshot(
            meta: .init(snapshotRev: 42, runtimeRev: 42, projectionRev: 42, snapshotAt: .now),
            ops: .init(
                cadenceMs: 15000,
                lastTickAt: "2026-03-26T07:15:00Z",
                lastReconcileAt: "2026-03-26T07:14:00Z",
                runningSlots: 2,
                maxSlots: 4,
                retryQueueCount: 1,
                queuedClaimCount: 1,
                workflowHealth: .ok,
                trackerHealth: "sync_degraded",
                paused: false,
            ),
            app: .init(
                activeRoute: activeRoute,
                focusedSessionID: "ses_review",
                degradedFlags: [.degraded, .unread],
            ),
            projects: [
                .init(
                    projectID: project.projectID,
                    name: project.name,
                    rootPath: project.rootPath,
                    status: .active,
                    workflowState: .ok,
                    sessionCount: 3,
                    attentionCount: 2,
                    taskCounts: ["running": 1, "review": 1, "blocked": 1],
                ),
                .init(
                    projectID: "proj_auth_service",
                    name: "Auth Service",
                    rootPath: "/tmp/auth-service",
                    status: .idle,
                    workflowState: .invalidKeptLastGood,
                    sessionCount: 1,
                    attentionCount: 1,
                    taskCounts: ["ready": 1],
                ),
            ],
            sessions: [
                .init(
                    sessionID: "ses_build",
                    title: "Build Session",
                    currentDirectory: project.rootPath,
                    mode: .generic,
                    runtimeState: .running,
                    manualControlState: .none,
                    dispatchState: .dispatchable,
                    unreadCount: 1,
                    projectID: project.projectID,
                    taskID: "task_101",
                    automationMode: .assisted,
                    trackerBindingState: "tracker_bound",
                    workspaceRoot: "\(project.rootPath)/.haneulchi/task_101",
                    baseRoot: ".",
                    branch: "feature/build",
                    latestSummary: "Building macOS target",
                    latestCommentary: "Compiled terminal renderer and refreshed bridge bindings.",
                    commentaryUpdatedAt: "2026-03-26T07:10:00Z",
                    claimState: .none,
                    adapterKind: nil,
                    lastActivityAt: "2026-03-26T07:10:00Z",
                    focusState: .background,
                    canTakeover: false,
                ),
                .init(
                    sessionID: "ses_review",
                    title: "Review Session",
                    currentDirectory: project.rootPath,
                    mode: .structuredAdapter,
                    runtimeState: .reviewReady,
                    manualControlState: .takeover,
                    dispatchState: .dispatchable,
                    unreadCount: 3,
                    projectID: project.projectID,
                    taskID: "task_104",
                    automationMode: .autoEligible,
                    trackerBindingState: "tracker_sync_degraded",
                    workspaceRoot: "\(project.rootPath)/.haneulchi/task_104",
                    baseRoot: ".",
                    branch: "feature/review",
                    latestSummary: "Review evidence ready",
                    providerID: "openai",
                    modelID: "gpt-5.4",
                    latestCommentary: "Waiting for operator review on the generated summary.",
                    commentaryUpdatedAt: "2026-03-26T07:11:00Z",
                    claimState: .claimed,
                    adapterKind: "codex",
                    lastActivityAt: "2026-03-26T07:11:00Z",
                    focusState: .focused,
                    canTakeover: true,
                ),
                .init(
                    sessionID: "ses_blocked",
                    title: "Blocked Session",
                    currentDirectory: "/tmp/auth-service",
                    mode: .structuredAdapter,
                    runtimeState: .blocked,
                    manualControlState: .none,
                    dispatchState: .dispatchFailed,
                    unreadCount: 0,
                    projectID: "proj_auth_service",
                    taskID: "task_201",
                    automationMode: .manual,
                    trackerBindingState: "tracker_unbound",
                    workspaceRoot: "/tmp/auth-service/.haneulchi/task_201",
                    baseRoot: ".",
                    branch: "hotfix/auth",
                    latestSummary: "Dispatch target stale",
                    dispatchReason: "Adapter unavailable",
                    latestCommentary: "Manual intervention required before dispatch can resume.",
                    commentaryUpdatedAt: "2026-03-26T07:08:00Z",
                    claimState: .stale,
                    adapterKind: "codex",
                    lastActivityAt: "2026-03-26T07:08:00Z",
                    focusState: .background,
                    canTakeover: true,
                ),
            ],
            attention: [
                .init(
                    attentionID: "att_shell_integration",
                    headline: "Shell integration not installed",
                    severity: .degraded,
                    targetRoute: .attentionCenter,
                    targetSessionID: nil,
                    projectID: project.projectID,
                    taskID: "task_104",
                    summary: "Install shell integration to improve commentary and dispatch context.",
                    createdAt: "2026-03-26T07:09:00Z",
                    actionHint: "Open Settings",
                ),
                .init(
                    attentionID: "att_review",
                    headline: "Review ready evidence available",
                    severity: .unread,
                    targetRoute: .reviewQueue,
                    targetSessionID: "ses_review",
                    projectID: project.projectID,
                    taskID: "task_104",
                    summary: "Open the review queue to inspect the evidence pack.",
                    createdAt: "2026-03-26T07:11:00Z",
                    actionHint: "Open Review Queue",
                ),
            ],
            retryQueue: [
                .init(
                    taskID: "task_201",
                    projectID: "proj_auth_service",
                    attempt: 2,
                    reasonCode: "dispatch_failed",
                    dueAt: "2026-03-26T07:20:00Z",
                    backoffMs: 60000,
                    claimState: .stale,
                ),
            ],
            warnings: [],
            recentArtifacts: [
                .init(
                    taskID: "task_104",
                    projectID: project.projectID,
                    summary: "Evidence pack updated with diff, tests, and commentary summary.",
                    jumpTarget: "review_queue",
                    manifestPath: "\(project.rootPath)/.haneulchi/evidence/task_104.json",
                ),
            ],
        )
    }

    static func settingsViewModel(snapshot: AppShellSnapshot? = nil) -> SettingsStatusViewModel {
        SettingsStatusViewModel(
            report: readinessReport,
            workflowStatus: workflowStatus,
            presetRegistry: previewPresetRegistry,
            runtimeInfo: .init(rendererID: "swiftterm", transport: "ffi", demoMode: true),
            snapshot: snapshot ?? self.snapshot(),
            terminalSettings: .init(
                shell: "/bin/zsh",
                defaultCols: 120,
                defaultRows: 36,
                scrollbackLines: 20000,
                fontName: "SF Mono",
                theme: "dark",
                cursorStyle: "block",
            ),
            runtimeInfoSummary: .init(
                socketPath: "/tmp/haneulchi-preview.sock",
                transport: "ffi",
                status: "available",
            ),
            degradedIssues: [
                .init(
                    issueCode: "shell_integration_missing",
                    details: "Install shell integration to unlock richer session metadata.",
                ),
            ],
        )
    }

    static func taskBoardProjection() -> TaskBoardProjectionPayload {
        TaskBoardProjectionPayload(
            selectedProjectID: project.projectID,
            projects: [
                .init(projectID: project.projectID, title: project.name, taskCount: 4),
                .init(projectID: "proj_auth_service", title: "Auth Service", taskCount: 2),
            ],
            columns: [
                .init(column: .inbox, tasks: [
                    .init(
                        id: "task_101",
                        projectID: project.projectID,
                        displayKey: "#101",
                        title: "Document preview support rollout",
                        description: "Outline preview coverage for route surfaces and overlays.",
                        priority: "high",
                        automationMode: .manual,
                        linkedSessionID: nil,
                        column: .inbox,
                    ),
                ]),
                .init(column: .ready, tasks: [
                    .init(
                        id: "task_102",
                        projectID: project.projectID,
                        displayKey: "#102",
                        title: "Wire preview fixtures into route views",
                        description: "Build shared preview snapshot and settings fixtures.",
                        priority: "medium",
                        automationMode: .assisted,
                        linkedSessionID: "ses_build",
                        column: .ready,
                    ),
                ]),
                .init(column: .running, tasks: [
                    .init(
                        id: "task_103",
                        projectID: project.projectID,
                        displayKey: "#103",
                        title: "Verify canvas loads for major routes",
                        description: "Confirm control tower, task board, and review previews render.",
                        priority: "high",
                        automationMode: .autoEligible,
                        linkedSessionID: "ses_build",
                        column: .running,
                    ),
                ]),
                .init(column: .review, tasks: [
                    .init(
                        id: "task_104",
                        projectID: project.projectID,
                        displayKey: "#104",
                        title: "Review preview gallery coverage",
                        description: "Check that all preview macros show meaningful data.",
                        priority: "high",
                        automationMode: .autoEligible,
                        linkedSessionID: "ses_review",
                        column: .review,
                    ),
                ]),
                .init(column: .blocked, tasks: [
                    .init(
                        id: "task_201",
                        projectID: "proj_auth_service",
                        displayKey: "#201",
                        title: "Recover stale dispatch target",
                        description: "Adapter target is unavailable until manual recovery completes.",
                        priority: "high",
                        automationMode: .manual,
                        linkedSessionID: "ses_blocked",
                        column: .blocked,
                    ),
                ]),
                .init(column: .done, tasks: []),
            ],
        )
    }

    @MainActor
    static func taskBoardViewModel() -> TaskBoardViewModel {
        let projection = taskBoardProjection()
        return TaskBoardViewModel(
            loadProjection: { _ in projection },
            moveTask: { _, _ in projection },
        )
    }

    static func reviewQueueProjection() -> ReviewQueueProjectionPayload {
        ReviewQueueProjectionPayload(
            items: [
                .init(
                    taskID: "task_104",
                    projectID: project.projectID,
                    title: "Review preview gallery coverage",
                    summary: "Evidence pack is ready for operator validation.",
                    touchedFiles: [
                        "Sources/HaneulchiApp/PreviewSupport/HaneulchiPreviewFixtures.swift",
                        "Sources/HaneulchiApp/PreviewSupport/HaneulchiPreviewGallery.swift",
                    ],
                    diffSummary: "Added preview fixtures and route-specific previews.",
                    testsSummary: "swift test HaneulchiPreviewFixturesTests",
                    commandSummary: "Preview macros compile for all target views.",
                    hookSummary: "Shell integration missing on preview host.",
                    evidenceSummary: "Preview gallery now covers route views, overlays, and sheets.",
                    checklistSummary: "Verify all previews open in canvas.",
                    warnings: ["Shell integration missing on preview host."],
                    evidenceManifestPath: "\(project.rootPath)/.haneulchi/evidence/task_104.json",
                    ciRunURL: "https://ci.example.com/runs/preview-104",
                    prURL: "https://example.com/pr/104",
                    timeline: previewTimeline,
                ),
            ],
            degradedReason: "Shell integration not installed on this preview host.",
        )
    }

    @MainActor
    static func reviewQueueViewModel() -> ReviewQueueViewModel {
        let projection = reviewQueueProjection()
        return ReviewQueueViewModel(
            loadProjection: { projection },
            applyDecision: { _, _ in },
        )
    }

    static func attentionViewModel(snapshot: AppShellSnapshot? = nil) -> AttentionCenterViewModel {
        let snapshot = snapshot ?? self.snapshot(activeRoute: .attentionCenter)
        return AttentionCenterViewModel(
            snapshot: snapshot,
            openTarget: { _ in },
            resolveAttention: { _ in },
            dismissAttention: { _ in },
            snoozeAttention: { _ in },
        )
    }

    static func inventoryViewModel() -> WorktreeInventoryViewModel {
        WorktreeInventoryViewModel(rows: [
            .init(
                worktreeId: "wt_101",
                taskID: "task_101",
                path: "\(project.rootPath)/.haneulchi/task_101",
                projectName: project.name,
                branch: "feature/preview",
                disposition: .inUse,
                isPinned: false,
                isDegraded: false,
                sizeBytes: 12_000_000,
                lastAccessedAt: "2026-03-26T07:10:00Z",
            ),
            .init(
                worktreeId: "wt_104",
                taskID: "task_104",
                path: "\(project.rootPath)/.haneulchi/task_104",
                projectName: project.name,
                branch: "feature/review",
                disposition: .recoverable,
                isPinned: true,
                isDegraded: false,
                sizeBytes: 14_200_000,
                lastAccessedAt: "2026-03-25T18:00:00Z",
            ),
            .init(
                worktreeId: "wt_201",
                taskID: "task_201",
                path: "/tmp/auth-service/.haneulchi/task_201",
                projectName: "Auth Service",
                branch: "hotfix/auth",
                disposition: .safeToDelete,
                isPinned: false,
                isDegraded: false,
                sizeBytes: 8_200_000,
                lastAccessedAt: "2026-03-20T09:00:00Z",
            ),
        ])
    }

    static func quickDispatchViewModel(origin: Route = .controlTower)
        -> QuickDispatchComposerViewModel
    {
        var viewModel = QuickDispatchComposerViewModel(
            snapshot: snapshot(activeRoute: origin),
            origin: origin,
        )
        viewModel.messageText = "Please summarize the latest review outcome."
        if let firstTarget = viewModel.targets.first?.id {
            viewModel.selectTarget(id: firstTarget)
        }
        return viewModel
    }

    static func taskDrawerModel() -> TaskDrawerModel? {
        TaskDrawerModel.resolve(
            from: snapshot(activeRoute: .projectFocus),
            workflowStatus: workflowStatus,
            timeline: previewTimeline,
            targetTaskID: "task_104",
        )
    }

    @MainActor
    static func commandPaletteViewModel() -> CommandPaletteViewModel {
        let catalog = CommandPaletteCatalog.build(
            snapshot: snapshot(activeRoute: .projectFocus),
            files: [
                .init(
                    relativePath: "README.md",
                    absolutePath: "\(project.rootPath)/README.md",
                ),
                .init(
                    relativePath: "Sources/HaneulchiApp/PreviewSupport/HaneulchiPreviewGallery.swift",
                    absolutePath: "\(project.rootPath)/Sources/HaneulchiApp/PreviewSupport/HaneulchiPreviewGallery.swift",
                ),
            ],
            tasks: [
                .init(
                    taskID: "task_104",
                    projectID: project.projectID,
                    title: "Review preview gallery coverage",
                    state: .review,
                    automationMode: .autoEligible,
                    linkedSessionID: "ses_review",
                ),
            ],
            inventory: [
                .init(
                    itemID: "wt_104",
                    title: "task_104",
                    rootPath: "\(project.rootPath)/.haneulchi/task_104",
                    disposition: "recoverable",
                ),
            ],
        )
        return CommandPaletteViewModel(catalog: catalog)
    }

    @MainActor
    static func newSessionSheetViewModel() -> NewSessionSheetViewModel {
        let workflowSummary = WorkflowLaunchSummary(
            name: workflowStatus.workflow?.name ?? "Demo Workflow",
            strategy: workflowStatus.workflow?.strategy ?? "worktree",
            baseRoot: workflowStatus.workflow?.baseRoot ?? ".",
            reviewChecklist: workflowStatus.workflow?.reviewChecklist ?? [],
            allowedAgents: workflowStatus.workflow?.allowedAgents ?? [],
        )

        let viewModel = NewSessionSheetViewModel(
            selectedProjectRoot: project.rootPath,
            selectedTaskID: "task_104",
            registry: previewPresetRegistry,
            workflowSummary: workflowSummary,
            preferredPresetID: "codex",
            provisionIsolatedWorkspace: { root, taskID in
                .init(
                    taskID: taskID,
                    worktreeID: "wt_preview_\(taskID)",
                    workspaceRoot: "\(root)/.haneulchi/\(taskID)",
                    baseRoot: ".",
                    branchName: "preview/\(taskID)",
                )
            },
        )
        viewModel.isolatedSessionName = "Preview Task Session"
        return viewModel
    }

    @MainActor
    static func shellModel(
        route: Route = .projectFocus,
        entrySurface: AppShellModel.EntrySurface = .shell,
        showNotifications: Bool = false,
        showCommandPalette: Bool = false,
        showWorkflowDrawer: Bool = false,
        showTaskDrawer: Bool = false,
        showQuickDispatch: Bool = false,
        showNewSessionSheet: Bool = false,
        showInventory: Bool = false,
    ) -> AppShellModel {
        let projectStore = ProjectLauncherStore.inMemory
        let restoreStore = TerminalSessionRestoreStore.inMemory
        let preferencesStore = AppShellPreferencesStore.inMemory
        try? projectStore.recordOpen(project)
        try? projectStore.saveLastSelectedProject(project)
        try? restoreStore.save([
            .genericShell(at: project.rootPath),
            .genericShell(at: "\(project.rootPath)/.haneulchi/task_104"),
        ])
        try? preferencesStore.save(.init(lastActiveRoute: route))

        let snapshot = snapshot(activeRoute: route)
        return AppShellModel(
            entrySurface: entrySurface,
            selectedRoute: route,
            selectedProject: entrySurface == .shell ? project : nil,
            recentProjects: recentProjects,
            readinessReport: readinessReport,
            projectStore: projectStore,
            restoreStore: restoreStore,
            preferencesStore: preferencesStore,
            shellSnapshot: snapshot,
            isNotificationDrawerPresented: showNotifications,
            isWorkflowDrawerPresented: showWorkflowDrawer,
            workflowStatus: workflowStatus,
            settingsStatusViewModel: settingsViewModel(snapshot: snapshot),
            isTaskContextDrawerPresented: showTaskDrawer,
            taskContextDrawerModel: showTaskDrawer ? taskDrawerModel() : nil,
            quickDispatchComposer: showQuickDispatch ? quickDispatchViewModel(origin: route) : nil,
            quickDispatchOrigin: showQuickDispatch ? route : nil,
            isNewSessionSheetPresented: showNewSessionSheet,
            newSessionSheetViewModel: showNewSessionSheet ? newSessionSheetViewModel() : nil,
            isCommandPalettePresented: showCommandPalette,
            commandPaletteViewModel: showCommandPalette ? commandPaletteViewModel() : nil,
            transientNotice: "Preview mode",
            isInventoryPresented: showInventory,
            inventoryViewModel: showInventory ? inventoryViewModel() : nil,
        )
    }

    @MainActor
    static func projectFolderPicker() -> ProjectFolderPicker {
        ProjectFolderPicker(pickFolder: { nil })
    }

    @MainActor
    static func demoWorkspaceScaffold() -> DemoWorkspaceScaffold {
        DemoWorkspaceScaffold(materialize: { project })
    }

    static var notificationItems: [NotificationDrawerModel.Item] {
        NotificationDrawerModel(snapshot: snapshot(activeRoute: .projectFocus)).items
    }

    private static let previewPresetRegistry = PresetRegistry(
        presets: [
            .init(
                id: "codex",
                title: "Codex",
                binary: "codex",
                defaultArgs: ["--model", "gpt-5.4"],
                capabilityFlags: ["dispatch", "commentary"],
                requiresShellIntegration: true,
                installState: .installed,
            ),
            .init(
                id: "generic-shell",
                title: "Generic Shell",
                binary: "/bin/zsh",
                defaultArgs: ["-l"],
                capabilityFlags: ["shell"],
                requiresShellIntegration: false,
                installState: .installed,
            ),
        ],
    )

    private static let previewTimeline: [TaskTimelineEntry] = [
        .init(
            id: "timeline_1",
            kind: "dispatch",
            actor: "codex",
            summary: "Generated preview fixtures for all route surfaces.",
            warningReason: nil,
            createdAt: "2026-03-26T07:12:00Z",
        ),
        .init(
            id: "timeline_2",
            kind: "review",
            actor: "operator",
            summary: "Requested a full canvas pass over route previews.",
            warningReason: nil,
            createdAt: "2026-03-26T07:13:00Z",
        ),
    ]
}
