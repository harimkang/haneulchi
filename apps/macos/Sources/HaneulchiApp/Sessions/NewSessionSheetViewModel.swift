import Foundation

enum NewSessionSheetError: Error, Equatable {
    case missingProjectRoot
    case missingPreset
    case presetUnavailable(String)
    case missingIsolatedName
}

final class NewSessionSheetViewModel: ObservableObject {
    let selectedProjectRoot: String?
    let registry: PresetRegistry
    let workflowSummary: WorkflowLaunchSummary?

    @Published var selectedPresetID: String?
    @Published var isolatedSessionName = ""

    init(
        selectedProjectRoot: String?,
        registry: PresetRegistry,
        workflowSummary: WorkflowLaunchSummary?
    ) {
        self.selectedProjectRoot = selectedProjectRoot
        self.registry = registry
        self.workflowSummary = workflowSummary
        self.selectedPresetID = registry.presets.first?.id
    }

    func makeGenericDescriptor() throws -> SessionLaunchDescriptor {
        let root = try requireProjectRoot()
        return SessionLaunchDescriptor(
            mode: .generic,
            title: "Generic Shell",
            presetID: nil,
            restoreBundle: .genericShell(at: root),
            workspaceRoot: root,
            workflowSummary: workflowSummary
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

        let launch = TerminalSessionLaunchRequest(
            program: preset.binary,
            args: preset.defaultArgs,
            currentDirectory: root,
            geometry: .defaultShell
        )
        return SessionLaunchDescriptor(
            mode: .preset,
            title: preset.title,
            presetID: preset.id,
            restoreBundle: .init(launch: launch, geometry: .defaultShell),
            workspaceRoot: root,
            workflowSummary: workflowSummary
        )
    }

    func makeIsolatedDescriptor() throws -> SessionLaunchDescriptor {
        let root = try requireProjectRoot()
        let trimmedName = isolatedSessionName.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmedName.isEmpty else {
            throw NewSessionSheetError.missingIsolatedName
        }

        let isolatedRoot = URL(fileURLWithPath: root)
            .appendingPathComponent(".haneulchi/isolated", isDirectory: true)
            .appendingPathComponent(trimmedName, isDirectory: true)
            .path

        return SessionLaunchDescriptor(
            mode: .isolated,
            title: trimmedName,
            presetID: nil,
            restoreBundle: .genericShell(at: isolatedRoot),
            workspaceRoot: isolatedRoot,
            workflowSummary: workflowSummary
        )
    }

    private func requireProjectRoot() throws -> String {
        guard let selectedProjectRoot, !selectedProjectRoot.isEmpty else {
            throw NewSessionSheetError.missingProjectRoot
        }
        return selectedProjectRoot
    }
}
