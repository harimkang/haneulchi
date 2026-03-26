import SwiftUI

struct TaskContextDrawerView: View {
    @Environment(\.viewportContext) private var viewportContext
    let model: TaskDrawerModel?
    let onPrimaryAction: ((TaskDrawerModel) -> Void)?
    let onQuickDispatch: (() -> Void)?

    init(
        model: TaskDrawerModel?,
        onPrimaryAction: ((TaskDrawerModel) -> Void)? = nil,
        onQuickDispatch: (() -> Void)? = nil,
    ) {
        self.model = model
        self.onPrimaryAction = onPrimaryAction
        self.onQuickDispatch = onQuickDispatch
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Task Context")
                .font(.headline)

            if let model {
                Text(model.taskID)
                    .font(.title3.weight(.semibold))

                Text("session: \(model.sessionTitle)")
                    .font(.caption)
                    .foregroundStyle(.secondary)
                Text("dispatch: \(model.dispatchReason ?? model.dispatchState.rawValue)")
                    .font(.caption)
                    .foregroundStyle(.secondary)

                if let workspaceRoot = model.workspaceRoot {
                    Text("workspace root: \(workspaceRoot)")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }

                VStack(alignment: .leading, spacing: 4) {
                    Text(model.workflowName)
                        .font(.headline)
                    Text(model.workflowPath)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                    if let strategy = model.strategy, let baseRoot = model.baseRoot {
                        Text("strategy: \(strategy) · base root: \(baseRoot)")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    } else if let baseRoot = model.baseRoot {
                        Text("base root: \(baseRoot)")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                    if let lastGoodHash = model.lastGoodHash {
                        Text("last good: \(lastGoodHash)")
                            .font(.caption)
                    }
                    if let lastReloadAt = model.lastReloadAt {
                        Text("last reload: \(lastReloadAt)")
                            .font(.caption)
                    }
                    Text("binding: \(model.workflowBindingSummary)")
                        .font(.caption)
                    Text("lineage: \(model.lineageSummary)")
                        .font(.caption)
                    if !model.reviewChecklist.isEmpty {
                        Text("review: \(model.reviewChecklist.joined(separator: ", "))")
                            .font(.caption)
                    }
                    if !model.allowedAgents.isEmpty {
                        Text("allowed agents: \(model.allowedAgents.joined(separator: ", "))")
                            .font(.caption)
                    }
                    if let lastError = model.lastError {
                        Text(lastError)
                            .font(.caption)
                            .foregroundStyle(HaneulchiChrome.Colors.warning)
                    }
                    if let bootstrapPhaseSummary = model.bootstrapPhaseSummary {
                        Text("bootstrap: \(bootstrapPhaseSummary)")
                            .font(.caption)
                    }
                    if let renderedPromptPath = model.renderedPromptPath {
                        Text("prompt: \(renderedPromptPath)")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                }

                TaskAutomationSection(model: model)

                TaskTimelineSection(title: "Timeline", entries: model.timeline)

                Button(model.primaryActionTitle) {
                    onPrimaryAction?(model)
                }
                .buttonStyle(.borderedProminent)
                .disabled(onPrimaryAction == nil)

                Button {
                    onQuickDispatch?()
                } label: {
                    Label(
                        "Quick Dispatch",
                        systemImage: HaneulchiChromeAction.dispatch.symbolName,
                    )
                }
                .buttonStyle(.bordered)
                .disabled(onQuickDispatch == nil)
            } else {
                Text("No linked task or workflow context.")
                    .foregroundStyle(.secondary)
            }
        }
        .padding(16)
        .frame(
            width: viewportContext.drawerWidthPolicy(for: .context).resolvedWidth(
                availableWidth: viewportContext.width,
            ),
            alignment: .topLeading,
        )
        .background(HaneulchiChrome.Colors.surfaceRaised)
    }
}
