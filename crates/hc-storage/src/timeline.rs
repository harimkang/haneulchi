use std::sync::atomic::{AtomicU64, Ordering};

use hc_domain::{TimelineEvent, TimelineEventKind};
use rusqlite::{Connection, params};

use crate::StorageError;

static EVENT_COUNTER: AtomicU64 = AtomicU64::new(1);

pub(crate) struct NewTimelineEvent {
    pub task_id: String,
    pub session_id: Option<String>,
    pub review_item_id: Option<String>,
    pub worktree_id: Option<String>,
    pub kind: TimelineEventKind,
    pub actor: String,
    pub reason_code: Option<String>,
    pub payload_json: Option<String>,
    pub created_at: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppendTimelineEvent {
    pub task_id: String,
    pub session_id: Option<String>,
    pub review_item_id: Option<String>,
    pub worktree_id: Option<String>,
    pub kind: TimelineEventKind,
    pub actor: String,
    pub reason_code: Option<String>,
    pub payload_json: Option<String>,
    pub created_at: String,
}

pub struct TimelineRepository<'connection> {
    connection: &'connection Connection,
}

impl<'connection> TimelineRepository<'connection> {
    pub(crate) fn new(connection: &'connection Connection) -> Self {
        Self { connection }
    }

    pub fn list_for_task(&self, task_id: &str) -> Result<Vec<TimelineEvent>, StorageError> {
        let mut statement = self.connection.prepare(
            r#"
            SELECT
                id,
                task_id,
                session_id,
                review_item_id,
                worktree_id,
                kind,
                actor,
                reason_code,
                payload_json,
                created_at
            FROM task_events
            WHERE task_id = ?1
            ORDER BY created_at ASC, id ASC
            "#,
        )?;
        let mut rows = statement.query(params![task_id])?;
        let mut events = Vec::new();

        while let Some(row) = rows.next()? {
            events.push(event_from_row(row)?);
        }

        Ok(events)
    }

    pub fn append(&self, event: AppendTimelineEvent) -> Result<(), StorageError> {
        append_event(
            self.connection,
            NewTimelineEvent {
                task_id: event.task_id,
                session_id: event.session_id,
                review_item_id: event.review_item_id,
                worktree_id: event.worktree_id,
                kind: event.kind,
                actor: event.actor,
                reason_code: event.reason_code,
                payload_json: event.payload_json,
                created_at: event.created_at,
            },
        )
    }
}

pub(crate) fn append_event(
    connection: &Connection,
    event: NewTimelineEvent,
) -> Result<(), StorageError> {
    connection.execute(
        r#"
        INSERT INTO task_events (
            id,
            task_id,
            session_id,
            review_item_id,
            worktree_id,
            kind,
            actor,
            reason_code,
            payload_json,
            created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
        "#,
        params![
            next_event_id(),
            event.task_id,
            event.session_id,
            event.review_item_id,
            event.worktree_id,
            event.kind.as_str(),
            event.actor,
            event.reason_code,
            event.payload_json,
            event.created_at,
        ],
    )?;

    Ok(())
}

fn event_from_row(row: &rusqlite::Row<'_>) -> Result<TimelineEvent, StorageError> {
    let kind = row.get::<_, String>("kind")?;

    Ok(TimelineEvent {
        id: row.get("id")?,
        task_id: row.get("task_id")?,
        session_id: row.get("session_id")?,
        review_item_id: row.get("review_item_id")?,
        worktree_id: row.get("worktree_id")?,
        kind: kind
            .parse()
            .map_err(StorageError::UnknownTimelineEventKind)?,
        actor: row.get("actor")?,
        reason_code: row.get("reason_code")?,
        payload_json: row.get("payload_json")?,
        created_at: row.get("created_at")?,
    })
}

fn next_event_id() -> String {
    format!("evt_{:016x}", EVENT_COUNTER.fetch_add(1, Ordering::Relaxed))
}
