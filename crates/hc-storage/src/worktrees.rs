use hc_domain::TimelineEventKind;
use rusqlite::{Connection, OptionalExtension, params};

use crate::StorageError;
use crate::timeline::{NewTimelineEvent, append_event};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewWorktreeRecord {
    pub id: String,
    pub task_id: String,
    pub project_id: String,
    pub workspace_root: String,
    pub base_root: String,
    pub branch_name: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorktreeRecord {
    pub id: String,
    pub task_id: String,
    pub project_id: String,
    pub workspace_root: String,
    pub base_root: String,
    pub branch_name: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

pub struct WorktreeRepository<'connection> {
    connection: &'connection Connection,
}

impl<'connection> WorktreeRepository<'connection> {
    pub(crate) fn new(connection: &'connection Connection) -> Self {
        Self { connection }
    }

    pub fn create_or_replace(
        &self,
        input: NewWorktreeRecord,
    ) -> Result<WorktreeRecord, StorageError> {
        self.connection.execute(
            r#"
            INSERT INTO worktrees (
                id,
                task_id,
                project_id,
                workspace_root,
                base_root,
                branch_name,
                status,
                created_at,
                updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            ON CONFLICT(task_id) DO UPDATE SET
                id = excluded.id,
                project_id = excluded.project_id,
                workspace_root = excluded.workspace_root,
                base_root = excluded.base_root,
                branch_name = excluded.branch_name,
                status = excluded.status,
                updated_at = excluded.updated_at
            "#,
            params![
                input.id,
                input.task_id,
                input.project_id,
                input.workspace_root,
                input.base_root,
                input.branch_name,
                input.status,
                input.created_at,
                input.updated_at,
            ],
        )?;

        self.connection.execute(
            "UPDATE tasks SET linked_worktree_id = ?2, updated_at = ?3 WHERE id = ?1",
            params![input.task_id, input.id, input.updated_at],
        )?;

        append_event(
            self.connection,
            NewTimelineEvent {
                task_id: input.task_id.clone(),
                session_id: None,
                review_item_id: None,
                worktree_id: Some(input.id.clone()),
                kind: TimelineEventKind::WorktreeCreated,
                actor: "worktree_repository".to_string(),
                reason_code: Some("worktree_created".to_string()),
                payload_json: Some(format!(
                    r#"{{"workspace_root":"{}","branch_name":"{}"}}"#,
                    input.workspace_root, input.branch_name
                )),
                created_at: input.updated_at.clone(),
            },
        )?;

        self.find_by_task(&input.task_id)?
            .ok_or_else(|| StorageError::WorktreeNotFound(input.task_id))
    }

    pub fn find_by_task(&self, task_id: &str) -> Result<Option<WorktreeRecord>, StorageError> {
        let mut statement = self.connection.prepare(
            r#"
            SELECT
                id,
                task_id,
                project_id,
                workspace_root,
                base_root,
                branch_name,
                status,
                created_at,
                updated_at
            FROM worktrees
            WHERE task_id = ?1
            ORDER BY updated_at DESC, id DESC
            LIMIT 1
            "#,
        )?;

        statement
            .query_row(params![task_id], |row| {
                Ok(WorktreeRecord {
                    id: row.get("id")?,
                    task_id: row.get("task_id")?,
                    project_id: row.get("project_id")?,
                    workspace_root: row.get("workspace_root")?,
                    base_root: row.get("base_root")?,
                    branch_name: row.get("branch_name")?,
                    status: row.get("status")?,
                    created_at: row.get("created_at")?,
                    updated_at: row.get("updated_at")?,
                })
            })
            .optional()
            .map_err(Into::into)
    }
}
