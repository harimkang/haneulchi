//! SQLite, artifact store, and Keychain boundary scaffold.

pub mod reviews;
pub mod schema;
pub mod tasks;
pub mod timeline;

use std::path::Path;

use rusqlite::Connection;

pub use reviews::{NewReviewItem, ReviewRepository};
pub use tasks::{NewTaskRecord, TaskRepository, TaskUpdatePatch};
pub use timeline::TimelineRepository;

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("task not found: {0}")]
    TaskNotFound(String),
    #[error("unknown task column: {0}")]
    UnknownTaskColumn(String),
    #[error("unknown task automation mode: {0}")]
    UnknownTaskAutomationMode(String),
    #[error("unknown task claim lifecycle state: {0}")]
    UnknownTaskClaimLifecycleState(String),
    #[error("unknown review status: {0}")]
    UnknownReviewStatus(String),
    #[error("unknown timeline event kind: {0}")]
    UnknownTimelineEventKind(String),
}

pub struct SqliteStore {
    connection: Connection,
}

impl SqliteStore {
    pub fn in_memory() -> Result<Self, StorageError> {
        let connection = Connection::open_in_memory()?;
        schema::migrate(&connection)?;

        Ok(Self { connection })
    }

    pub fn open(path: impl AsRef<Path>) -> Result<Self, StorageError> {
        let connection = Connection::open(path)?;
        schema::migrate(&connection)?;

        Ok(Self { connection })
    }

    pub fn connection(&self) -> &Connection {
        &self.connection
    }

    pub fn tasks(&self) -> TaskRepository<'_> {
        TaskRepository::new(&self.connection)
    }

    pub fn reviews(&self) -> ReviewRepository<'_> {
        ReviewRepository::new(&self.connection)
    }

    pub fn timeline(&self) -> TimelineRepository<'_> {
        TimelineRepository::new(&self.connection)
    }
}
