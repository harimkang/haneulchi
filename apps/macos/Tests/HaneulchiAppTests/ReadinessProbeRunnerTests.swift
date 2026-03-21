import Testing
@testable import HaneulchiApp

@Test("readiness report keeps generic shell available when preset binaries are missing")
func readinessReportAllowsGenericShellFallback() async throws {
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

    let report = try await runner.run(forProjectRoot: "/tmp/demo")

    #expect(report.canContinueWithGenericShell == true)
    #expect(report.check(named: .presetBinaries)?.status == .degraded)
}

@Test("informational readiness gaps do not require startup recovery")
func readinessReportDoesNotRequireRecoveryForInformationalGaps() async throws {
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

    let report = try await runner.run(forProjectRoot: "/tmp/demo")

    #expect(report.requiresRecoverySurface == false)
    #expect(report.check(named: .shellIntegration)?.status == .degraded)
    #expect(report.check(named: .workflow)?.status == .degraded)
}

@Test("blocked shell still requires startup recovery")
func readinessReportRequiresRecoveryForBlockedShell() async throws {
    let runner = ReadinessProbeRunner(
        processClient: .mock(
            shellPath: nil,
            commands: [
                "which git": .success("/opt/homebrew/bin/git\n"),
                "which claude": .failure("missing"),
                "which codex": .failure("missing"),
                "which gemini": .failure("missing"),
                "which yazi": .failure("missing"),
                "which lazygit": .failure("missing"),
            ]
        ),
        keychainClient: .mock(isAvailable: true),
        workflowProbe: .mock(.present)
    )

    let report = try await runner.run(forProjectRoot: "/tmp/demo")

    #expect(report.requiresRecoverySurface == true)
    #expect(report.canContinueWithGenericShell == false)
    #expect(report.check(named: .shell)?.status == .blocked)
}

@Test("readiness runner keeps an opaque project id separate from root path")
func readinessRunnerDoesNotBakeProjectIdentityIntoRootPath() async throws {
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
    let project = LauncherProject(
        projectID: "proj_demo",
        name: "demo",
        rootPath: "/tmp/demo",
        lastOpenedAt: .now
    )

    let report = try await runner.run(for: project)

    #expect(report.project?.projectID == "proj_demo")
    #expect(report.project?.rootPath == "/tmp/demo")
}

@Test("login shell readiness uses the configured shell instead of hard-coded zsh")
func readinessRunnerUsesConfiguredShell() async throws {
    let runner = ReadinessProbeRunner(
        processClient: .mock(
            shellPath: "/bin/bash",
            commands: [
                "which git": .success("/opt/homebrew/bin/git\n"),
                "which claude": .failure("missing"),
                "which codex": .failure("missing"),
                "which gemini": .failure("missing"),
                "which yazi": .failure("missing"),
                "which lazygit": .failure("missing"),
                "which git [shell:/bin/bash]": .success("/usr/local/bin/git\n"),
            ]
        ),
        keychainClient: .mock(isAvailable: true),
        workflowProbe: .mock(.none)
    )

    let report = try await runner.run(forProjectRoot: "/tmp/demo")

    #expect(report.check(named: .shell)?.detail == "/bin/bash")
    #expect(report.check(named: .loginShellPath)?.detail == "/usr/local/bin/git")
}
