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
            warnings_json TEXT NOT NULL,
            evidence_manifest_path TEXT,
            review_checklist_result TEXT,
            created_at TEXT NOT NULL,
            decided_at TEXT
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

        CREATE INDEX IF NOT EXISTS idx_tasks_project_column
            ON tasks(project_id, column_name, updated_at);
        CREATE INDEX IF NOT EXISTS idx_task_claims_task_state
            ON task_claims(task_id, state, claimed_at);
        CREATE INDEX IF NOT EXISTS idx_review_items_task_created
            ON review_items(task_id, created_at);
        CREATE INDEX IF NOT EXISTS idx_task_events_task_created
            ON task_events(task_id, created_at);
        "#,
    )
}
