use hc_domain::TrackerBinding;
use rusqlite::{Connection, params};

use crate::StorageError;

pub struct TrackerBindingRepository<'connection> {
    connection: &'connection Connection,
}

impl<'connection> TrackerBindingRepository<'connection> {
    pub(crate) fn new(connection: &'connection Connection) -> Self {
        Self { connection }
    }

    pub fn upsert(&self, binding: TrackerBinding) -> Result<TrackerBinding, StorageError> {
        self.connection.execute(
            r#"
            INSERT INTO tracker_bindings (
                id,
                task_id,
                provider,
                external_id,
                external_key,
                sync_mode,
                state,
                last_sync_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ON CONFLICT(id) DO UPDATE SET
                task_id = excluded.task_id,
                provider = excluded.provider,
                external_id = excluded.external_id,
                external_key = excluded.external_key,
                sync_mode = excluded.sync_mode,
                state = excluded.state,
                last_sync_at = excluded.last_sync_at
            "#,
            params![
                &binding.id,
                &binding.task_id,
                &binding.provider,
                &binding.external_id,
                &binding.external_key,
                &binding.sync_mode,
                &binding.state,
                &binding.last_sync_at,
            ],
        )?;

        Ok(binding)
    }

    pub fn list_for_task(&self, task_id: &str) -> Result<Vec<TrackerBinding>, StorageError> {
        let mut statement = self.connection.prepare(
            r#"
            SELECT
                id,
                task_id,
                provider,
                external_id,
                external_key,
                sync_mode,
                state,
                last_sync_at
            FROM tracker_bindings
            WHERE task_id = ?1
            ORDER BY provider ASC, id ASC
            "#,
        )?;
        let mut rows = statement.query(params![task_id])?;
        let mut bindings = Vec::new();

        while let Some(row) = rows.next()? {
            bindings.push(TrackerBinding {
                id: row.get("id")?,
                task_id: row.get("task_id")?,
                provider: row.get("provider")?,
                external_id: row.get("external_id")?,
                external_key: row.get("external_key")?,
                sync_mode: row.get("sync_mode")?,
                state: row.get("state")?,
                last_sync_at: row.get("last_sync_at")?,
            });
        }

        Ok(bindings)
    }
}
