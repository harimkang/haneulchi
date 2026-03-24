use rusqlite::Connection;

pub fn migrate(connection: &Connection) -> rusqlite::Result<()> {
    connection.execute_batch(
        r#"
        PRAGMA foreign_keys = ON;

        CREATE TABLE IF NOT EXISTS tasks (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            display_key TEXT NOT NULL,
            title TEXT NOT NULL,
            description TEXT NOT NULL,
            column_name TEXT NOT NULL,
            priority TEXT NOT NULL,
            automation_mode TEXT NOT NULL,
            tracker_binding_state TEXT NOT NULL,
            linked_session_id TEXT,
            linked_worktree_id TEXT,
            latest_review_id TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS task_claims (
            id TEXT PRIMARY KEY,
            task_id TEXT NOT NULL REFERENCES tasks(id),
            state TEXT NOT NULL,
            claimed_at TEXT NOT NULL,
            released_at TEXT,
            session_id TEXT,
            reason TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS review_items (
            id TEXT PRIMARY KEY,
            task_id TEXT NOT NULL REFERENCES tasks(id),
            session_id TEXT,
            status TEXT NOT NULL,
            summary TEXT NOT NULL,
            touched_files_json TEXT NOT NULL,
            diff_summary TEXT,
            tests_summary TEXT,
            command_summary TEXT,
            hook_summary TEXT,
            evidence_summary TEXT,
            checklist_summary TEXT,
            warnings_json TEXT NOT NULL,
            evidence_manifest_path TEXT,
            review_checklist_result TEXT,
            created_at TEXT NOT NULL,
            decided_at TEXT
        );

        CREATE TABLE IF NOT EXISTS worktrees (
            id TEXT PRIMARY KEY,
            task_id TEXT NOT NULL UNIQUE,
            project_id TEXT NOT NULL,
            workspace_root TEXT NOT NULL,
            base_root TEXT NOT NULL,
            branch_name TEXT NOT NULL,
            status TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS task_events (
            id TEXT PRIMARY KEY,
            task_id TEXT NOT NULL REFERENCES tasks(id),
            session_id TEXT,
            review_item_id TEXT,
            worktree_id TEXT,
            kind TEXT NOT NULL,
            actor TEXT NOT NULL,
            reason_code TEXT,
            payload_json TEXT,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS orchestrator_runtime (
            singleton_key TEXT PRIMARY KEY,
            cadence_ms INTEGER NOT NULL,
            last_tick_at TEXT,
            last_reconcile_at TEXT,
            max_slots INTEGER NOT NULL,
            running_slots INTEGER NOT NULL,
            workflow_state TEXT NOT NULL,
            tracker_state TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS retry_queue (
            id TEXT PRIMARY KEY,
            task_id TEXT NOT NULL REFERENCES tasks(id),
            project_id TEXT NOT NULL,
            attempt INTEGER NOT NULL,
            reason_code TEXT NOT NULL,
            due_at TEXT,
            backoff_ms INTEGER NOT NULL,
            claim_state TEXT NOT NULL,
            retry_state TEXT NOT NULL DEFAULT 'none'
        );

        CREATE TABLE IF NOT EXISTS workflow_reload_events (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            file_path TEXT NOT NULL,
            status TEXT NOT NULL,
            loaded_hash TEXT,
            kept_last_good_hash TEXT,
            message TEXT,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS tracker_bindings (
            id TEXT PRIMARY KEY,
            task_id TEXT NOT NULL REFERENCES tasks(id),
            provider TEXT NOT NULL,
            external_id TEXT NOT NULL,
            external_key TEXT NOT NULL,
            sync_mode TEXT NOT NULL,
            state TEXT NOT NULL,
            last_sync_at TEXT
        );

        CREATE INDEX IF NOT EXISTS idx_tasks_project_column
            ON tasks(project_id, column_name, updated_at);
        CREATE INDEX IF NOT EXISTS idx_task_claims_task_state
            ON task_claims(task_id, state, claimed_at);
        CREATE INDEX IF NOT EXISTS idx_review_items_task_created
            ON review_items(task_id, created_at);
        CREATE INDEX IF NOT EXISTS idx_worktrees_project_task
            ON worktrees(project_id, task_id, updated_at);
        CREATE INDEX IF NOT EXISTS idx_task_events_task_created
            ON task_events(task_id, created_at);
        CREATE INDEX IF NOT EXISTS idx_retry_queue_task_due
            ON retry_queue(task_id, due_at);
        CREATE INDEX IF NOT EXISTS idx_workflow_reload_events_project_created
            ON workflow_reload_events(project_id, created_at);
        CREATE INDEX IF NOT EXISTS idx_tracker_bindings_task_provider
            ON tracker_bindings(task_id, provider);
        "#,
    )?;

    ensure_column(connection, "review_items", "hook_summary", "TEXT")?;
    ensure_column(connection, "review_items", "evidence_summary", "TEXT")?;
    ensure_column(connection, "review_items", "checklist_summary", "TEXT")?;
    ensure_column(
        connection,
        "retry_queue",
        "retry_state",
        "TEXT NOT NULL DEFAULT 'none'",
    )?;

    Ok(())
}

fn ensure_column(
    connection: &Connection,
    table: &str,
    column: &str,
    definition: &str,
) -> rusqlite::Result<()> {
    let pragma = format!("PRAGMA table_info({table})");
    let mut statement = connection.prepare(&pragma)?;
    let mut rows = statement.query([])?;
    let mut exists = false;

    while let Some(row) = rows.next()? {
        if row.get::<_, String>(1)? == column {
            exists = true;
            break;
        }
    }

    if !exists {
        connection.execute(
            &format!("ALTER TABLE {table} ADD COLUMN {column} {definition}"),
            [],
        )?;
    }

    Ok(())
}
