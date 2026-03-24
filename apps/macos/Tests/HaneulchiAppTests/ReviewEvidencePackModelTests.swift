import Testing
@testable import HaneulchiApp

@Test("review evidence pack model groups summaries, warnings, manifest links, and null-safe placeholders")
func reviewEvidencePackModelFormatsProjectionPayload() {
    let item = ReviewQueueProjectionPayload.Item(
        taskID: "task_review",
        projectID: "proj_demo",
        title: "Review auth flow",
        summary: "Ready for handoff",
        touchedFiles: ["Sources/Auth.swift", "Tests/AuthTests.swift"],
        diffSummary: "+42 -8",
        testsSummary: "12 passing",
        commandSummary: "cargo test -p hc-workflow",
        hookSummary: "after_run_failed: evidence degraded",
        evidenceSummary: "Captured diff, tests, command, and degraded hook note",
        checklistSummary: "1/2 checks complete",
        warnings: ["after_run_failed", "snapshot drift"],
        evidenceManifestPath: "evidence/reviews/task_review/review_01/manifest.json",
        ciRunURL: nil,
        prURL: nil
    )

    let model = ReviewEvidencePackModel(item: item)

    #expect(model.summary == "Ready for handoff")
    #expect(model.primaryFacts.count == 5)
    #expect(model.primaryFacts.contains(where: { $0.label == "hook" && $0.value == "after_run_failed: evidence degraded" }))
    #expect(model.primaryFacts.contains(where: { $0.label == "checklist" && $0.value == "1/2 checks complete" }))
    #expect(model.warningRows == ["after_run_failed", "snapshot drift"])
    #expect(model.manifestPath == "evidence/reviews/task_review/review_01/manifest.json")
    #expect(model.ciRunURL == nil)
    #expect(model.prURL == nil)
    #expect(model.hasDegradedEvidence == true)
}
