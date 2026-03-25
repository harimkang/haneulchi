import Foundation

struct ProvisionedTaskWorkspace: Codable, Equatable, Sendable {
    let taskID: String
    let worktreeID: String
    let workspaceRoot: String
    let baseRoot: String
    let branchName: String

    enum CodingKeys: String, CodingKey {
        case taskID = "task_id"
        case worktreeID = "worktree_id"
        case workspaceRoot = "workspace_root"
        case baseRoot = "base_root"
        case branchName = "branch_name"
    }
}

enum NewSessionSheetError: Error, Equatable {
    case missingProjectRoot
    case missingPreset
    case presetUnavailable(String)
    case missingIsolatedName
    case isolatedProvisionUnavailable
    case isolatedProvisionFailed(String)
}

final class NewSessionSheetViewModel: ObservableObject {
    let selectedProjectRoot: String?
    let selectedTaskID: String?
    let registry: PresetRegistry
    let workflowSummary: WorkflowLaunchSummary?
    private let provisionIsolatedWorkspace:
        @Sendable (String, String) throws -> ProvisionedTaskWorkspace
    private let resolveSecretEnv: @Sendable () throws -> [String: String]

    @Published var selectedPresetID: String?
    @Published var isolatedSessionName = ""

    init(
        selectedProjectRoot: String?,
        selectedTaskID: String? = nil,
        registry: PresetRegistry,
        workflowSummary: WorkflowLaunchSummary?,
        preferredPresetID: String? = nil,
        provisionIsolatedWorkspace: @escaping @Sendable (String, String) throws
            -> ProvisionedTaskWorkspace = { _, _ in
                throw NewSessionSheetError.isolatedProvisionUnavailable
            },
        resolveSecretEnv: @escaping @Sendable () throws -> [String: String] = { [:] },
    ) {
        self.selectedProjectRoot = selectedProjectRoot
        self.selectedTaskID = selectedTaskID
        self.registry = registry
        self.workflowSummary = workflowSummary
        self.provisionIsolatedWorkspace = provisionIsolatedWorkspace
        self.resolveSecretEnv = resolveSecretEnv
        selectedPresetID =
            registry.preset(id: preferredPresetID)?.id
                ?? registry.presets.first?.id
    }

    func makeGenericDescriptor() throws -> SessionLaunchDescriptor {
        let root = try requireProjectRoot()
        let secretEnv = resolvedSecretEnv()
        var bundle = TerminalRestoreBundle.genericShell(at: root)
        bundle = TerminalRestoreBundle(
            launch: TerminalSessionLaunchRequest(
                program: bundle.launch.program,
                args: bundle.launch.args,
                currentDirectory: bundle.launch.currentDirectory,
                geometry: bundle.launch.geometry,
                environment: secretEnv,
            ),
            geometry: bundle.geometry,
        )
        return SessionLaunchDescriptor(
            mode: .generic,
            title: "Generic Shell",
            presetID: nil,
            restoreBundle: bundle,
            workspaceRoot: root,
            workflowSummary: workflowSummary,
        )
    }

    func makePresetDescriptor() throws -> SessionLaunchDescriptor {
        let root = try requireProjectRoot()
        guard let preset = registry.preset(id: selectedPresetID) else {
            throw NewSessionSheetError.missingPreset
        }
        guard preset.installState == .installed else {
            throw NewSessionSheetError.presetUnavailable(preset.id)
        }

        let secretEnv = resolvedSecretEnv()
        let launch = TerminalSessionLaunchRequest(
            program: preset.binary,
            args: preset.defaultArgs,
            currentDirectory: root,
            geometry: .defaultShell,
            environment: secretEnv,
        )
        return SessionLaunchDescriptor(
            mode: .preset,
            title: preset.title,
            presetID: preset.id,
            restoreBundle: .init(launch: launch, geometry: .defaultShell),
            workspaceRoot: root,
            workflowSummary: workflowSummary,
        )
    }

    func makeIsolatedDescriptor() throws -> SessionLaunchDescriptor {
        let root = try requireProjectRoot()
        let trimmedName = isolatedSessionName.trimmingCharacters(in: .whitespacesAndNewlines)
        let taskID = selectedTaskID ?? trimmedName
        guard !taskID.isEmpty else {
            throw NewSessionSheetError.missingIsolatedName
        }
        let provisionedWorkspace: ProvisionedTaskWorkspace
        do {
            provisionedWorkspace = try provisionIsolatedWorkspace(root, taskID)
        } catch let error as NewSessionSheetError {
            throw error
        } catch {
            throw NewSessionSheetError.isolatedProvisionFailed(String(describing: error))
        }
        let title = trimmedName.isEmpty ? taskID : trimmedName

        let secretEnv = resolvedSecretEnv()
        var bundle = TerminalRestoreBundle.genericShell(at: provisionedWorkspace.workspaceRoot)
        bundle = TerminalRestoreBundle(
            launch: TerminalSessionLaunchRequest(
                program: bundle.launch.program,
                args: bundle.launch.args,
                currentDirectory: bundle.launch.currentDirectory,
                geometry: bundle.launch.geometry,
                environment: secretEnv,
            ),
            geometry: bundle.geometry,
        )
        return SessionLaunchDescriptor(
            mode: .isolated,
            title: title,
            presetID: nil,
            restoreBundle: bundle,
            workspaceRoot: provisionedWorkspace.workspaceRoot,
            workflowSummary: workflowSummary,
        )
    }

    /// Resolves Keychain secrets into environment variables.
    /// On failure, logs and returns an empty dict so launch is not blocked.
    private func resolvedSecretEnv() -> [String: String] {
        do {
            return try resolveSecretEnv()
        } catch {
            NSLog(
                "[NewSessionSheetViewModel] resolveSecretEnv failed (continuing without secrets): %@",
                String(describing: error),
            )
            return [:]
        }
    }

    private func requireProjectRoot() throws -> String {
        guard let selectedProjectRoot, !selectedProjectRoot.isEmpty else {
            throw NewSessionSheetError.missingProjectRoot
        }
        return selectedProjectRoot
    }
}
