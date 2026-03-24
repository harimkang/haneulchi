use hc_domain::{ClaimState, RetryQueueEntry, RetryState};
use rusqlite::{Connection, params};

use crate::StorageError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetryFailureClass {
    Retryable,
    NonRetryable,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewRetryQueueEntry {
    pub id: String,
    pub task_id: String,
    pub project_id: String,
    pub attempt: u32,
    pub reason_code: String,
    pub due_at: Option<String>,
    pub backoff_ms: u64,
    pub claim_state: ClaimState,
    pub retry_state: RetryState,
}

pub struct RetryQueueRepository<'connection> {
    connection: &'connection Connection,
}

impl<'connection> RetryQueueRepository<'connection> {
    pub(crate) fn new(connection: &'connection Connection) -> Self {
        Self { connection }
    }

    pub fn save(&self, entry: NewRetryQueueEntry) -> Result<RetryQueueEntry, StorageError> {
        self.connection.execute(
            r#"
            INSERT INTO retry_queue (
                id,
                task_id,
                project_id,
                attempt,
                reason_code,
                due_at,
                backoff_ms,
                claim_state,
                retry_state
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            ON CONFLICT(id) DO UPDATE SET
                task_id = excluded.task_id,
                project_id = excluded.project_id,
                attempt = excluded.attempt,
                reason_code = excluded.reason_code,
                due_at = excluded.due_at,
                backoff_ms = excluded.backoff_ms,
                claim_state = excluded.claim_state,
                retry_state = excluded.retry_state
            "#,
            params![
                &entry.id,
                &entry.task_id,
                &entry.project_id,
                entry.attempt,
                &entry.reason_code,
                &entry.due_at,
                entry.backoff_ms as i64,
                entry.claim_state.as_str(),
                entry.retry_state.as_str(),
            ],
        )?;

        Ok(RetryQueueEntry {
            task_id: entry.task_id,
            project_id: entry.project_id,
            attempt: entry.attempt,
            reason_code: entry.reason_code,
            due_at: entry.due_at,
            backoff_ms: entry.backoff_ms,
            claim_state: entry.claim_state,
            retry_state: entry.retry_state,
        })
    }

    pub fn list(&self) -> Result<Vec<RetryQueueEntry>, StorageError> {
        let mut statement = self.connection.prepare(
            r#"
            SELECT
                task_id,
                project_id,
                attempt,
                reason_code,
                due_at,
                backoff_ms,
                claim_state,
                retry_state
            FROM retry_queue
            ORDER BY due_at ASC, id ASC
            "#,
        )?;
        let mut rows = statement.query([])?;
        let mut entries = Vec::new();

        while let Some(row) = rows.next()? {
            let claim_state = row.get::<_, String>("claim_state")?;
            let retry_state = row.get::<_, String>("retry_state")?;
            let backoff_ms = row.get::<_, i64>("backoff_ms")?;

            entries.push(RetryQueueEntry {
                task_id: row.get("task_id")?,
                project_id: row.get("project_id")?,
                attempt: row.get("attempt")?,
                reason_code: row.get("reason_code")?,
                due_at: row.get("due_at")?,
                backoff_ms: backoff_ms as u64,
                claim_state: claim_state
                    .parse()
                    .map_err(StorageError::UnknownClaimState)?,
                retry_state: retry_state
                    .parse()
                    .map_err(StorageError::UnknownClaimState)?,
            });
        }

        Ok(entries)
    }
}

pub fn schedule_retry_entry(
    task_id: &str,
    project_id: &str,
    attempt: u32,
    now_ms: u64,
    failure_class: RetryFailureClass,
    claim_state: ClaimState,
) -> Option<RetryQueueEntry> {
    if failure_class == RetryFailureClass::NonRetryable {
        return None;
    }

    let backoff_ms = 30_000_u64.saturating_mul(2_u64.saturating_pow(attempt.saturating_sub(1)));
    Some(RetryQueueEntry {
        task_id: task_id.to_string(),
        project_id: project_id.to_string(),
        attempt,
        reason_code: "retryable_failure".to_string(),
        due_at: Some((now_ms + backoff_ms).to_string()),
        backoff_ms,
        claim_state,
        retry_state: RetryState::BackingOff,
    })
}

pub fn advance_retry_state(entry: &RetryQueueEntry, now_ms: u64, stalled: bool) -> RetryQueueEntry {
    let mut updated = entry.clone();
    updated.retry_state = if stalled {
        RetryState::Exhausted
    } else if updated
        .due_at
        .as_deref()
        .and_then(|value| value.parse::<u64>().ok())
        .is_some_and(|due_at| now_ms >= due_at)
    {
        RetryState::Due
    } else {
        RetryState::BackingOff
    };
    updated
}
