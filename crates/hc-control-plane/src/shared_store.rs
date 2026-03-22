use std::sync::{Mutex, OnceLock};

use hc_domain::{TaskAutomationMode, TaskColumn};
use hc_storage::{NewReviewItem, NewTaskRecord, SqliteStore};

use crate::reviews::ReviewQueueError;
use crate::tasks::TaskBoardError;

pub(crate) fn lock_shared_store() -> Result<std::sync::MutexGuard<'static, SqliteStore>, TaskBoardError> {
    shared_store().lock().map_err(|_| TaskBoardError::LockPoisoned)
}

pub(crate) fn lock_shared_store_for_reviews(
) -> Result<std::sync::MutexGuard<'static, SqliteStore>, ReviewQueueError> {
    shared_store().lock().map_err(|_| ReviewQueueError::LockPoisoned)
}

pub(crate) fn reset_shared_store() {
    if let Ok(mut guard) = shared_store().lock() {
        *guard = build_seeded_store().expect("seeded control-plane store");
    }
}

fn shared_store() -> &'static Mutex<SqliteStore> {
    static STORE: OnceLock<Mutex<SqliteStore>> = OnceLock::new();
    STORE.get_or_init(|| Mutex::new(build_seeded_store().expect("seeded control-plane store")))
}

fn build_seeded_store() -> Result<SqliteStore, hc_storage::StorageError> {
    let store = SqliteStore::in_memory()?;

    store.tasks().create(NewTaskRecord {
        id: "task_inbox".to_string(),
        project_id: "proj_demo".to_string(),
        display_key: "TASK-INBOX".to_string(),
        title: "Task draft".to_string(),
        description: "Inbox task for board filtering".to_string(),
        priority: "p2".to_string(),
        automation_mode: TaskAutomationMode::Manual,
        created_at: "2026-03-23T02:00:00Z".to_string(),
        updated_at: "2026-03-23T02:00:00Z".to_string(),
    })?;
    store.tasks().create(NewTaskRecord {
        id: "task_ready".to_string(),
        project_id: "proj_demo".to_string(),
        display_key: "TASK-READY".to_string(),
        title: "Ready handoff".to_string(),
        description: "Task ready to move across the board".to_string(),
        priority: "p1".to_string(),
        automation_mode: TaskAutomationMode::Assisted,
        created_at: "2026-03-23T02:00:00Z".to_string(),
        updated_at: "2026-03-23T02:00:00Z".to_string(),
    })?;
    store
        .tasks()
        .move_to("task_ready", TaskColumn::Ready, "seed_shared", "2026-03-23T02:05:00Z")?;
    store.tasks().create(NewTaskRecord {
        id: "task_running".to_string(),
        project_id: "proj_alpha".to_string(),
        display_key: "TASK-RUNNING".to_string(),
        title: "Running automation".to_string(),
        description: "Second project task for filter coverage".to_string(),
        priority: "p0".to_string(),
        automation_mode: TaskAutomationMode::AutoEligible,
        created_at: "2026-03-23T02:00:00Z".to_string(),
        updated_at: "2026-03-23T02:00:00Z".to_string(),
    })?;
    store
        .tasks()
        .move_to("task_running", TaskColumn::Running, "seed_shared", "2026-03-23T02:05:00Z")?;

    store.tasks().create(NewTaskRecord {
        id: "task_review".to_string(),
        project_id: "proj_demo".to_string(),
        display_key: "TASK-REVIEW".to_string(),
        title: "Review auth flow".to_string(),
        description: "Pending handoff review".to_string(),
        priority: "p1".to_string(),
        automation_mode: TaskAutomationMode::Assisted,
        created_at: "2026-03-23T09:00:00Z".to_string(),
        updated_at: "2026-03-23T09:00:00Z".to_string(),
    })?;
    store
        .tasks()
        .move_to("task_review", TaskColumn::Review, "seed_shared", "2026-03-23T09:01:00Z")?;
    store.reviews().create_pending(NewReviewItem {
        id: "review_01".to_string(),
        task_id: "task_review".to_string(),
        session_id: Some("ses_02".to_string()),
        summary: "Ready for handoff".to_string(),
        created_at: "2026-03-23T09:02:00Z".to_string(),
    })?;
    store.connection().execute(
        r#"
        UPDATE review_items
        SET touched_files_json = ?2,
            diff_summary = ?3,
            tests_summary = ?4,
            command_summary = ?5,
            warnings_json = ?6,
            evidence_manifest_path = ?7
        WHERE id = ?1
        "#,
        rusqlite::params![
            "review_01",
            serde_json::to_string(&vec!["Sources/Auth.swift", "Tests/AuthTests.swift"])?,
            "+42 -8",
            "12 passing",
            "cargo test -p hc-workflow",
            serde_json::to_string(&vec!["snapshot drift"])?,
            "evidence/reviews/task_review/review_01/manifest.json",
        ],
    )?;
    store.timeline().append(hc_storage::AppendTimelineEvent {
        task_id: "task_review".to_string(),
        session_id: Some("ses_02".to_string()),
        review_item_id: Some("review_01".to_string()),
        worktree_id: None,
        kind: hc_domain::TimelineEventKind::ReviewReady,
        actor: "workflow".to_string(),
        reason_code: Some("review_ready".to_string()),
        payload_json: None,
        created_at: "2026-03-23T09:02:00Z".to_string(),
    })?;

    Ok(store)
}
