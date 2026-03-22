use hc_domain::{ClaimState, ReviewStatus, TaskAutomationMode, TaskColumn, TimelineEventKind};
use hc_storage::{NewReviewItem, NewTaskRecord, SqliteStore, TaskUpdatePatch};

#[test]
fn sqlite_task_repository_persists_crud_board_projection_and_append_only_events() {
    let store = SqliteStore::in_memory().expect("sqlite store");

    let created = store
        .tasks()
        .create(NewTaskRecord {
            id: "task_01".to_string(),
            project_id: "proj_demo".to_string(),
            display_key: "TASK-1".to_string(),
            title: "Add task CRUD".to_string(),
            description: "Implement the local task repository".to_string(),
            priority: "p1".to_string(),
            automation_mode: TaskAutomationMode::Assisted,
            created_at: "2026-03-23T00:00:00Z".to_string(),
            updated_at: "2026-03-23T00:00:00Z".to_string(),
        })
        .expect("task row");

    assert_eq!(created.column, TaskColumn::Inbox);
    assert_eq!(created.automation_mode, TaskAutomationMode::Assisted);

    let updated = store
        .tasks()
        .update(TaskUpdatePatch {
            task_id: "task_01".to_string(),
            title: Some("Add task CRUD baseline".to_string()),
            description: Some("Persist tasks and board projections in sqlite".to_string()),
            priority: Some("p0".to_string()),
            automation_mode: Some(TaskAutomationMode::AutoEligible),
            updated_at: "2026-03-23T00:05:00Z".to_string(),
        })
        .expect("task update");

    assert_eq!(updated.title, "Add task CRUD baseline");
    assert_eq!(updated.priority, "p0");
    assert_eq!(updated.automation_mode, TaskAutomationMode::AutoEligible);

    let moved = store
        .tasks()
        .move_to(
            "task_01",
            TaskColumn::Ready,
            "operator_drag",
            "2026-03-23T00:10:00Z",
        )
        .expect("task move");

    assert_eq!(moved.column, TaskColumn::Ready);

    let board = store
        .tasks()
        .board(Some("proj_demo"))
        .expect("board projection");
    assert_eq!(
        board.iter().map(|column| column.column).collect::<Vec<_>>(),
        vec![
            TaskColumn::Inbox,
            TaskColumn::Ready,
            TaskColumn::Running,
            TaskColumn::Review,
            TaskColumn::Blocked,
            TaskColumn::Done,
        ]
    );
    assert!(board[0].tasks.is_empty());
    assert_eq!(board[1].tasks.len(), 1);
    assert_eq!(board[1].tasks[0].id, "task_01");

    let drawer = store
        .tasks()
        .drawer("task_01")
        .expect("drawer projection")
        .expect("task drawer");
    assert_eq!(drawer.task.id, "task_01");
    assert_eq!(drawer.claim_state, ClaimState::None);

    let events = store
        .timeline()
        .list_for_task("task_01")
        .expect("timeline rows");
    assert_eq!(
        events.iter().map(|event| event.kind).collect::<Vec<_>>(),
        vec![TimelineEventKind::TaskCreated, TimelineEventKind::TaskMoved,]
    );
}

#[test]
fn sqlite_review_repository_scaffold_persists_pending_review_items() {
    let store = SqliteStore::in_memory().expect("sqlite store");
    store
        .tasks()
        .create(NewTaskRecord {
            id: "task_02".to_string(),
            project_id: "proj_demo".to_string(),
            display_key: "TASK-2".to_string(),
            title: "Review item scaffold".to_string(),
            description: "Persist pending review rows".to_string(),
            priority: "p2".to_string(),
            automation_mode: TaskAutomationMode::Manual,
            created_at: "2026-03-23T01:00:00Z".to_string(),
            updated_at: "2026-03-23T01:00:00Z".to_string(),
        })
        .expect("task row");

    let review = store
        .reviews()
        .create_pending(NewReviewItem {
            id: "review_01".to_string(),
            task_id: "task_02".to_string(),
            session_id: Some("ses_01".to_string()),
            summary: "Ready for manual review".to_string(),
            created_at: "2026-03-23T01:05:00Z".to_string(),
        })
        .expect("review row");

    assert_eq!(review.status, ReviewStatus::Pending);
    assert_eq!(
        store
            .reviews()
            .list_for_task("task_02")
            .expect("review rows"),
        vec![review]
    );
}
