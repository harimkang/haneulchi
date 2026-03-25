use rusqlite::{Connection, OptionalExtension, params};

use crate::StorageError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CacheRootRecord {
    pub id: String,
    pub root_path: String,
    pub created_at: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CacheEntryRecord {
    pub id: String,
    pub cache_root_id: String,
    pub path: String,
    pub size_bytes: u64,
    pub last_accessed_at: Option<String>,
    pub content_hash: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuotaRecord {
    pub id: String,
    pub cache_root_id: String,
    pub max_bytes: u64,
    pub action: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct QuotaStatus {
    pub max_bytes: u64,
    pub current_bytes: u64,
    /// `true` when `current_bytes >= max_bytes`. A quota of 0 bytes is always over.
    pub is_over: bool,
}

pub struct CacheRepository<'connection> {
    connection: &'connection Connection,
}

impl<'connection> CacheRepository<'connection> {
    pub(crate) fn new(connection: &'connection Connection) -> Self {
        Self { connection }
    }

    pub fn create_root(
        &self,
        id: &str,
        root_path: &str,
        created_at: &str,
    ) -> Result<CacheRootRecord, StorageError> {
        self.connection.execute(
            r#"
            INSERT INTO cache_roots (id, root_path, created_at)
            VALUES (?1, ?2, ?3)
            ON CONFLICT(id) DO UPDATE SET
                root_path = excluded.root_path,
                created_at = excluded.created_at
            "#,
            params![id, root_path, created_at],
        )?;

        Ok(CacheRootRecord {
            id: id.to_string(),
            root_path: root_path.to_string(),
            created_at: created_at.to_string(),
        })
    }

    pub fn list_entries(
        &self,
        cache_root_id: &str,
    ) -> Result<Vec<CacheEntryRecord>, StorageError> {
        let mut statement = self.connection.prepare(
            r#"
            SELECT id, cache_root_id, path, size_bytes, last_accessed_at, content_hash
            FROM cache_entries
            WHERE cache_root_id = ?1
            ORDER BY path ASC
            "#,
        )?;

        let mut rows = statement.query(params![cache_root_id])?;
        let mut entries = Vec::new();

        while let Some(row) = rows.next()? {
            entries.push(CacheEntryRecord {
                id: row.get("id")?,
                cache_root_id: row.get("cache_root_id")?,
                path: row.get("path")?,
                size_bytes: row.get::<_, i64>("size_bytes")? as u64,
                last_accessed_at: row.get("last_accessed_at")?,
                content_hash: row.get("content_hash")?,
            });
        }

        Ok(entries)
    }

    pub fn upsert_entry(&self, record: CacheEntryRecord) -> Result<(), StorageError> {
        self.connection.execute(
            r#"
            INSERT INTO cache_entries (
                id, cache_root_id, path, size_bytes, last_accessed_at, content_hash
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            ON CONFLICT(id) DO UPDATE SET
                cache_root_id = excluded.cache_root_id,
                path = excluded.path,
                size_bytes = excluded.size_bytes,
                last_accessed_at = excluded.last_accessed_at,
                content_hash = excluded.content_hash
            "#,
            params![
                record.id,
                record.cache_root_id,
                record.path,
                record.size_bytes as i64,
                record.last_accessed_at,
                record.content_hash,
            ],
        )?;

        Ok(())
    }

    pub fn total_size_bytes(&self, cache_root_id: &str) -> Result<u64, StorageError> {
        let total: i64 = self.connection.query_row(
            "SELECT COALESCE(SUM(size_bytes), 0) FROM cache_entries WHERE cache_root_id = ?1",
            params![cache_root_id],
            |row| row.get(0),
        )?;

        Ok(total as u64)
    }

    pub fn upsert_quota(&self, record: QuotaRecord) -> Result<(), StorageError> {
        self.connection.execute(
            r#"
            INSERT INTO cache_quotas (id, cache_root_id, max_bytes, action)
            VALUES (?1, ?2, ?3, ?4)
            ON CONFLICT(cache_root_id) DO UPDATE SET
                id = excluded.id,
                max_bytes = excluded.max_bytes,
                action = excluded.action
            "#,
            params![
                record.id,
                record.cache_root_id,
                record.max_bytes as i64,
                record.action,
            ],
        )?;

        Ok(())
    }

    pub fn get_quota(&self, cache_root_id: &str) -> Result<Option<QuotaRecord>, StorageError> {
        let result = self
            .connection
            .query_row(
                r#"
                SELECT id, cache_root_id, max_bytes, action
                FROM cache_quotas
                WHERE cache_root_id = ?1
                LIMIT 1
                "#,
                params![cache_root_id],
                |row| {
                    Ok(QuotaRecord {
                        id: row.get("id")?,
                        cache_root_id: row.get("cache_root_id")?,
                        max_bytes: row.get::<_, i64>("max_bytes")? as u64,
                        action: row.get("action")?,
                    })
                },
            )
            .optional()?;

        Ok(result)
    }

    pub fn check_quota(&self, cache_root_id: &str) -> Result<Option<QuotaStatus>, StorageError> {
        let Some(quota) = self.get_quota(cache_root_id)? else {
            return Ok(None);
        };

        let current_bytes = self.total_size_bytes(cache_root_id)?;
        let is_over = current_bytes >= quota.max_bytes;

        Ok(Some(QuotaStatus {
            max_bytes: quota.max_bytes,
            current_bytes,
            is_over,
        }))
    }
}
