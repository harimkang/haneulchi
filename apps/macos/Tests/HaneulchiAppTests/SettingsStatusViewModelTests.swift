import Foundation
@testable import HaneulchiApp
import Testing

private func makeAutomationSnapshot() -> AppShellSnapshot {
    AppShellSnapshot(
        meta: .init(snapshotRev: 7, runtimeRev: 4, projectionRev: 9, snapshotAt: .now),
        ops: .init(
            cadenceMs: 15000,
            lastTickAt: "2026-03-23T12:00:00Z",
            lastReconcileAt: "2026-03-23T12:00:30Z",
            runningSlots: 2,
            maxSlots: 4,
            retryQueueCount: 3,
            queuedClaimCount: 1,
            workflowHealth: .invalidKeptLastGood,
            trackerHealth: "degraded",
            paused: false,
        ),
        app: .init(activeRoute: .settings, focusedSessionID: nil, degradedFlags: [.degraded]),
        projects: [],
        sessions: [],
        attention: [],
        retryQueue: [],
        warnings: [],
        workflow: .init(
            state: .invalidKeptLastGood,
            path: "/tmp/demo/WORKFLOW.md",
            lastGoodHash: "sha256:abc123",
            lastReloadAt: "2026-03-23T12:00:00Z",
            lastError: "front matter parse error",
        ),
        tracker: .init(state: "local_only", lastSyncAt: nil, health: "degraded"),
        recentArtifacts: [
            .init(
                taskID: "task_demo",
                projectID: "proj_demo",
                summary: "Review ready",
                jumpTarget: "review_queue",
                manifestPath: "evidence/manifest.json",
            ),
        ],
    )
}

@Test(
    "settings status view model separates readiness, shell integration, presets, workflow, and automation status",
)
func settingsStatusViewModelSeparatesSections() {
    let project = LauncherProject(
        projectID: "proj_demo",
        name: "demo",
        rootPath: "/tmp/demo",
        lastOpenedAt: .now,
    )
    let report = ReadinessReport(
        project: project,
        checks: [
            .init(
                name: .shell,
                status: .ready,
                headline: "Shell ready",
                detail: "/bin/zsh",
                nextAction: nil,
            ),
            .init(
                name: .git,
                status: .ready,
                headline: "Git ready",
                detail: "2.47.0",
                nextAction: nil,
            ),
            .init(
                name: .shellIntegration,
                status: .degraded,
                headline: "Shell integration missing",
                detail: "Command markers are not configured yet.",
                nextAction: "Open Settings",
            ),
        ],
    )
    let workflowStatus = WorkflowStatusPayload(
        state: .ok,
        path: "/tmp/demo/WORKFLOW.md",
        lastGoodHash: "sha256:abc123",
        lastReloadAt: nil,
        lastError: nil,
        workflow: .init(
            name: "Demo Workflow",
            strategy: "worktree",
            baseRoot: ".",
            reviewChecklist: ["tests passed"],
            allowedAgents: ["codex", "claude"],
            hooks: ["after_create"],
            hookRuns: [:],
            templateBody: nil,
        ),
    )
    let presetRegistry = PresetRegistry(
        presets: [
            .init(
                id: "codex",
                title: "Codex",
                binary: "codex",
                defaultArgs: [],
                capabilityFlags: ["agent", "dispatch"],
                requiresShellIntegration: false,
                installState: .installed,
            ),
            .init(
                id: "claude",
                title: "Claude",
                binary: "claude",
                defaultArgs: [],
                capabilityFlags: ["agent"],
                requiresShellIntegration: true,
                installState: .missing,
            ),
        ],
    )
    let model = SettingsStatusViewModel(
        report: report,
        workflowStatus: workflowStatus,
        presetRegistry: presetRegistry,
        runtimeInfo: .init(rendererID: "swiftterm", transport: "ffi_c_abi", demoMode: false),
        snapshot: makeAutomationSnapshot(),
        terminalSettings: nil,
        runtimeInfoSummary: nil,
        degradedIssues: [],
    )

    #expect(model.readinessRows.map(\.headline) == ["Shell ready", "Git ready"])
    #expect(model.shellIntegrationRow?.headline == "Shell integration missing")
    #expect(model.workflowRow?.title == "Demo Workflow")
    #expect(model.workflowRow?.statusLabel == "ok")
    #expect(model.presetRows.map(\.id) == ["claude", "codex"])
    #expect(model.presetRows.first(where: { $0.id == "codex" })?.statusLabel == "installed")
    #expect(model.presetRows.first(where: { $0.id == "claude" })?.requiresShellIntegration == true)
    #expect(model.automationRows.map(\.id) == [.localAPI, .cli, .workflowWatch, .workflowDefaults])
    #expect(model.automationRows.first(where: { $0.id == .localAPI })?.statusLabel == "available")
    #expect(model.automationRows.first(where: { $0.id == .localAPI })?.detail
        .contains("ffi_c_abi") == true)
    #expect(model.automationRows.first(where: { $0.id == .localAPI })?
        .nextAction == "Export Snapshot")
    #expect(model.automationRows.first(where: { $0.id == .workflowWatch })?
        .statusLabel == "invalid_kept_last_good")
    #expect(model.automationRows.first(where: { $0.id == .workflowDefaults })?.detail
        .contains("15000ms") == true)
    #expect(model.automationRows.first(where: { $0.id == .workflowDefaults })?.detail
        .contains("2/4") == true)
    #expect(model.automationRows.first(where: { $0.id == .workflowDefaults })?.detail
        .contains("3 retry") == true)
    #expect(model.automationRows.first(where: { $0.id == .workflowWatch })?.detail
        .contains("future launches and retries") == true)
    #expect(model.controlPanel != nil)
}

