import Testing
@testable import HaneulchiApp

@MainActor
@Test("review queue view model only surfaces review-ready items and keeps evidence summary visible")
func reviewQueueViewModelUsesReviewReadyProjection() throws {
    let projection = ReviewQueueProjectionPayload(
        items: [
            .init(
                taskID: "task_review",
                projectID: "proj_demo",
                title: "Review auth flow",
                summary: "Ready for handoff",
                touchedFiles: ["Sources/Auth.swift", "Tests/AuthTests.swift"],
                diffSummary: "+42 -8",
                testsSummary: "12 passing",
                commandSummary: "cargo test -p hc-workflow",
                warnings: ["snapshot drift"],
                evidenceManifestPath: "evidence/reviews/task_review/review_01/manifest.json"
            )
        ],
        degradedReason: nil
    )
    let viewModel = ReviewQueueViewModel(loadProjection: { projection })

    try viewModel.reload()

    #expect(viewModel.items.count == 1)
    #expect(viewModel.selectedItem?.taskID == "task_review")
    #expect(viewModel.selectedItem?.touchedFiles.count == 2)
    #expect(viewModel.selectedItem?.warnings == ["snapshot drift"])
}

@MainActor
@Test("review queue view model makes empty and degraded states explicit")
func reviewQueueViewModelExposesEmptyAndDegradedStates() throws {
    let degraded = ReviewQueueProjectionPayload(items: [], degradedReason: "review_evidence_unavailable")
    let viewModel = ReviewQueueViewModel(loadProjection: { degraded })

    try viewModel.reload()

    #expect(viewModel.items.isEmpty)
    #expect(viewModel.emptyStateMessage == "No review-ready tasks.")
    #expect(viewModel.degradedReason == "review_evidence_unavailable")
}
