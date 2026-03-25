use rusqlite::{Connection, OptionalExtension, params};

use crate::StorageError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppStateRow {
    pub active_route: String,
    pub last_project_id: Option<String>,
    pub last_session_id: Option<String>,
    pub saved_at: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionMetadataRow {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub cwd: String,
    pub branch: Option<String>,
    pub last_active_at: Option<String>,
    pub is_recoverable: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LayoutRow {
    pub id: String,
    pub project_id: String,
    pub data_json: String,
    pub saved_at: Option<String>,
}

pub struct PersistenceRepository<'connection> {
    connection: &'connection Connection,
}

impl<'connection> PersistenceRepository<'connection> {
    pub(crate) fn new(connection: &'connection Connection) -> Self {
        Self { connection }
    }

    pub fn save_app_state(
        &self,
        route: &str,
        last_project_id: Option<&str>,
        last_session_id: Option<&str>,
        saved_at: &str,
    ) -> Result<(), StorageError> {
        self.connection.execute(
            r#"
            INSERT INTO app_state (
                singleton_key, active_route, last_project_id, last_session_id, saved_at
            ) VALUES ('default', ?1, ?2, ?3, ?4)
            ON CONFLICT(singleton_key) DO UPDATE SET
                active_route = excluded.active_route,
                last_project_id = excluded.last_project_id,
                last_session_id = excluded.last_session_id,
                saved_at = excluded.saved_at
            "#,
            params![route, last_project_id, last_session_id, saved_at],
        )?;

        Ok(())
    }

    pub fn load_app_state(&self) -> Result<Option<AppStateRow>, StorageError> {
        let mut statement = self.connection.prepare(
            r#"
            SELECT active_route, last_project_id, last_session_id, saved_at
            FROM app_state
            WHERE singleton_key = 'default'
            "#,
        )?;

        statement
            .query_row([], |row| {
                Ok(AppStateRow {
                    active_route: row.get("active_route")?,
                    last_project_id: row.get("last_project_id")?,
                    last_session_id: row.get("last_session_id")?,
                    saved_at: row.get("saved_at")?,
                })
            })
            .optional()
            .map_err(Into::into)
    }

    pub fn upsert_session_metadata(
        &self,
        row: SessionMetadataRow,
    ) -> Result<(), StorageError> {
        self.connection.execute(
            r#"
            INSERT INTO session_metadata (
                id, project_id, title, cwd, branch, last_active_at, is_recoverable
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ON CONFLICT(id) DO UPDATE SET
                project_id = excluded.project_id,
                title = excluded.title,
                cwd = excluded.cwd,
                branch = excluded.branch,
                last_active_at = excluded.last_active_at,
                is_recoverable = excluded.is_recoverable
            "#,
            params![
                row.id,
                row.project_id,
                row.title,
                row.cwd,
                row.branch,
                row.last_active_at,
                row.is_recoverable as i32,
            ],
        )?;

        Ok(())
    }

    pub fn list_recoverable_sessions(
        &self,
        project_id: &str,
    ) -> Result<Vec<SessionMetadataRow>, StorageError> {
        let mut statement = self.connection.prepare(
            r#"
            SELECT id, project_id, title, cwd, branch, last_active_at, is_recoverable
            FROM session_metadata
            WHERE project_id = ?1 AND is_recoverable = 1
            ORDER BY last_active_at DESC, id DESC
            "#,
        )?;

        let mut rows = statement.query(params![project_id])?;
        let mut sessions = Vec::new();

        while let Some(row) = rows.next()? {
            sessions.push(SessionMetadataRow {
                id: row.get("id")?,
                project_id: row.get("project_id")?,
                title: row.get("title")?,
                cwd: row.get("cwd")?,
                branch: row.get("branch")?,
                last_active_at: row.get("last_active_at")?,
                is_recoverable: row.get::<_, i32>("is_recoverable")? != 0,
            });
        }

        Ok(sessions)
    }

    pub fn upsert_layout(&self, row: LayoutRow) -> Result<(), StorageError> {
        self.connection.execute(
            r#"
            INSERT INTO layouts (id, project_id, data_json, saved_at)
            VALUES (?1, ?2, ?3, ?4)
            ON CONFLICT(id) DO UPDATE SET
                project_id = excluded.project_id,
                data_json = excluded.data_json,
                saved_at = excluded.saved_at
            "#,
            params![row.id, row.project_id, row.data_json, row.saved_at],
        )?;

        Ok(())
    }

    pub fn load_latest_layout(
        &self,
        project_id: &str,
    ) -> Result<Option<LayoutRow>, StorageError> {
        let mut statement = self.connection.prepare(
            r#"
            SELECT id, project_id, data_json, saved_at
            FROM layouts
            WHERE project_id = ?1
            ORDER BY saved_at DESC, id DESC
            LIMIT 1
            "#,
        )?;

        statement
            .query_row(params![project_id], |row| {
                Ok(LayoutRow {
                    id: row.get("id")?,
                    project_id: row.get("project_id")?,
                    data_json: row.get("data_json")?,
                    saved_at: row.get("saved_at")?,
                })
            })
            .optional()
            .map_err(Into::into)
    }
}