@Test("settings status view model has terminal settings data")
func settingsViewModelHasTerminalSection() {
    let model = SettingsStatusViewModel(
        report: nil,
        workflowStatus: nil,
        presetRegistry: .init(presets: []),
        runtimeInfo: nil,
        snapshot: nil,
        terminalSettings: TerminalSettingsPayload(
            shell: "/bin/zsh",
            defaultCols: 220,
            defaultRows: 50,
            scrollbackLines: 10000,
            fontName: "Menlo",
            theme: "dark",
            cursorStyle: "block",
        ),
        runtimeInfoSummary: nil,
        degradedIssues: [],
    )

    #expect(model.terminalSettingsRow?.shell == "/bin/zsh")
    #expect(model.terminalSettingsRow?.defaultCols == 220)
    #expect(model.terminalSettingsRow?.defaultRows == 50)
    #expect(model.terminalSettingsRow?.scrollbackLines == 10000)
    #expect(model.terminalSettingsRow?.fontName == "Menlo")
    #expect(model.terminalSettingsRow?.theme == "dark")
    #expect(model.terminalSettingsRow?.cursorStyle == "block")
}

@Test(
    "terminal settings payload decodes from JSON with snake_case keys including font, theme, cursor",
)
func terminalSettingsPayloadDecodesExtendedFields() throws {
    let json = Data(
        #"""
        {
          "shell": "/bin/bash",
          "default_cols": 120,
          "default_rows": 40,
          "scrollback_lines": 5000,
          "font_name": "JetBrains Mono",
          "theme": "light",
          "cursor_style": "bar"
        }
        """#.utf8,
    )

    let payload = try JSONDecoder().decode(TerminalSettingsPayload.self, from: json)

    #expect(payload.shell == "/bin/bash")
    #expect(payload.defaultCols == 120)
    #expect(payload.defaultRows == 40)
    #expect(payload.scrollbackLines == 5000)
    #expect(payload.fontName == "JetBrains Mono")
    #expect(payload.theme == "light")
    #expect(payload.cursorStyle == "bar")
}

@Test("settings status view model exposes api runtime info socket_path and transport")
func settingsViewModelHasAPIRuntimeInfo() {
    let summary = RuntimeInfoSummaryPayload(
        socketPath: "/tmp/haneulchi.sock",
        transport: "unix_socket",
        status: "running",
    )
    let model = SettingsStatusViewModel(
        report: nil,
        workflowStatus: nil,
        presetRegistry: .init(presets: []),
        runtimeInfo: nil,
        snapshot: nil,
        terminalSettings: nil,
        runtimeInfoSummary: summary,
        degradedIssues: [],
    )

    #expect(model.apiRuntimeInfoRow?.socketPath == "/tmp/haneulchi.sock")
    #expect(model.apiRuntimeInfoRow?.transport == "unix_socket")
    #expect(model.apiRuntimeInfoRow?.status == "running")
}

@Test("settings status view model has diagnostics readiness rows")
func settingsViewModelHasDiagnosticsRows() {
    let project = LauncherProject(
        projectID: "proj_diag",
        name: "diag",
        rootPath: "/tmp/diag",
        lastOpenedAt: .now,
    )
    let report = ReadinessReport(
        project: project,
        checks: [
            .init(
                name: .shell,
                status: .ready,
                headline: "Shell ready",
                detail: "/bin/zsh",
                nextAction: nil,
            ),
            .init(
                name: .git,
                status: .degraded,
                headline: "Git degraded",
                detail: "version too old",
                nextAction: "Upgrade git",
            ),
        ],
    )
    let model = SettingsStatusViewModel(
        report: report,
        workflowStatus: nil,
        presetRegistry: .init(presets: []),
        runtimeInfo: nil,
        snapshot: nil,
        terminalSettings: nil,
        runtimeInfoSummary: nil,
        degradedIssues: [],
    )

    #expect(model.readinessRows.count == 2)
    #expect(model.readinessRows.map(\.headline).contains("Shell ready"))
    #expect(model.readinessRows.map(\.headline).contains("Git degraded"))
}

@Test(
    "settings status view model renders deferred automation diagnostics when runtime details are unavailable",
)
func settingsStatusViewModelMarksUnavailableAutomationDetails() {
    let model = SettingsStatusViewModel(
        report: nil,
        workflowStatus: nil,
        presetRegistry: .init(presets: []),
        runtimeInfo: nil,
        snapshot: nil,
        terminalSettings: nil,
        runtimeInfoSummary: nil,
        degradedIssues: [],
    )

    #expect(model.readinessRows.isEmpty)
    #expect(model.shellIntegrationRow == nil)
    #expect(model.workflowRow == nil)
    #expect(model.presetRows.isEmpty)
    #expect(model.automationRows.first(where: { $0.id == .localAPI })?.statusLabel == "deferred")
    #expect(model.automationRows.first(where: { $0.id == .workflowWatch })?
        .statusLabel == "deferred")
    #expect(model.controlPanel == nil)
}

@Test("settings status view model surfaces socket path and workflow watch context")
func settingsStatusViewModelSurfacesSocketPathAndWatchContext() {
    let workflowStatus = WorkflowStatusPayload(
        state: .invalidKeptLastGood,
        path: "/tmp/demo/WORKFLOW.md",
        lastGoodHash: "sha256:abc123",
        lastReloadAt: "2026-03-23T12:00:00Z",
        lastError: "front matter parse error",
        workflow: .init(
            name: "Demo Workflow",
            strategy: "worktree",
            baseRoot: ".",
            reviewChecklist: ["tests passed"],
            allowedAgents: ["codex"],
            hooks: ["after_create"],
            hookRuns: [:],
            templateBody: nil,
        ),
    )
    let runtimeSummary = RuntimeInfoSummaryPayload(
        socketPath: "/tmp/haneulchi.sock",
        transport: "unix_domain_socket_local_only",
        status: "running",
    )

    let model = SettingsStatusViewModel(
        report: nil,
        workflowStatus: workflowStatus,
        presetRegistry: .init(presets: []),
        runtimeInfo: .init(rendererID: "swiftterm", transport: "ffi_c_abi", demoMode: false),
        snapshot: makeAutomationSnapshot(),
        terminalSettings: nil,
        runtimeInfoSummary: runtimeSummary,
        degradedIssues: [],
    )

    #expect(model.workflowRow?.detail.contains("watched path") == true)
    #expect(model.workflowRow?.detail.contains("future launches and retries") == true)
    #expect(model.automationRows.first(where: { $0.id == .localAPI })?.detail
        .contains("/tmp/haneulchi.sock") == true)
}
