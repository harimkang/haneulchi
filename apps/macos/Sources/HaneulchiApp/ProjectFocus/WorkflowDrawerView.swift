import SwiftUI

struct WorkflowStatusPayload: Codable, Equatable, Sendable {
    struct Summary: Codable, Equatable, Sendable {
        let name: String?
        let strategy: String?
        let baseRoot: String?
        let reviewChecklist: [String]
        let allowedAgents: [String]
        let hooks: [String]
        let hookRuns: [String: String]
        let templateBody: String?

        enum CodingKeys: String, CodingKey {
            case name
            case strategy
            case baseRoot = "base_root"
            case reviewChecklist = "review_checklist"
            case allowedAgents = "allowed_agents"
            case hooks
            case hookRuns = "hook_runs"
            case templateBody = "template_body"
        }

        init(from decoder: Decoder) throws {
            let container = try decoder.container(keyedBy: CodingKeys.self)
            name = try container.decodeIfPresent(String.self, forKey: .name)
            strategy = try container.decodeIfPresent(String.self, forKey: .strategy)
            baseRoot = try container.decodeIfPresent(String.self, forKey: .baseRoot)
            reviewChecklist = try container.decodeIfPresent([String].self, forKey: .reviewChecklist) ?? []
            allowedAgents = try container.decodeIfPresent([String].self, forKey: .allowedAgents) ?? []
            hooks = try container.decodeIfPresent([String].self, forKey: .hooks) ?? []
            hookRuns = try container.decodeIfPresent([String: String].self, forKey: .hookRuns) ?? [:]
            templateBody = try container.decodeIfPresent(String.self, forKey: .templateBody)
        }

        init(
            name: String?,
            strategy: String?,
            baseRoot: String?,
            reviewChecklist: [String],
            allowedAgents: [String],
            hooks: [String],
            hookRuns: [String: String],
            templateBody: String?
        ) {
            self.name = name
            self.strategy = strategy
            self.baseRoot = baseRoot
            self.reviewChecklist = reviewChecklist
            self.allowedAgents = allowedAgents
            self.hooks = hooks
            self.hookRuns = hookRuns
            self.templateBody = templateBody
        }
    }

    let state: WorkflowHealth
    let path: String
    let lastGoodHash: String?
    let lastReloadAt: String?
    let lastError: String?
    let workflow: Summary?

    enum CodingKeys: String, CodingKey {
        case state
        case path
        case lastGoodHash = "last_good_hash"
        case lastReloadAt = "last_reload_at"
        case lastError = "last_error"
        case workflow
    }

    init(
        state: WorkflowHealth,
        path: String,
        lastGoodHash: String?,
        lastReloadAt: String?,
        lastError: String?,
        workflow: Summary?
    ) {
        self.state = state
        self.path = path
        self.lastGoodHash = lastGoodHash
        self.lastReloadAt = lastReloadAt
        self.lastError = lastError
        self.workflow = workflow
    }
}

struct WorkflowDrawerView: View {
    let status: WorkflowStatusPayload?
    let onReload: () -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Workflow Contract")
                .font(.headline)

            if let status {
                Text(status.path)
                    .font(.caption)
                    .foregroundStyle(.secondary)
                Text("state: \(status.state.rawValue)")
                    .font(.caption)
                if let name = status.workflow?.name {
                    Text(name)
                        .font(.subheadline.weight(.semibold))
                }
                if let strategy = status.workflow?.strategy, let baseRoot = status.workflow?.baseRoot {
                    Text("strategy: \(strategy) · base root: \(baseRoot)")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
                if !status.workflow.map(\.hooks).unwrapOrDefault().isEmpty {
                    Text("hooks: \(status.workflow?.hooks.joined(separator: ", ") ?? "")")
                        .font(.caption)
                }
                if !status.workflow.map(\.allowedAgents).unwrapOrDefault().isEmpty {
                    Text("allowed agents: \(status.workflow?.allowedAgents.joined(separator: ", ") ?? "")")
                        .font(.caption)
                }
                if !status.workflow.map(\.reviewChecklist).unwrapOrDefault().isEmpty {
                    Text("review: \(status.workflow?.reviewChecklist.joined(separator: ", ") ?? "")")
                        .font(.caption)
                }
                if let lastError = status.lastError {
                    Text(lastError)
                        .font(.caption)
                        .foregroundStyle(HaneulchiChrome.Colors.warning)
                }
            } else {
                Text("No workflow loaded.")
                    .foregroundStyle(.secondary)
            }

            Button("Reload Workflow", action: onReload)
                .buttonStyle(.bordered)
        }
        .padding(16)
        .frame(minWidth: 420, alignment: .topLeading)
        .background(HaneulchiChrome.Colors.surfaceRaised)
    }
}

private extension Optional where Wrapped == [String] {
    func unwrapOrDefault() -> [String] {
        self ?? []
    }
}
