import Foundation
import Testing
@testable import HaneulchiApp

@Test("preset registry loads documented presets and resolves installed state")
func presetRegistryLoadsAndResolvesInstallState() throws {
    let registry = try PresetRegistry.loadDefault(
        commandResolver: { command in
            command == "codex" || command == "yazi"
        }
    )

    #expect(registry.presets.map(\.id) == ["claude", "codex", "gemini", "yazi", "lazygit"])
    #expect(registry.presets.first(where: { $0.id == "codex" })?.installState == .installed)
    #expect(registry.presets.first(where: { $0.id == "claude" })?.installState == .missing)
}

@Test("new session sheet builds generic, preset, and isolated launch descriptors")
func newSessionSheetBuildsLaunchDescriptors() throws {
    let registry = PresetRegistry(
        presets: [
            .init(
                id: "codex",
                title: "Codex",
                binary: "codex",
                defaultArgs: ["--sandbox", "workspace-write"],
                capabilityFlags: ["agent", "dispatch"],
                requiresShellIntegration: false,
                installState: .installed
            )
        ]
    )
    let workflowSummary = WorkflowLaunchSummary(
        name: "Demo Workflow",
        strategy: "worktree",
        baseRoot: ".",
        reviewChecklist: ["tests passed"],
        allowedAgents: ["codex", "claude"]
    )

    let viewModel = NewSessionSheetViewModel(
        selectedProjectRoot: "/tmp/demo",
        registry: registry,
        workflowSummary: workflowSummary
    )

    let generic = try viewModel.makeGenericDescriptor()
    #expect(generic.mode == .generic)
    #expect(generic.restoreBundle.launch.program == "/bin/zsh")
    #expect(generic.restoreBundle.launch.currentDirectory == "/tmp/demo")

    viewModel.selectedPresetID = "codex"
    let preset = try viewModel.makePresetDescriptor()
    #expect(preset.mode == .preset)
    #expect(preset.restoreBundle.launch.program == "codex")
    #expect(preset.restoreBundle.launch.args == ["--sandbox", "workspace-write"])
    #expect(preset.workflowSummary?.allowedAgents == ["codex", "claude"])

    viewModel.isolatedSessionName = "task-104"
    let isolated = try viewModel.makeIsolatedDescriptor()
    #expect(isolated.mode == .isolated)
    #expect(isolated.restoreBundle.launch.currentDirectory?.contains(".haneulchi/isolated/task-104") == true)
    #expect(isolated.workspaceRoot?.contains(".haneulchi/isolated/task-104") == true)
}
