use rusqlite::{Connection, OptionalExtension, params};

use crate::StorageError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TerminalSettingsRow {
    pub shell: String,
    pub default_cols: i64,
    pub default_rows: i64,
    pub scrollback_lines: i64,
    pub font_name: String,
    pub theme: String,
    pub cursor_style: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SecretRefRow {
    pub id: String,
    pub label: String,
    pub env_var_name: String,
    pub keychain_service: String,
    pub keychain_account: String,
    pub scope: String,
}

pub struct SettingsRepository<'connection> {
    connection: &'connection Connection,
}

impl<'connection> SettingsRepository<'connection> {
    pub(crate) fn new(connection: &'connection Connection) -> Self {
        Self { connection }
    }

    pub fn get_terminal_settings(&self) -> Result<Option<TerminalSettingsRow>, StorageError> {
        let mut statement = self.connection.prepare(
            r#"
            SELECT shell, default_cols, default_rows, scrollback_lines,
                   font_name, theme, cursor_style
            FROM terminal_settings
            WHERE singleton_key = 'default'
            "#,
        )?;

        statement
            .query_row([], |row| {
                Ok(TerminalSettingsRow {
                    shell: row.get("shell")?,
                    default_cols: row.get("default_cols")?,
                    default_rows: row.get("default_rows")?,
                    scrollback_lines: row.get("scrollback_lines")?,
                    font_name: row.get("font_name")?,
                    theme: row.get("theme")?,
                    cursor_style: row.get("cursor_style")?,
                })
            })
            .optional()
            .map_err(Into::into)
    }

    pub fn upsert_terminal_settings(&self, row: TerminalSettingsRow) -> Result<(), StorageError> {
        self.connection.execute(
            r#"
            INSERT INTO terminal_settings (
                singleton_key, shell, default_cols, default_rows, scrollback_lines,
                font_name, theme, cursor_style
            ) VALUES ('default', ?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ON CONFLICT(singleton_key) DO UPDATE SET
                shell = excluded.shell,
                default_cols = excluded.default_cols,
                default_rows = excluded.default_rows,
                scrollback_lines = excluded.scrollback_lines,
                font_name = excluded.font_name,
                theme = excluded.theme,
                cursor_style = excluded.cursor_style
            "#,
            params![
                row.shell,
                row.default_cols,
                row.default_rows,
                row.scrollback_lines,
                row.font_name,
                row.theme,
                row.cursor_style,
            ],
        )?;

        Ok(())
    }

    pub fn list_secret_refs(&self) -> Result<Vec<SecretRefRow>, StorageError> {
        let mut statement = self.connection.prepare(
            r#"
            SELECT id, label, env_var_name, keychain_service, keychain_account, scope
            FROM secret_refs
            ORDER BY label ASC, id ASC
            "#,
        )?;

        let mut rows = statement.query([])?;
        let mut refs = Vec::new();

        while let Some(row) = rows.next()? {
            refs.push(SecretRefRow {
                id: row.get("id")?,
                label: row.get("label")?,
                env_var_name: row.get("env_var_name")?,
                keychain_service: row.get("keychain_service")?,
                keychain_account: row.get("keychain_account")?,
                scope: row.get("scope")?,
            });
        }

        Ok(refs)
    }

    pub fn upsert_secret_ref(&self, row: SecretRefRow) -> Result<(), StorageError> {
        self.connection.execute(
            r#"
            INSERT INTO secret_refs (
                id, label, env_var_name, keychain_service, keychain_account, scope
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            ON CONFLICT(id) DO UPDATE SET
                label = excluded.label,
                env_var_name = excluded.env_var_name,
                keychain_service = excluded.keychain_service,
                keychain_account = excluded.keychain_account,
                scope = excluded.scope
            "#,
            params![
                row.id,
                row.label,
                row.env_var_name,
                row.keychain_service,
                row.keychain_account,
                row.scope
            ],
        )?;

        Ok(())
    }

    pub fn delete_secret_ref(&self, id: &str) -> Result<(), StorageError> {
        self.connection
            .execute("DELETE FROM secret_refs WHERE id = ?1", params![id])?;

        Ok(())
    }
}
