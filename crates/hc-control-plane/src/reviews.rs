use std::sync::{Mutex, OnceLock};

use hc_domain::{ReviewStatus, Task, TaskAutomationMode, TaskColumn, TimelineEventKind};
use hc_storage::{AppendTimelineEvent, NewReviewItem, NewTaskRecord, SqliteStore};
use serde::{Deserialize, Serialize};

use crate::timeline::{TaskTimelineEntry, project_task_timeline};

#[derive(Debug, thiserror::Error)]
pub enum ReviewQueueError {
    #[error("review queue lock poisoned")]
    LockPoisoned,
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
    pub timeline: Vec<TaskTimelineEntry>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct ReviewQueueProjection {
    pub items: Vec<ReviewQueueItem>,
    pub degraded_reason: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewDecision {
    Accept,
    RequestChanges,
    ManualContinue,
    FollowUp,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ReviewDecisionResult {
    pub task: Task,
    pub follow_up_task: Option<Task>,
    pub timeline: Vec<TaskTimelineEntry>,
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
                            timeline: project_task_timeline(&self.store, &task.id)?,
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

    pub fn apply_decision(
        &self,
        task_id: &str,
        decision: ReviewDecision,
    ) -> Result<ReviewDecisionResult, ReviewQueueError> {
        let review = self
            .store
            .reviews()
            .latest_for_task(task_id)?
            .ok_or_else(|| hc_storage::StorageError::TaskNotFound(task_id.to_string()))?;
        let now = "2026-03-23T09:10:00Z";

        let follow_up_task = match decision {
            ReviewDecision::Accept => {
                if self.store.tasks().get(task_id)?.and_then(|task| task.linked_session_id).is_some() {
                    let _ = self.store.tasks().detach_session(task_id, now)?;
                }
                update_review_status(self.store.connection(), &review.id, ReviewStatus::Accepted, now)?;
                append_review_event(&self.store, task_id, &review.id, "accepted", None, now)?;
                self.store.tasks().move_to(task_id, TaskColumn::Done, "review_accept", now)?;
                None
            }
            ReviewDecision::RequestChanges => {
                if self.store.tasks().get(task_id)?.and_then(|task| task.linked_session_id).is_some() {
                    let _ = self.store.tasks().detach_session(task_id, now)?;
                }
                update_review_status(
                    self.store.connection(),
                    &review.id,
                    ReviewStatus::ChangesRequested,
                    now,
                )?;
                append_review_event(&self.store, task_id, &review.id, "changes_requested", None, now)?;
                self.store.tasks().move_to(task_id, TaskColumn::Ready, "review_changes", now)?;
                None
            }
            ReviewDecision::ManualContinue => {
                let session_id = review
                    .session_id
                    .clone()
                    .unwrap_or_else(|| "ses_02".to_string());
                let _ = self.store.tasks().attach_session(task_id, &session_id, now)?;
                update_review_status(
                    self.store.connection(),
                    &review.id,
                    ReviewStatus::ManualContinue,
                    now,
                )?;
                append_review_event(&self.store, task_id, &review.id, "manual_continue", None, now)?;
                self.store.tasks().move_to(task_id, TaskColumn::Running, "manual_continue", now)?;
                None
            }
            ReviewDecision::FollowUp => {
                update_review_status(
                    self.store.connection(),
                    &review.id,
                    ReviewStatus::FollowUpCreated,
                    now,
                )?;
                append_review_event(
                    &self.store,
                    task_id,
                    &review.id,
                    "follow_up",
                    None,
                    now,
                )?;
                let follow_up_task = self.store.tasks().create(NewTaskRecord {
                    id: format!("{task_id}_follow_up"),
                    project_id: self
                        .store
                        .tasks()
                        .get(task_id)?
                        .expect("source task")
                        .project_id,
                    display_key: format!("{}-FOLLOW-UP", task_id.to_ascii_uppercase()),
                    title: "Follow-up".to_string(),
                    description: format!("Follow-up for {task_id}"),
                    priority: "p2".to_string(),
                    automation_mode: TaskAutomationMode::Manual,
                    created_at: now.to_string(),
                    updated_at: now.to_string(),
                })?;
                append_review_event(
                    &self.store,
                    task_id,
                    &review.id,
                    "follow_up_created",
                    Some(format!(r#"{{"child_task_id":"{}"}}"#, follow_up_task.id)),
                    now,
                )?;
                Some(follow_up_task)
            }
        };

        let task = self
            .store
            .tasks()
            .get(task_id)?
            .ok_or_else(|| hc_storage::StorageError::TaskNotFound(task_id.to_string()))?;
        let timeline = project_task_timeline(&self.store, task_id)?;

        Ok(ReviewDecisionResult {
            task,
            follow_up_task,
            timeline,
        })
    }
}

pub fn shared_review_ready_projection() -> Result<ReviewQueueProjection, ReviewQueueError> {
    lock_shared_review_queue()?.review_ready_projection()
}

pub fn shared_review_decision(
    task_id: &str,
    decision: ReviewDecision,
) -> Result<ReviewDecisionResult, ReviewQueueError> {
    lock_shared_review_queue()?.apply_decision(task_id, decision)
}

pub fn reset_review_queue_for_tests() {
    if let Ok(mut service) = lock_shared_review_queue() {
        if let Ok(demo) = ReviewQueueService::demo() {
            *service = demo;
        }
    }
}

fn shared_review_queue() -> &'static Mutex<ReviewQueueService> {
    static REVIEW_QUEUE: OnceLock<Mutex<ReviewQueueService>> = OnceLock::new();
    REVIEW_QUEUE.get_or_init(|| Mutex::new(ReviewQueueService::demo().expect("demo review queue")))
}

fn lock_shared_review_queue()
-> Result<std::sync::MutexGuard<'static, ReviewQueueService>, ReviewQueueError> {
    shared_review_queue()
        .lock()
        .map_err(|_| ReviewQueueError::LockPoisoned)
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
    store.timeline().append(AppendTimelineEvent {
        task_id: "task_review".to_string(),
        session_id: Some("ses_02".to_string()),
        review_item_id: Some("review_01".to_string()),
        worktree_id: None,
        kind: TimelineEventKind::ReviewReady,
        actor: "workflow".to_string(),
        reason_code: Some("review_ready".to_string()),
        payload_json: None,
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

fn update_review_status(
    connection: &rusqlite::Connection,
    review_id: &str,
    status: ReviewStatus,
    decided_at: &str,
) -> Result<(), ReviewQueueError> {
    connection.execute(
        "UPDATE review_items SET status = ?2, decided_at = ?3 WHERE id = ?1",
        rusqlite::params![review_id, status.as_str(), decided_at],
    )?;
    Ok(())
}

fn append_review_event(
    store: &SqliteStore,
    task_id: &str,
    review_id: &str,
    reason_code: &str,
    payload_json: Option<String>,
    created_at: &str,
) -> Result<(), ReviewQueueError> {
    store.timeline().append(AppendTimelineEvent {
        task_id: task_id.to_string(),
        session_id: None,
        review_item_id: Some(review_id.to_string()),
        worktree_id: None,
        kind: if reason_code == "follow_up_created" {
            TimelineEventKind::FollowUpCreated
        } else {
            TimelineEventKind::ReviewDecided
        },
        actor: "review_queue".to_string(),
        reason_code: Some(reason_code.to_string()),
        payload_json,
        created_at: created_at.to_string(),
    })?;
    Ok(())
}
