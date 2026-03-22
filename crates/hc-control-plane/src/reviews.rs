use hc_domain::{TaskAutomationMode, TaskColumn};
use hc_storage::{NewReviewItem, NewTaskRecord, SqliteStore};
use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum ReviewQueueError {
    #[error("storage error: {0}")]
    Storage(#[from] hc_storage::StorageError),
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ReviewQueueItem {
    pub task_id: String,
    pub project_id: String,
    pub title: String,
    pub summary: String,
    pub touched_files: Vec<String>,
    pub diff_summary: Option<String>,
    pub tests_summary: Option<String>,
    pub command_summary: Option<String>,
    pub warnings: Vec<String>,
    pub evidence_manifest_path: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct ReviewQueueProjection {
    pub items: Vec<ReviewQueueItem>,
    pub degraded_reason: Option<String>,
}

pub struct ReviewQueueService {
    store: SqliteStore,
}

impl ReviewQueueService {
    pub fn demo() -> Result<Self, ReviewQueueError> {
        let store = SqliteStore::in_memory()?;
        seed_review_demo(&store)?;
        Ok(Self { store })
    }

    pub fn review_ready_projection(&self) -> Result<ReviewQueueProjection, ReviewQueueError> {
        let board = self.store.tasks().board(None)?;
        let mut items = Vec::new();

        if let Some(review_column) = board.iter().find(|column| column.column == TaskColumn::Review) {
            for task in &review_column.tasks {
                if let Some(review) = self.store.reviews().latest_for_task(&task.id)? {
                    if review.status == hc_domain::ReviewStatus::Pending {
                        items.push(ReviewQueueItem {
                            task_id: task.id.clone(),
                            project_id: task.project_id.clone(),
                            title: task.title.clone(),
                            summary: review.summary,
                            touched_files: review.touched_files,
                            diff_summary: review.diff_summary,
                            tests_summary: review.tests_summary,
                            command_summary: review.command_summary,
                            warnings: review.warnings,
                            evidence_manifest_path: review.evidence_manifest_path,
                        });
                    }
                }
            }
        }

        Ok(ReviewQueueProjection {
            items,
            degraded_reason: None,
        })
    }
}

fn seed_review_demo(store: &SqliteStore) -> Result<(), ReviewQueueError> {
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
        .move_to("task_review", TaskColumn::Review, "seed_review", "2026-03-23T09:01:00Z")?;
    store.reviews().create_pending(NewReviewItem {
        id: "review_01".to_string(),
        task_id: "task_review".to_string(),
        session_id: Some("ses_02".to_string()),
        summary: "Ready for handoff".to_string(),
        created_at: "2026-03-23T09:02:00Z".to_string(),
    })?;

    let connection = store.connection();
    connection.execute(
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

    store.tasks().create(NewTaskRecord {
        id: "task_running".to_string(),
        project_id: "proj_demo".to_string(),
        display_key: "TASK-RUNNING".to_string(),
        title: "Still running".to_string(),
        description: "Should not appear in review queue".to_string(),
        priority: "p2".to_string(),
        automation_mode: TaskAutomationMode::Manual,
        created_at: "2026-03-23T09:03:00Z".to_string(),
        updated_at: "2026-03-23T09:03:00Z".to_string(),
    })?;
    store
        .tasks()
        .move_to("task_running", TaskColumn::Running, "seed_review", "2026-03-23T09:04:00Z")?;

    Ok(())
}
