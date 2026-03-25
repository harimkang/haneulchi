import Testing
@testable import HaneulchiApp

@MainActor
@Test("task board view model preserves the Rust column vocabulary and task counts")
func taskBoardViewModelUsesProjectionVocabularyAndCounts() throws {
    let projection = TaskBoardProjectionPayload(
        selectedProjectID: nil,
        projects: [
            .init(projectID: "proj_demo", title: "proj_demo", taskCount: 2)
        ],
        columns: [
            .init(column: .inbox, tasks: [.fixture(id: "task_inbox", title: "Inbox task", projectID: "proj_demo")]),
            .init(column: .ready, tasks: [.fixture(id: "task_ready", title: "Ready task", projectID: "proj_demo")]),
            .init(column: .running, tasks: []),
            .init(column: .review, tasks: []),
            .init(column: .blocked, tasks: []),
            .init(column: .done, tasks: []),
        ]
    )

    let viewModel = TaskBoardViewModel(
        loadProjection: { _ in projection },
        moveTask: { _, _ in projection }
    )

    try viewModel.reload()

    #expect(viewModel.columns.map(\.column) == TaskBoardColumnID.allCases)
    #expect(viewModel.columns.map(\.taskCount) == [1, 1, 0, 0, 0, 0])
    #expect(viewModel.projectOptions.first?.projectID == "proj_demo")
}

@MainActor
@Test("task board view model re-applies project filter and move responses from the bridge")
func taskBoardViewModelUsesBridgeMoveResponses() throws {
    let allProjects = TaskBoardProjectionPayload(
        selectedProjectID: nil,
        projects: [
            .init(projectID: "proj_alpha", title: "proj_alpha", taskCount: 1),
            .init(projectID: "proj_demo", title: "proj_demo", taskCount: 2),
        ],
        columns: [
            .init(column: .inbox, tasks: [.fixture(id: "task_inbox", title: "Inbox task", projectID: "proj_demo")]),
            .init(column: .ready, tasks: [.fixture(id: "task_ready", title: "Ready task", projectID: "proj_demo")]),
            .init(column: .running, tasks: [.fixture(id: "task_running", title: "Running task", projectID: "proj_alpha")]),
            .init(column: .review, tasks: []),
            .init(column: .blocked, tasks: []),
            .init(column: .done, tasks: []),
        ]
    )
    let movedProjection = TaskBoardProjectionPayload(
        selectedProjectID: "proj_demo",
        projects: [
            .init(projectID: "proj_alpha", title: "proj_alpha", taskCount: 1),
            .init(projectID: "proj_demo", title: "proj_demo", taskCount: 2),
        ],
        columns: [
            .init(column: .inbox, tasks: [.fixture(id: "task_inbox", title: "Inbox task", projectID: "proj_demo")]),
            .init(column: .ready, tasks: []),
            .init(column: .running, tasks: []),
            .init(column: .review, tasks: [.fixture(id: "task_ready", title: "Ready task", projectID: "proj_demo")]),
            .init(column: .blocked, tasks: []),
            .init(column: .done, tasks: []),
        ]
    )

    let recorder = BoardTestRecorder()
    let viewModel = TaskBoardViewModel(
        loadProjection: { projectID in
            recorder.requestedProjects.append(projectID)
            return projectID == "proj_demo" ? movedProjection : allProjects
        },
        moveTask: { taskID, _ in
            recorder.movedTaskIDs.append(taskID)
            return movedProjection
        }
    )

    try viewModel.reload()
    try viewModel.selectProject("proj_demo")
    try viewModel.moveTask(taskID: "task_ready", to: .review)

    #expect(recorder.requestedProjects == [nil, "proj_demo"])
    #expect(recorder.movedTaskIDs == ["task_ready"])
    #expect(viewModel.columns[1].tasks.isEmpty)
    #expect(viewModel.columns[3].tasks.first?.id == "task_ready")
}

@Test("task cards retain evidence readiness and next action hints")
func taskCardPresentationPreservesOperationalMetadata() {
    let reviewTask = TaskBoardProjectionPayload.TaskCard(
        id: "task_review",
        projectID: "proj_demo",
        displayKey: "TASK-104",
        title: "Review auth flow",
        description: "",
        priority: "p1",
        automationMode: .assisted,
        linkedSessionID: "ses_review",
        column: .review
    )
    let blockedTask = TaskBoardProjectionPayload.TaskCard(
        id: "task_blocked",
        projectID: "proj_demo",
        displayKey: "TASK-105",
        title: "Fix failing deploy",
        description: "",
        priority: "p0",
        automationMode: .manual,
        linkedSessionID: nil,
        column: .blocked
    )

    #expect(reviewTask.evidenceReadinessLabel == "review_ready")
    #expect(reviewTask.nextActionLabel == "open review queue")
    #expect(reviewTask.compactMetadataChips == ["P1", "review_ready", "assisted"])
    #expect(reviewTask.contextSummaryLabel == "proj_demo · ses_review")
    #expect(blockedTask.evidenceReadinessLabel == "not_ready")
    #expect(blockedTask.nextActionLabel == "resolve blocker")
    #expect(blockedTask.compactMetadataChips == ["P0", "not_ready", "manual"])
    #expect(blockedTask.contextSummaryLabel == "proj_demo · unassigned")
}

private final class BoardTestRecorder: @unchecked Sendable {
    var requestedProjects: [String?] = []
    var movedTaskIDs: [String] = []
}

private extension TaskBoardProjectionPayload.TaskCard {
    static func fixture(id: String, title: String, projectID: String) -> Self {
        .init(
            id: id,
            projectID: projectID,
            displayKey: id.uppercased(),
            title: title,
            description: "",
            priority: "p1",
            automationMode: .manual,
            linkedSessionID: nil,
            column: .inbox
        )
    }
}
