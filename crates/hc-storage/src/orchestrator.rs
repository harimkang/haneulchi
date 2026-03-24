use hc_domain::{OrchestratorRuntime, WorkflowReloadEvent};
use rusqlite::{Connection, OptionalExtension, params};

use crate::StorageError;

pub struct OrchestratorRepository<'connection> {
    connection: &'connection Connection,
}

impl<'connection> OrchestratorRepository<'connection> {
    pub(crate) fn new(connection: &'connection Connection) -> Self {
        Self { connection }
    }

    pub fn save_runtime(
        &self,
        runtime: OrchestratorRuntime,
    ) -> Result<OrchestratorRuntime, StorageError> {
        self.connection.execute(
            r#"
            INSERT INTO orchestrator_runtime (
                singleton_key,
                cadence_ms,
                last_tick_at,
                last_reconcile_at,
                max_slots,
                running_slots,
                workflow_state,
                tracker_state
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ON CONFLICT(singleton_key) DO UPDATE SET
                cadence_ms = excluded.cadence_ms,
                last_tick_at = excluded.last_tick_at,
                last_reconcile_at = excluded.last_reconcile_at,
                max_slots = excluded.max_slots,
                running_slots = excluded.running_slots,
                workflow_state = excluded.workflow_state,
                tracker_state = excluded.tracker_state
            "#,
            params![
                &runtime.singleton_key,
                runtime.cadence_ms as i64,
                &runtime.last_tick_at,
                &runtime.last_reconcile_at,
                runtime.max_slots,
                runtime.running_slots,
                &runtime.workflow_state,
                &runtime.tracker_state,
            ],
        )?;

        Ok(runtime)
    }

    pub fn load_runtime(&self) -> Result<Option<OrchestratorRuntime>, StorageError> {
        self.connection
            .query_row(
                r#"
                SELECT
                    singleton_key,
                    cadence_ms,
                    last_tick_at,
                    last_reconcile_at,
                    max_slots,
                    running_slots,
                    workflow_state,
                    tracker_state
                FROM orchestrator_runtime
                WHERE singleton_key = 'main'
                "#,
                [],
                |row| {
                    Ok(OrchestratorRuntime {
                        singleton_key: row.get("singleton_key")?,
                        cadence_ms: row.get::<_, i64>("cadence_ms")? as u64,
                        last_tick_at: row.get("last_tick_at")?,
                        last_reconcile_at: row.get("last_reconcile_at")?,
                        max_slots: row.get("max_slots")?,
                        running_slots: row.get("running_slots")?,
                        workflow_state: row.get("workflow_state")?,
                        tracker_state: row.get("tracker_state")?,
                    })
                },
            )
            .optional()
            .map_err(StorageError::from)
    }

    pub fn append_workflow_reload_event(
        &self,
        event: WorkflowReloadEvent,
    ) -> Result<WorkflowReloadEvent, StorageError> {
        self.connection.execute(
            r#"
            INSERT INTO workflow_reload_events (
                id,
                project_id,
                file_path,
                status,
                loaded_hash,
                kept_last_good_hash,
                message,
                created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
            params![
                &event.id,
                &event.project_id,
                &event.file_path,
                &event.status,
                &event.loaded_hash,
                &event.kept_last_good_hash,
                &event.message,
                &event.created_at,
            ],
        )?;

        Ok(event)
    }

    pub fn list_workflow_reload_events(
        &self,
        project_id: &str,
    ) -> Result<Vec<WorkflowReloadEvent>, StorageError> {
        let mut statement = self.connection.prepare(
            r#"
            SELECT
                id,
                project_id,
                file_path,
                status,
                loaded_hash,
                kept_last_good_hash,
                message,
                created_at
            FROM workflow_reload_events
            WHERE project_id = ?1
            ORDER BY created_at ASC, id ASC
            "#,
        )?;
        let mut rows = statement.query(params![project_id])?;
        let mut events = Vec::new();

        while let Some(row) = rows.next()? {
            events.push(WorkflowReloadEvent {
                id: row.get("id")?,
                project_id: row.get("project_id")?,
                file_path: row.get("file_path")?,
                status: row.get("status")?,
                loaded_hash: row.get("loaded_hash")?,
                kept_last_good_hash: row.get("kept_last_good_hash")?,
                message: row.get("message")?,
                created_at: row.get("created_at")?,
            });
        }

        Ok(events)
    }
}
