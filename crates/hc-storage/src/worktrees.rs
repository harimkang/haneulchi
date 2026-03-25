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
    pub lifecycle_state: String,
    pub size_bytes: Option<u64>,
    pub is_pinned: bool,
    pub last_accessed_at: Option<String>,
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
                payload_json: Some(
                    serde_json::json!({
                        "workspace_root": input.workspace_root,
                        "branch_name": input.branch_name
                    })
                    .to_string(),
                ),
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
                updated_at,
                lifecycle_state,
                size_bytes,
                is_pinned,
                last_accessed_at
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
                    lifecycle_state: row
                        .get::<_, Option<String>>("lifecycle_state")?
                        .unwrap_or_else(|| "in_use".to_string()),
                    size_bytes: row.get::<_, Option<i64>>("size_bytes")?.map(|v| v as u64),
                    is_pinned: row.get::<_, i32>("is_pinned")? != 0,
                    last_accessed_at: row.get("last_accessed_at")?,
                })
            })
            .optional()
            .map_err(Into::into)
    }

    pub fn update_lifecycle(
        &self,
        worktree_id: &str,
        lifecycle_state: &str,
        updated_at: &str,
    ) -> Result<(), StorageError> {
        self.connection.execute(
            "UPDATE worktrees SET lifecycle_state = ?2, updated_at = ?3 WHERE id = ?1",
            params![worktree_id, lifecycle_state, updated_at],
        )?;

        Ok(())
    }

    pub fn set_pinned(
        &self,
        worktree_id: &str,
        is_pinned: bool,
        updated_at: &str,
    ) -> Result<(), StorageError> {
        self.connection.execute(
            "UPDATE worktrees SET is_pinned = ?2, updated_at = ?3 WHERE id = ?1",
            params![worktree_id, is_pinned as i32, updated_at],
        )?;

        Ok(())
    }

    pub fn list_by_project(&self, project_id: &str) -> Result<Vec<WorktreeRecord>, StorageError> {
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
                updated_at,
                lifecycle_state,
                size_bytes,
                is_pinned,
                last_accessed_at
            FROM worktrees
            WHERE project_id = ?1
            ORDER BY updated_at DESC, id DESC
            "#,
        )?;

        let mut rows = statement.query(params![project_id])?;
        let mut records = Vec::new();

        while let Some(row) = rows.next()? {
            records.push(WorktreeRecord {
                id: row.get("id")?,
                task_id: row.get("task_id")?,
                project_id: row.get("project_id")?,
                workspace_root: row.get("workspace_root")?,
                base_root: row.get("base_root")?,
                branch_name: row.get("branch_name")?,
                status: row.get("status")?,
                created_at: row.get("created_at")?,
                updated_at: row.get("updated_at")?,
                lifecycle_state: row
                    .get::<_, Option<String>>("lifecycle_state")?
                    .unwrap_or_else(|| "in_use".to_string()),
                size_bytes: row.get::<_, Option<i64>>("size_bytes")?.map(|v| v as u64),
                is_pinned: row.get::<_, i32>("is_pinned")? != 0,
                last_accessed_at: row.get("last_accessed_at")?,
            });
        }

        Ok(records)
    }

    /// Returns worktrees with `lifecycle_state = 'stale'` where
    /// `last_accessed_at < stale_before`. Rows with NULL `last_accessed_at`
    /// are excluded because NULL comparisons yield NULL (not true) in SQLite.
    pub fn list_stale(&self, stale_before: &str) -> Result<Vec<WorktreeRecord>, StorageError> {
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
                updated_at,
                lifecycle_state,
                size_bytes,
                is_pinned,
                last_accessed_at
            FROM worktrees
            WHERE lifecycle_state = 'stale'
              AND last_accessed_at < ?1
              AND is_pinned = 0
            ORDER BY last_accessed_at ASC, id ASC
            "#,
        )?;

        let mut rows = statement.query(params![stale_before])?;
        let mut records = Vec::new();

        while let Some(row) = rows.next()? {
            records.push(WorktreeRecord {
                id: row.get("id")?,
                task_id: row.get("task_id")?,
                project_id: row.get("project_id")?,
                workspace_root: row.get("workspace_root")?,
                base_root: row.get("base_root")?,
                branch_name: row.get("branch_name")?,
                status: row.get("status")?,
                created_at: row.get("created_at")?,
                updated_at: row.get("updated_at")?,
                lifecycle_state: row
                    .get::<_, Option<String>>("lifecycle_state")?
                    .unwrap_or_else(|| "in_use".to_string()),
                size_bytes: row.get::<_, Option<i64>>("size_bytes")?.map(|v| v as u64),
                is_pinned: row.get::<_, i32>("is_pinned")? != 0,
                last_accessed_at: row.get("last_accessed_at")?,
            });
        }

        Ok(records)
    }
}
