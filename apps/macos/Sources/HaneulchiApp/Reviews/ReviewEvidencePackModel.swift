import Foundation

struct ReviewEvidencePackModel: Equatable, Sendable {
    struct FactRow: Equatable, Sendable {
        let label: String
        let value: String
    }

    let title: String
    let summary: String
    let touchedFiles: [String]
    let primaryFacts: [FactRow]
    let warningRows: [String]
    let manifestPath: String?
    let ciRunURL: String?
    let prURL: String?
    let hasDegradedEvidence: Bool

    init(item: ReviewQueueProjectionPayload.Item) {
        title = item.title
        summary = item.summary
        touchedFiles = item.touchedFiles
        var facts: [FactRow] = []

        if let diffSummary = item.diffSummary {
            facts.append(.init(label: "diff", value: diffSummary))
        }
        if let testsSummary = item.testsSummary {
            facts.append(.init(label: "tests", value: testsSummary))
        }
        if let commandSummary = item.commandSummary {
            facts.append(.init(label: "command", value: commandSummary))
        }
        if let hookSummary = item.hookSummary {
            facts.append(.init(label: "hook", value: hookSummary))
        }
        if let checklistSummary = item.checklistSummary {
            facts.append(.init(label: "checklist", value: checklistSummary))
        }

        primaryFacts = facts
        warningRows = item.warnings
        manifestPath = item.evidenceManifestPath
        ciRunURL = item.ciRunURL
        prURL = item.prURL
        hasDegradedEvidence = item.warnings.contains("after_run_failed")
            || (item.hookSummary?.contains("after_run_failed") ?? false)
    }
}
