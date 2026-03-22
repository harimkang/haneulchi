use hc_control_plane::{ReviewQueueService, TaskBoardColumnSummary, TaskBoardService};
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
