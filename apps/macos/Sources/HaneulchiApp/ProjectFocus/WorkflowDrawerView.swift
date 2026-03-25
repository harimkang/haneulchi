import SwiftUI

struct WorkflowStatusPayload: Codable, Equatable, Sendable {
    struct HookPhaseResult: Codable, Equatable, Sendable {
        let phase: String
        let commandPath: String?
        let exitCode: Int?
        let stdout: String
        let stderr: String
        let succeeded: Bool

        enum CodingKeys: String, CodingKey {
            case phase
            case commandPath = "command_path"
            case exitCode = "exit_code"
            case stdout
            case stderr
            case succeeded
        }
    }

    struct BootstrapSummary: Codable, Equatable, Sendable {
        let workspaceRoot: String
        let baseRoot: String
        let sessionCwd: String
        let renderedPromptPath: String
        let phaseSequence: [String]
        let hookPhaseResults: [HookPhaseResult]
        let outcomeCode: String
        let warningCodes: [String]
        let claimReleased: Bool
        let launchExitCode: Int?
        let lastKnownGoodHash: String?

        enum CodingKeys: String, CodingKey {
            case workspaceRoot = "workspace_root"
            case baseRoot = "base_root"
            case sessionCwd = "session_cwd"
            case renderedPromptPath = "rendered_prompt_path"
            case phaseSequence = "phase_sequence"
            case hookPhaseResults = "hook_phase_results"
            case outcomeCode = "outcome_code"
            case warningCodes = "warning_codes"
            case claimReleased = "claim_released"
            case launchExitCode = "launch_exit_code"
            case lastKnownGoodHash = "last_known_good_hash"
        }
    }

    struct Summary: Codable, Equatable, Sendable {
        let name: String?
        let strategy: String?
        let baseRoot: String?
        let requireReview: Bool
        let maxRuntimeMinutes: Int?
        let unsafeOverridePolicy: String?
        let reviewChecklist: [String]
        let allowedAgents: [String]
        let hooks: [String]
        let hookRuns: [String: String]
        let templateBody: String?

        enum CodingKeys: String, CodingKey {
            case name
            case strategy
            case baseRoot = "base_root"
            case requireReview = "require_review"
            case maxRuntimeMinutes = "max_runtime_minutes"
            case unsafeOverridePolicy = "unsafe_override_policy"
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
            requireReview = try container.decodeIfPresent(Bool.self, forKey: .requireReview) ?? false
            maxRuntimeMinutes = try container.decodeIfPresent(Int.self, forKey: .maxRuntimeMinutes)
            unsafeOverridePolicy = try container.decodeIfPresent(String.self, forKey: .unsafeOverridePolicy)
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
            requireReview: Bool = false,
            maxRuntimeMinutes: Int? = nil,
            unsafeOverridePolicy: String? = nil,
            reviewChecklist: [String],
            allowedAgents: [String],
            hooks: [String],
            hookRuns: [String: String],
            templateBody: String?
        ) {
            self.name = name
            self.strategy = strategy
            self.baseRoot = baseRoot
            self.requireReview = requireReview
            self.maxRuntimeMinutes = maxRuntimeMinutes
            self.unsafeOverridePolicy = unsafeOverridePolicy
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
    let lastBootstrap: BootstrapSummary?
    let workflow: Summary?

    enum CodingKeys: String, CodingKey {
        case state
        case path
        case lastGoodHash = "last_good_hash"
        case lastReloadAt = "last_reload_at"
        case lastError = "last_error"
        case lastBootstrap = "last_bootstrap"
        case workflow
    }

    init(
        state: WorkflowHealth,
        path: String,
        lastGoodHash: String?,
        lastReloadAt: String?,
        lastError: String?,
        lastBootstrap: BootstrapSummary? = nil,
        workflow: Summary?
    ) {
        self.state = state
        self.path = path
        self.lastGoodHash = lastGoodHash
        self.lastReloadAt = lastReloadAt
        self.lastError = lastError
        self.lastBootstrap = lastBootstrap
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
                if let lastGoodHash = status.lastGoodHash {
                    Text("last known good: \(lastGoodHash)")
                        .font(.caption)
                }
                if let lastReloadAt = status.lastReloadAt {
                    Text("last reload: \(lastReloadAt)")
                        .font(.caption)
                }
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
                    Text("last error: \(lastError)")
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
