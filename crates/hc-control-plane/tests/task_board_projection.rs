use hc_control_plane::{ReviewDecision, ReviewQueueService, TaskBoardColumnSummary, TaskBoardService};
use hc_domain::TaskColumn;

#[test]
fn board_projection_groups_tasks_by_fixed_columns() {
    let service = TaskBoardService::demo().expect("demo board");
    let projection = service.board(None).expect("board projection");

    assert_eq!(
        projection
            .columns
            .iter()
            .map(|column| column.column)
            .collect::<Vec<_>>(),
        vec![
            TaskColumn::Inbox,
            TaskColumn::Ready,
            TaskColumn::Running,
            TaskColumn::Review,
            TaskColumn::Blocked,
            TaskColumn::Done,
        ]
    );
    assert_eq!(
        projection.projects,
        vec![
            TaskBoardColumnSummary::new("proj_alpha", 1),
            TaskBoardColumnSummary::new("proj_demo", 2),
        ]
    );
}

#[test]
fn project_filter_and_move_mutation_update_storage_backed_projection() {
    let service = TaskBoardService::demo().expect("demo board");

    let filtered = service.board(Some("proj_demo")).expect("filtered board");
    assert_eq!(filtered.selected_project_id.as_deref(), Some("proj_demo"));
    assert_eq!(filtered.columns[0].tasks.len(), 1);
    assert_eq!(filtered.columns[1].tasks.len(), 1);
    assert!(filtered.columns[3].tasks.is_empty());

    service
        .move_task("task_ready", TaskColumn::Review, "operator_drag")
        .expect("move task");

    let updated = service.board(Some("proj_demo")).expect("updated board");
    assert!(updated.columns[1].tasks.is_empty());
    assert_eq!(updated.columns[3].tasks.len(), 1);
    assert_eq!(updated.columns[3].tasks[0].id, "task_ready");
}

#[test]
fn review_ready_queue_only_lists_pending_review_items() {
    let service = ReviewQueueService::demo().expect("review queue");

    let projection = service.review_ready_projection().expect("review queue projection");

    assert_eq!(projection.items.len(), 1);
    assert_eq!(projection.items[0].task_id, "task_review");
    assert_eq!(projection.items[0].touched_files, vec!["Sources/Auth.swift", "Tests/AuthTests.swift"]);
    assert_eq!(projection.items[0].warnings, vec!["snapshot drift"]);
}

#[test]
fn timeline_accept_request_changes_manual_continue_and_follow_up_update_projections() {
    let service = ReviewQueueService::demo().expect("review queue");

    let accepted = service
        .apply_decision("task_review", ReviewDecision::Accept)
        .expect("accept decision");
    assert_eq!(accepted.task.column, TaskColumn::Done);
    assert_eq!(accepted.task.linked_session_id, None);

    let request_changes = ReviewQueueService::demo()
        .expect("review queue")
        .apply_decision("task_review", ReviewDecision::RequestChanges)
        .expect("request changes");
    assert_eq!(request_changes.task.column, TaskColumn::Ready);

    let manual_continue = ReviewQueueService::demo()
        .expect("review queue")
        .apply_decision("task_review", ReviewDecision::ManualContinue)
        .expect("manual continue");
    assert_eq!(manual_continue.task.column, TaskColumn::Running);
    assert_eq!(manual_continue.task.linked_session_id.as_deref(), Some("ses_02"));

    let follow_up = ReviewQueueService::demo()
        .expect("review queue")
        .apply_decision("task_review", ReviewDecision::FollowUp)
        .expect("follow up");
    assert_eq!(follow_up.follow_up_task.as_ref().expect("follow up").column, TaskColumn::Inbox);

    let timeline = follow_up.timeline;
    assert!(timeline.iter().any(|item| item.kind == "task_created"));
    assert!(timeline.iter().any(|item| item.kind == "review_decided"));
    assert!(timeline.iter().any(|item| item.kind == "follow_up_created"));
}
