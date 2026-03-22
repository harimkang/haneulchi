use std::sync::atomic::{AtomicU64, Ordering};

use hc_domain::{
    Task, TaskAutomationMode, TaskBoardColumnProjection, TaskClaimLifecycleState, TaskColumn,
    TaskDrawerProjection, project_claim_state,
};
use rusqlite::{Connection, OptionalExtension, params};

use crate::StorageError;
use crate::reviews::ReviewRepository;
use crate::timeline::{NewTimelineEvent, append_event};

static CLAIM_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewTaskRecord {
    pub id: String,
    pub project_id: String,
    pub display_key: String,
    pub title: String,
    pub description: String,
    pub priority: String,
    pub automation_mode: TaskAutomationMode,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskUpdatePatch {
    pub task_id: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub priority: Option<String>,
    pub automation_mode: Option<TaskAutomationMode>,
    pub updated_at: String,
}

pub struct TaskRepository<'connection> {
    connection: &'connection Connection,
}

impl<'connection> TaskRepository<'connection> {
    pub(crate) fn new(connection: &'connection Connection) -> Self {
        Self { connection }
    }

    pub fn create(&self, input: NewTaskRecord) -> Result<Task, StorageError> {
        let task = Task::new(
            input.id,
            input.project_id,
            input.display_key,
            input.title,
            input.description,
            input.priority,
            input.automation_mode,
            input.created_at,
            input.updated_at,
        );

        self.connection.execute(
            r#"
            INSERT INTO tasks (
                id,
                project_id,
                display_key,
                title,
                description,
                column_name,
                priority,
                automation_mode,
                tracker_binding_state,
                linked_session_id,
                linked_worktree_id,
                latest_review_id,
                created_at,
                updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, NULL, NULL, NULL, ?10, ?11)
            "#,
            params![
                task.id,
                task.project_id,
                task.display_key,
                task.title,
                task.description,
                task.column.as_str(),
                task.priority,
                task.automation_mode.as_str(),
                task.tracker_binding_state,
                task.created_at,
                task.updated_at,
            ],
        )?;

        append_event(
            self.connection,
            NewTimelineEvent {
                task_id: task.id.clone(),
                session_id: None,
                review_item_id: None,
                worktree_id: None,
                kind: hc_domain::TimelineEventKind::TaskCreated,
                actor: "task_repository".to_string(),
                reason_code: Some("task_created".to_string()),
                payload_json: None,
                created_at: task.created_at.clone(),
            },
        )?;

        Ok(task)
    }

    pub fn update(&self, patch: TaskUpdatePatch) -> Result<Task, StorageError> {
        let mut task = self
            .get(&patch.task_id)?
            .ok_or_else(|| StorageError::TaskNotFound(patch.task_id.clone()))?;

        if let Some(title) = patch.title {
            task.title = title;
        }
        if let Some(description) = patch.description {
            task.description = description;
        }
        if let Some(priority) = patch.priority {
            task.priority = priority;
        }
        if let Some(automation_mode) = patch.automation_mode {
            task.automation_mode = automation_mode;
        }
        task.updated_at = patch.updated_at;

        self.connection.execute(
            r#"
            UPDATE tasks
            SET title = ?2,
                description = ?3,
                priority = ?4,
                automation_mode = ?5,
                updated_at = ?6
            WHERE id = ?1
            "#,
            params![
                task.id,
                task.title,
                task.description,
                task.priority,
                task.automation_mode.as_str(),
                task.updated_at,
            ],
        )?;

        Ok(task)
    }

