import Testing
@testable import HaneulchiApp

@Test("app shell shows welcome launcher until a project is selected")
@MainActor
func appShellBootstrapsLauncherWhenNoProjectExists() throws {
    let store = ProjectLauncherStore.inMemory
    let model = try AppShellModel.bootstrap(
        projectStore: store,
        restoreStore: .inMemory,
        preferencesStore: .inMemory
    )

    #expect(model.entrySurface == .welcome(.firstRun))
}

@Test("saved last project with degraded startup readiness re-enters the launcher in recovery mode")
@MainActor
func appShellBootstrapsLauncherForDegradedRecovery() async throws {
    let store = ProjectLauncherStore.inMemory
    let project = LauncherProject(
        projectID: "proj_demo",
        name: "demo",
        rootPath: "/tmp/demo",
        lastOpenedAt: .now
    )
    try store.recordOpen(project)
    try store.saveLastSelectedProject(project)

    let report = ReadinessReport(
        project: project,
        checks: [
            .init(name: .shell, status: .ready, headline: "Shell ready", detail: "zsh available", nextAction: nil),
            .init(name: .presetBinaries, status: .degraded, headline: "Preset binaries missing", detail: "Generic shell remains available.", nextAction: "Open Settings"),
        ]
    )

    let model = try await AppShellModel.bootstrap(
        projectStore: store,
        restoreStore: .inMemory,
        preferencesStore: .inMemory,
        initialReport: report
    )

    #expect(model.entrySurface == .welcome(.degradedRecovery))
}

@Test("live default evaluates startup readiness for a previously selected project")
@MainActor
func liveDefaultEvaluatesStartupReadiness() async throws {
    let store = ProjectLauncherStore.inMemory
    let project = LauncherProject(
        projectID: "proj_demo",
        name: "demo",
        rootPath: "/tmp/demo",
        lastOpenedAt: .now
    )
    try store.recordOpen(project)
    try store.saveLastSelectedProject(project)

    let runner = ReadinessProbeRunner(
        processClient: .mock(
            shellPath: "/bin/zsh",
            commands: [
                "which git": .success("/opt/homebrew/bin/git\n"),
                "which claude": .failure("missing"),
                "which codex": .failure("missing"),
                "which gemini": .failure("missing"),
                "which yazi": .failure("missing"),
                "which lazygit": .failure("missing"),
                "which git [shell:/bin/zsh]": .success("/opt/homebrew/bin/git\n"),
            ]
        ),
        keychainClient: .mock(isAvailable: true),
        workflowProbe: .mock(.none)
    )

    let model = AppShellModel.liveDefault(
        projectStore: store,
        restoreStore: .inMemory,
        preferencesStore: .inMemory,
        readinessRunner: runner
    )

    await model.startupReadinessTask?.value

    #expect(model.entrySurface == .welcome(.degradedRecovery))
    #expect(model.readinessReport?.requiresRecoverySurface == true)
}

@Test("bootstrapped shell restores the last active route when a project exists")
@MainActor
func bootstrapRestoresPersistedRoute() throws {
    let store = ProjectLauncherStore.inMemory
    let preferences = AppShellPreferencesStore.inMemory
    let project = LauncherProject(
        projectID: "proj_demo",
        name: "demo",
        rootPath: "/tmp/demo",
        lastOpenedAt: .now
    )
    try store.recordOpen(project)
    try store.saveLastSelectedProject(project)
    try preferences.save(.init(lastActiveRoute: .controlTower))

    let model = try AppShellModel.bootstrap(
        projectStore: store,
        restoreStore: .inMemory,
        preferencesStore: preferences
    )

    #expect(model.selectedRoute == .controlTower)
}
