import Foundation

struct LocalAppShellSnapshotSource: Sendable {
    let restoreStore: TerminalSessionRestoreStore
    private let dateProvider: @Sendable () -> Date

    init(
        restoreStore: TerminalSessionRestoreStore,
        dateProvider: @escaping @Sendable () -> Date = Date.init,
    ) {
        self.restoreStore = restoreStore
        self.dateProvider = dateProvider
    }

    func load(
        activeRoute: Route,
        selectedProject: LauncherProject?,
        readinessReport: ReadinessReport?,
        recentProjects: [LauncherProject],
    ) async throws -> AppShellSnapshot {
        let restoreBundles = try restoreStore.load()
        let sessions = makeSessions(from: restoreBundles)
        let runningSessions = sessions.filter { $0.runtimeState == .running }
        let warnings = makeWarnings(from: readinessReport)
        let attention = makeAttention(from: warnings)
        let projects = makeProjects(
            selectedProject: selectedProject,
            recentProjects: recentProjects,
            sessions: runningSessions,
            warnings: warnings,
            attention: attention,
            readinessReport: readinessReport,
        )
        let workflowHealth = projects.first(where: { $0.status == .active })?.workflowState
            ?? projects.first?.workflowState
            ?? .none
        let focusedSessionID = runningSessions.first?.sessionID

        return AppShellSnapshot(
            meta: .init(
                snapshotRev: 1,
                runtimeRev: 1,
                projectionRev: 1,
                snapshotAt: dateProvider(),
            ),
            ops: .init(
                runningSlots: runningSessions.count,
                maxSlots: max(1, runningSessions.count),
                retryQueueCount: 0,
                workflowHealth: workflowHealth,
            ),
            app: .init(
                activeRoute: activeRoute,
                focusedSessionID: focusedSessionID,
                degradedFlags: warnings.map(\.severity),
            ),
            projects: projects,
            sessions: sessions,
            attention: attention,
            retryQueue: [],
            warnings: warnings,
            workflow: nil,
            tracker: .init(state: "local_only", lastSyncAt: nil, health: "ok"),
        )
    }

    private func makeSessions(from restoreBundles: [TerminalRestoreBundle])
        -> [AppShellSnapshot.SessionSummary]
    {
        restoreBundles.enumerated().map { index, bundle in
            let currentDirectory = bundle.launch.currentDirectory
            let sessionTitle = currentDirectory
                .flatMap {
                    URL(fileURLWithPath: $0).lastPathComponent
                        .isEmpty ? nil : URL(fileURLWithPath: $0).lastPathComponent
                }
                ?? "Generic Shell"

            return .init(
                sessionID: "restore-\(index + 1)",
                title: sessionTitle,
                currentDirectory: currentDirectory,
                mode: .generic,
                runtimeState: .exited,
                manualControlState: .none,
                dispatchState: .notDispatchable,
                unreadCount: 0,
            )
        }
    }

    private func makeWarnings(from readinessReport: ReadinessReport?)
        -> [AppShellSnapshot.WarningSummary]
    {
        guard let readinessReport else {
            return []
        }

        return readinessReport.checks.compactMap { check in
            switch check.status {
            case .ready:
                nil
            case .degraded:
                .init(
                    warningID: "warning-\(check.name.rawValue)",
                    severity: .degraded,
                    headline: check.headline,
                    nextAction: check.nextAction,
                )
            case .blocked:
                .init(
                    warningID: "warning-\(check.name.rawValue)",
                    severity: .failed,
                    headline: check.headline,
                    nextAction: check.nextAction,
                )
            }
        }
    }

    private func makeProjects(
        selectedProject: LauncherProject?,
        recentProjects: [LauncherProject],
        sessions: [AppShellSnapshot.SessionSummary],
        warnings: [AppShellSnapshot.WarningSummary],
        attention: [AppShellSnapshot.AttentionSummary],
        readinessReport: ReadinessReport?,
    ) -> [AppShellSnapshot.ProjectSummary] {
        let orderedProjects = orderedProjects(
            selectedProject: selectedProject,
            recentProjects: recentProjects,
        )
        return orderedProjects.map { project in
            let projectSessionCount = sessions
                .count(where: { $0.currentDirectory == project.rootPath })
            let hasFailure = warnings.contains { $0.severity == .failed }
            let projectAttentionCount = project.projectID == selectedProject?.projectID ? attention
                .count : 0

            return .init(
                projectID: project.projectID,
                name: project.name,
                rootPath: project.rootPath,
                status: project.projectID == selectedProject?.projectID
                    ? (hasFailure ? .error : .active)
                    : .idle,
                workflowState: workflowHealth(for: project, readinessReport: readinessReport),
                sessionCount: projectSessionCount,
                attentionCount: projectAttentionCount,
            )
        }
    }

    private func makeAttention(
        from warnings: [AppShellSnapshot.WarningSummary],
    ) -> [AppShellSnapshot.AttentionSummary] {
        warnings.enumerated().map { index, warning in
            .init(
                attentionID: "attention-\(index + 1)",
                headline: warning.headline,
                severity: warning.severity,
                targetRoute: .attentionCenter,
                targetSessionID: nil,
            )
        }
    }

    private func orderedProjects(
        selectedProject: LauncherProject?,
        recentProjects: [LauncherProject],
    ) -> [LauncherProject] {
        var projects: [LauncherProject] = []
        var seen = Set<String>()

        if let selectedProject {
            projects.append(selectedProject)
            seen.insert(selectedProject.projectID)
        }

        for project in recentProjects where !seen.contains(project.projectID) {
            projects.append(project)
            seen.insert(project.projectID)
        }

        return projects
    }

    private func workflowHealth(
        for project: LauncherProject,
        readinessReport: ReadinessReport?,
    ) -> WorkflowHealth {
        guard readinessReport?.project?.projectID == project.projectID else {
            return .none
        }

        guard let workflowCheck = readinessReport?.check(named: .workflow) else {
            return .none
        }

        switch workflowCheck.status {
        case .ready:
            return .ok
        case .degraded, .blocked:
            return .invalidKeptLastGood
        }
    }
}