    pub fn move_to(
        &self,
        task_id: &str,
        column: TaskColumn,
        actor: &str,
        updated_at: &str,
    ) -> Result<Task, StorageError> {
        let mut task = self
            .get(task_id)?
            .ok_or_else(|| StorageError::TaskNotFound(task_id.to_string()))?;
        task.column = column;
        task.updated_at = updated_at.to_string();

        self.connection.execute(
            "UPDATE tasks SET column_name = ?2, updated_at = ?3 WHERE id = ?1",
            params![task.id, task.column.as_str(), task.updated_at],
        )?;

        append_event(
            self.connection,
            NewTimelineEvent {
                task_id: task.id.clone(),
                session_id: None,
                review_item_id: None,
                worktree_id: None,
                kind: hc_domain::TimelineEventKind::TaskMoved,
                actor: actor.to_string(),
                reason_code: Some("task_moved".to_string()),
                payload_json: Some(format!(r#"{{"column":"{}"}}"#, task.column.as_str())),
                created_at: task.updated_at.clone(),
            },
        )?;

        Ok(task)
    }

    pub fn attach_session(
        &self,
        task_id: &str,
        session_id: &str,
        updated_at: &str,
    ) -> Result<Task, StorageError> {
        let mut task = self
            .get(task_id)?
            .ok_or_else(|| StorageError::TaskNotFound(task_id.to_string()))?;
        task.linked_session_id = Some(session_id.to_string());
        task.updated_at = updated_at.to_string();

        self.connection.execute(
            "UPDATE tasks SET linked_session_id = ?2, updated_at = ?3 WHERE id = ?1",
            params![task.id, task.linked_session_id, task.updated_at],
        )?;
        append_claim_row(
            self.connection,
            task_id,
            TaskClaimLifecycleState::Claimed,
            updated_at,
            Some(session_id),
            "session_attached",
        )?;
        append_event(
            self.connection,
            NewTimelineEvent {
                task_id: task.id.clone(),
                session_id: Some(session_id.to_string()),
                review_item_id: None,
                worktree_id: None,
                kind: hc_domain::TimelineEventKind::TaskAttached,
                actor: "control_plane".to_string(),
                reason_code: Some("task_attached".to_string()),
                payload_json: None,
                created_at: task.updated_at.clone(),
            },
        )?;

        Ok(task)
    }

    pub fn detach_session(&self, task_id: &str, updated_at: &str) -> Result<Task, StorageError> {
        let mut task = self
            .get(task_id)?
            .ok_or_else(|| StorageError::TaskNotFound(task_id.to_string()))?;
        task.linked_session_id = None;
        task.updated_at = updated_at.to_string();

        self.connection.execute(
            "UPDATE tasks SET linked_session_id = NULL, updated_at = ?2 WHERE id = ?1",
            params![task.id, task.updated_at],
        )?;
        append_claim_row(
            self.connection,
            task_id,
            TaskClaimLifecycleState::Released,
            updated_at,
            None,
            "session_detached",
        )?;
        append_event(
            self.connection,
            NewTimelineEvent {
                task_id: task.id.clone(),
                session_id: None,
                review_item_id: None,
                worktree_id: None,
                kind: hc_domain::TimelineEventKind::TaskDetached,
                actor: "control_plane".to_string(),
                reason_code: Some("task_detached".to_string()),
                payload_json: None,
                created_at: task.updated_at.clone(),
            },
        )?;

        Ok(task)
    }

    pub fn get(&self, task_id: &str) -> Result<Option<Task>, StorageError> {
        let mut statement = self.connection.prepare(
            r#"
            SELECT
                id,
                project_id,
                display_key,
                title,
                description,
                column_name,
                priority,
                automation_mode,
                tracker_binding_state,
                linked_session_id,
                linked_worktree_id,
                latest_review_id,
                created_at,
                updated_at
            FROM tasks
            WHERE id = ?1
            "#,
        )?;
        let mut rows = statement.query(params![task_id])?;

        match rows.next()? {
            Some(row) => Ok(Some(task_from_row(row)?)),
            None => Ok(None),
        }
    }

    pub fn board(
        &self,
        project_id: Option<&str>,
    ) -> Result<Vec<TaskBoardColumnProjection>, StorageError> {
        let query = if project_id.is_some() {
            r#"
            SELECT
                id,
                project_id,
                display_key,
                title,
                description,
                column_name,
                priority,
                automation_mode,
                tracker_binding_state,
                linked_session_id,
                linked_worktree_id,
                latest_review_id,
                created_at,
                updated_at
            FROM tasks
            WHERE project_id = ?1
            ORDER BY updated_at DESC, created_at DESC, id ASC
            "#
        } else {
            r#"
            SELECT
                id,
                project_id,
                display_key,
                title,
                description,
                column_name,
                priority,
                automation_mode,
                tracker_binding_state,
                linked_session_id,
                linked_worktree_id,
                latest_review_id,
                created_at,
                updated_at
            FROM tasks
            ORDER BY updated_at DESC, created_at DESC, id ASC
            "#
        };

        let mut statement = self.connection.prepare(query)?;
        let mut rows = if let Some(project_id) = project_id {
            statement.query(params![project_id])?
        } else {
            statement.query([])?
        };

        let mut columns = TaskColumn::all()
            .into_iter()
            .map(|column| TaskBoardColumnProjection {
                column,
                tasks: Vec::new(),
            })
            .collect::<Vec<_>>();

        while let Some(row) = rows.next()? {
            let task = task_from_row(row)?;
            if let Some(column) = columns.iter_mut().find(|entry| entry.column == task.column) {
                column.tasks.push(task);
            }
        }

        Ok(columns)
    }

    pub fn drawer(&self, task_id: &str) -> Result<Option<TaskDrawerProjection>, StorageError> {
        let Some(task) = self.get(task_id)? else {
            return Ok(None);
        };

        let (claim_state, has_live_owner) = latest_claim_state(self.connection, task_id)?;
        let latest_review = ReviewRepository::new(self.connection).latest_for_task(task_id)?;

        Ok(Some(TaskDrawerProjection {
            task,
            claim_state: project_claim_state(claim_state, has_live_owner),
            latest_review,
        }))
    }
}

fn latest_claim_state(
    connection: &Connection,
    task_id: &str,
) -> Result<(Option<TaskClaimLifecycleState>, bool), StorageError> {
    let mut statement = connection.prepare(
        r#"
        SELECT state, session_id
        FROM task_claims
        WHERE task_id = ?1
        ORDER BY claimed_at DESC, id DESC
        LIMIT 1
        "#,
    )?;

    statement
        .query_row(params![task_id], |row| {
            Ok((
                row.get::<_, String>("state")?,
                row.get::<_, Option<String>>("session_id")?,
            ))
        })
        .optional()?
        .map(|(state, session_id)| {
            Ok((
                Some(
                    state
                        .parse()
                        .map_err(StorageError::UnknownTaskClaimLifecycleState)?,
                ),
                session_id.is_some(),
            ))
        })
        .transpose()
        .map(|value| value.unwrap_or((None, false)))
}

fn task_from_row(row: &rusqlite::Row<'_>) -> Result<Task, StorageError> {
    let column = row.get::<_, String>("column_name")?;
    let automation_mode = row.get::<_, String>("automation_mode")?;

    Ok(Task {
        id: row.get("id")?,
        project_id: row.get("project_id")?,
        display_key: row.get("display_key")?,
        title: row.get("title")?,
        description: row.get("description")?,
        column: column.parse().map_err(StorageError::UnknownTaskColumn)?,
        priority: row.get("priority")?,
        automation_mode: automation_mode
            .parse()
            .map_err(StorageError::UnknownTaskAutomationMode)?,
        tracker_binding_state: row.get("tracker_binding_state")?,
        linked_session_id: row.get("linked_session_id")?,
        linked_worktree_id: row.get("linked_worktree_id")?,
        latest_review_id: row.get("latest_review_id")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

fn append_claim_row(
    connection: &Connection,
    task_id: &str,
    state: TaskClaimLifecycleState,
    claimed_at: &str,
    session_id: Option<&str>,
    reason: &str,
) -> Result<(), StorageError> {
    let released_at = match state {
        TaskClaimLifecycleState::Released | TaskClaimLifecycleState::Terminal => Some(claimed_at),
        _ => None,
    };

    connection.execute(
        r#"
        INSERT INTO task_claims (
            id,
            task_id,
            state,
            claimed_at,
            released_at,
            session_id,
            reason
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        "#,
        params![
            next_claim_id(),
            task_id,
            state.as_str(),
            claimed_at,
            released_at,
            session_id,
            reason,
        ],
    )?;

    Ok(())
}

fn next_claim_id() -> String {
    format!(
        "claim_{:016x}",
        CLAIM_COUNTER.fetch_add(1, Ordering::Relaxed)
    )
}
