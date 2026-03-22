import SwiftUI

struct TaskContextDrawerView: View {
    struct Model: Equatable, Sendable {
        let taskID: String
        let sessionTitle: String
        let workspaceRoot: String?
        let workflowName: String
        let workflowPath: String
        let strategy: String?
        let baseRoot: String?
        let reviewChecklist: [String]
        let allowedAgents: [String]
        let lastGoodHash: String?
        let lastReloadAt: String?
        let lastError: String?
    }

    let model: Model?

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

                if let workspaceRoot = model.workspaceRoot {
                    Text("workspace: \(workspaceRoot)")
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
                    }
                    if let lastGoodHash = model.lastGoodHash {
                        Text("last good: \(lastGoodHash)")
                            .font(.caption)
                    }
                    if let lastReloadAt = model.lastReloadAt {
                        Text("last reload: \(lastReloadAt)")
                            .font(.caption)
                    }
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
                }
            } else {
                Text("No linked task or workflow context.")
                    .foregroundStyle(.secondary)
            }
        }
        .padding(16)
        .frame(minWidth: 420, alignment: .topLeading)
        .background(HaneulchiChrome.Colors.surfaceRaised)
    }
}
