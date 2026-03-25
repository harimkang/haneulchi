import Foundation

struct ReadinessProbeRunner: Sendable {
    let processClient: SystemProcessClient
    let keychainClient: KeychainAvailabilityClient
    let workflowProbe: WorkflowPresenceProbe
    let presetCatalog: PresetBinaryCatalog

    init(
        processClient: SystemProcessClient,
        keychainClient: KeychainAvailabilityClient,
        workflowProbe: WorkflowPresenceProbe,
        presetCatalog: PresetBinaryCatalog = .launcherDefault,
    ) {
        self.processClient = processClient
        self.keychainClient = keychainClient
        self.workflowProbe = workflowProbe
        self.presetCatalog = presetCatalog
    }

    static let live = Self(
        processClient: .live,
        keychainClient: .live,
        workflowProbe: .live,
    )

    func run(for project: LauncherProject) async throws -> ReadinessReport {
        let shellPath = processClient.detectedShellPath()
        let shellCheck = shellReadyCheck(shellPath: shellPath)
        let gitCheck = try await gitReadyCheck()
        let loginShellCheck = try await loginShellPathCheck(shellPath: shellPath)
        let shellIntegrationCheck = ReadinessCheck(
            name: .shellIntegration,
            status: .degraded,
            headline: "Shell integration not installed",
            detail: "Command markers are not configured yet.",
            nextAction: "Open Settings",
        )
        let presetCheck = try await presetBinaryCheck()
        let keychainCheck = await keychainReadyCheck()
        let workflowCheck = await workflowReadyCheck(rootPath: project.rootPath)

        return ReadinessReport(
            project: project,
            checks: [
                shellCheck,
                loginShellCheck,
                gitCheck,
                shellIntegrationCheck,
                presetCheck,
                keychainCheck,
                workflowCheck,
            ],
        )
    }

    func run(forProjectRoot rootPath: String) async throws -> ReadinessReport {
        try await run(
            for: LauncherProject(
                projectID: UUID().uuidString,
                name: URL(fileURLWithPath: rootPath).lastPathComponent,
                rootPath: rootPath,
                lastOpenedAt: .now,
            ),
        )
    }

    private func shellReadyCheck(shellPath: String?) -> ReadinessCheck {
        guard let shellPath else {
            return ReadinessCheck(
                name: .shell,
                status: .blocked,
                headline: "Shell unavailable",
                detail: "Configured shell could not be determined.",
                nextAction: "Open Settings",
            )
        }

        return ReadinessCheck(
            name: .shell,
            status: .ready,
            headline: "Shell ready",
            detail: shellPath,
            nextAction: nil,
        )
    }

    private func loginShellPathCheck(shellPath: String?) async throws -> ReadinessCheck {
        guard let shellPath else {
            return ReadinessCheck(
                name: .loginShellPath,
                status: .blocked,
                headline: "Login shell PATH unavailable",
                detail: "Configured shell could not be determined.",
                nextAction: "Open Settings",
            )
        }

        switch try await processClient.run("which git", shellPath) {
        case let .success(value):
            return ReadinessCheck(
                name: .loginShellPath,
                status: .ready,
                headline: "Login shell PATH ready",
                detail: value.trimmingCharacters(in: .whitespacesAndNewlines),
                nextAction: nil,
            )
        case let .failure(error):
            return ReadinessCheck(
                name: .loginShellPath,
                status: .degraded,
                headline: "Login shell PATH incomplete",
                detail: error,
                nextAction: "Retry",
            )
        }
    }

    private func gitReadyCheck() async throws -> ReadinessCheck {
        switch try await processClient.run("which git", nil) {
        case let .success(value):
            ReadinessCheck(
                name: .git,
                status: .ready,
                headline: "Git ready",
                detail: value.trimmingCharacters(in: .whitespacesAndNewlines),
                nextAction: nil,
            )
        case let .failure(error):
            ReadinessCheck(
                name: .git,
                status: .degraded,
                headline: "Git unavailable",
                detail: error,
                nextAction: "Open Settings",
            )
        }
    }

    private func presetBinaryCheck() async throws -> ReadinessCheck {
        for command in presetCatalog.commands {
            if case .success = try await processClient.run("which \(command)", nil) {
                return ReadinessCheck(
                    name: .presetBinaries,
                    status: .ready,
                    headline: "Preset binaries detected",
                    detail: "At least one preset binary is available.",
                    nextAction: nil,
                )
            }
        }

        return ReadinessCheck(
            name: .presetBinaries,
            status: .degraded,
            headline: "Preset binaries missing",
            detail: "Generic shell remains available.",
            nextAction: "Open Settings",
        )
    }

    private func keychainReadyCheck() async -> ReadinessCheck {
        if await keychainClient.isAvailable() {
            return ReadinessCheck(
                name: .keychain,
                status: .ready,
                headline: "Keychain available",
                detail: "Secrets can be stored securely.",
                nextAction: nil,
            )
        }

        return ReadinessCheck(
            name: .keychain,
            status: .degraded,
            headline: "Keychain unavailable",
            detail: "Secrets storage is not ready yet.",
            nextAction: "Open Settings",
        )
    }

    private func workflowReadyCheck(rootPath: String) async -> ReadinessCheck {
        switch await workflowProbe.probe(rootPath) {
        case .none:
            ReadinessCheck(
                name: .workflow,
                status: .degraded,
                headline: "Workflow contract not found",
                detail: "Future launches can still use a generic shell.",
                nextAction: "Continue with Generic Shell",
            )
        case .present:
            ReadinessCheck(
                name: .workflow,
                status: .ready,
                headline: "Workflow contract detected",
                detail: "WORKFLOW.md is present.",
                nextAction: nil,
            )
        case .unreadable:
            ReadinessCheck(
                name: .workflow,
                status: .degraded,
                headline: "Workflow contract unreadable",
                detail: "The file exists but could not be read.",
                nextAction: "Retry",
            )
        }
    }
}
