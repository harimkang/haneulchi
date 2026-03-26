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
        CREATE INDEX IF NOT EXISTS idx_tasks_automation_mode
            ON tasks(automation_mode);
        CREATE INDEX IF NOT EXISTS idx_retry_queue_due_state
            ON retry_queue(due_at, retry_state);

        CREATE VIEW IF NOT EXISTS v_control_tower_ops_strip AS
        SELECT
            runtime.cadence_ms AS cadence_ms,
            runtime.last_tick_at AS last_tick_at,
            runtime.last_reconcile_at AS last_reconcile_at,
            runtime.running_slots AS running_slots,
            runtime.max_slots AS max_slots,
            COALESCE(retry.retry_due_count, 0) AS retry_due_count,
            runtime.workflow_state AS workflow_state,
            runtime.tracker_state AS tracker_state
        FROM orchestrator_runtime runtime
        LEFT JOIN (
            SELECT COUNT(*) AS retry_due_count
            FROM retry_queue
            WHERE retry_state = 'due'
        ) retry ON 1 = 1
        WHERE runtime.singleton_key = 'main';

        CREATE VIEW IF NOT EXISTS v_task_drawer_automation AS
        SELECT
            tasks.id AS task_id,
            tasks.project_id AS project_id,
            tasks.automation_mode AS automation_mode,
            tasks.tracker_binding_state AS tracker_binding_state,
            (
                SELECT task_claims.state
                FROM task_claims
                WHERE task_claims.task_id = tasks.id
                ORDER BY task_claims.claimed_at DESC, task_claims.id DESC
                LIMIT 1
            ) AS latest_claim_state,
            (
                SELECT retry_queue.retry_state
                FROM retry_queue
                WHERE retry_queue.task_id = tasks.id
                ORDER BY retry_queue.attempt DESC, retry_queue.id DESC
                LIMIT 1
            ) AS latest_retry_state,
            (
                SELECT retry_queue.due_at
                FROM retry_queue
                WHERE retry_queue.task_id = tasks.id
                ORDER BY retry_queue.attempt DESC, retry_queue.id DESC
                LIMIT 1
            ) AS latest_retry_due_at,
            tasks.linked_session_id AS linked_session_id,
            tasks.linked_worktree_id AS linked_worktree_id,
            tasks.latest_review_id AS latest_review_id
        FROM tasks;

        CREATE VIEW IF NOT EXISTS v_automation_health AS
        SELECT
            runtime.workflow_state AS workflow_state,
            runtime.tracker_state AS tracker_state,
            runtime.running_slots AS running_slots,
            runtime.max_slots AS max_slots,
            COALESCE(retry.retry_due_count, 0) AS retry_due_count,
            COALESCE(retry.retry_backing_off_count, 0) AS retry_backing_off_count,
            reload.status AS last_reload_status,
            reload.kept_last_good_hash AS last_reload_last_good_hash,
            reload.message AS last_reload_message
        FROM orchestrator_runtime runtime
        LEFT JOIN (
            SELECT
                SUM(CASE WHEN retry_state = 'due' THEN 1 ELSE 0 END) AS retry_due_count,
                SUM(CASE WHEN retry_state = 'backing_off' THEN 1 ELSE 0 END) AS retry_backing_off_count
            FROM retry_queue
        ) retry ON 1 = 1
        LEFT JOIN (
            SELECT
                workflow_reload_events.status AS status,
                workflow_reload_events.kept_last_good_hash AS kept_last_good_hash,
                workflow_reload_events.message AS message
            FROM workflow_reload_events
            ORDER BY workflow_reload_events.created_at DESC, workflow_reload_events.id DESC
            LIMIT 1
        ) reload ON 1 = 1
        WHERE runtime.singleton_key = 'main';

        CREATE TABLE IF NOT EXISTS cache_roots (
            id TEXT PRIMARY KEY,
            root_path TEXT NOT NULL,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS cache_entries (
            id TEXT PRIMARY KEY,
            cache_root_id TEXT REFERENCES cache_roots(id),
            path TEXT NOT NULL,
            size_bytes INTEGER NOT NULL,
            last_accessed_at TEXT,
            content_hash TEXT
        );

        CREATE TABLE IF NOT EXISTS cache_quotas (
            id TEXT PRIMARY KEY,
            cache_root_id TEXT REFERENCES cache_roots(id),
            max_bytes INTEGER NOT NULL,
            action TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS terminal_settings (
            singleton_key TEXT PRIMARY KEY,
            shell TEXT NOT NULL,
            default_cols INTEGER NOT NULL,
            default_rows INTEGER NOT NULL,
            scrollback_lines INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS secret_refs (
            id TEXT PRIMARY KEY,
            label TEXT NOT NULL,
            env_var_name TEXT NOT NULL,
            keychain_service TEXT NOT NULL,
            keychain_account TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS worktree_policies (
            id TEXT PRIMARY KEY,
            max_age_days INTEGER,
            max_count INTEGER,
            auto_prune INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS notification_rules (
            id TEXT PRIMARY KEY,
            event_kind TEXT NOT NULL,
            enabled INTEGER NOT NULL DEFAULT 1
        );

        CREATE TABLE IF NOT EXISTS projects (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            root_path TEXT NOT NULL,
            last_focused_at TEXT
        );

        CREATE TABLE IF NOT EXISTS layouts (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            data_json TEXT NOT NULL,
            saved_at TEXT
        );

        CREATE TABLE IF NOT EXISTS session_metadata (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            title TEXT NOT NULL,
            cwd TEXT NOT NULL,
            branch TEXT,
            last_active_at TEXT,
            is_recoverable INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS app_state (
            singleton_key TEXT PRIMARY KEY,
            active_route TEXT NOT NULL,
            last_project_id TEXT,
            last_session_id TEXT,
            saved_at TEXT
        );

        CREATE INDEX IF NOT EXISTS idx_cache_entries_root
            ON cache_entries(cache_root_id);
        CREATE UNIQUE INDEX IF NOT EXISTS idx_cache_quotas_root
            ON cache_quotas(cache_root_id);
        CREATE INDEX IF NOT EXISTS idx_session_metadata_project_recoverable
            ON session_metadata(project_id, is_recoverable);
        CREATE INDEX IF NOT EXISTS idx_layouts_project_saved
            ON layouts(project_id, saved_at);
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
    ensure_column(
        connection,
        "worktrees",
        "lifecycle_state",
        "TEXT NOT NULL DEFAULT 'in_use'",
    )?;
    ensure_column(connection, "worktrees", "size_bytes", "INTEGER")?;
    ensure_column(
        connection,
        "worktrees",
        "is_pinned",
        "INTEGER NOT NULL DEFAULT 0",
    )?;
    ensure_column(connection, "worktrees", "last_accessed_at", "TEXT")?;
    ensure_column(
        connection,
        "terminal_settings",
        "font_name",
        "TEXT NOT NULL DEFAULT ''",
    )?;
    ensure_column(
        connection,
        "terminal_settings",
        "theme",
        "TEXT NOT NULL DEFAULT ''",
    )?;
    ensure_column(
        connection,
        "terminal_settings",
        "cursor_style",
        "TEXT NOT NULL DEFAULT ''",
    )?;
    ensure_column(
        connection,
        "secret_refs",
        "scope",
        "TEXT NOT NULL DEFAULT ''",
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
