//! SQLite, artifact store, and Keychain boundary scaffold.

pub mod cache;
pub mod keychain;
pub mod orchestrator;
pub mod persistence;
pub mod retry_queue;
pub mod reviews;
pub mod schema;
pub mod settings;
pub mod tasks;
pub mod timeline;
pub mod tracker_bindings;
pub mod worktrees;

use std::path::Path;

use rusqlite::Connection;

pub use cache::{CacheEntryRecord, CacheRepository, CacheRootRecord, QuotaRecord, QuotaStatus};
pub use keychain::KeychainBoundary;
pub use orchestrator::OrchestratorRepository;
pub use persistence::{AppStateRow, LayoutRow, PersistenceRepository, SessionMetadataRow};
pub use retry_queue::{
    NewRetryQueueEntry, RetryFailureClass, RetryQueueRepository, advance_retry_state,
    schedule_retry_entry,
};
pub use reviews::{NewReviewItem, ReviewRepository};
pub use settings::{SecretRefRow, SettingsRepository, TerminalSettingsRow};
pub use tasks::{NewTaskRecord, TaskRepository, TaskUpdatePatch};
pub use timeline::{AppendTimelineEvent, TimelineRepository};
pub use tracker_bindings::TrackerBindingRepository;
pub use worktrees::{NewWorktreeRecord, WorktreeRecord, WorktreeRepository};

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("task not found: {0}")]
    TaskNotFound(String),
    #[error("worktree not found for task: {0}")]
    WorktreeNotFound(String),
    #[error("cache root not found: {0}")]
    CacheRootNotFound(String),
    #[error("layout not found: {0}")]
    LayoutNotFound(String),
    #[error("unknown task column: {0}")]
    UnknownTaskColumn(String),
    #[error("unknown task automation mode: {0}")]
    UnknownTaskAutomationMode(String),
    #[error("unknown claim state: {0}")]
    UnknownClaimState(String),
    #[error("unknown task claim lifecycle state: {0}")]
    UnknownTaskClaimLifecycleState(String),
    #[error("unknown review status: {0}")]
    UnknownReviewStatus(String),
    #[error("unknown timeline event kind: {0}")]
    UnknownTimelineEventKind(String),
    #[error("keychain error: {0}")]
    Keychain(String),
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

    pub fn retry_queue(&self) -> RetryQueueRepository<'_> {
        RetryQueueRepository::new(&self.connection)
    }

    pub fn orchestrator(&self) -> OrchestratorRepository<'_> {
        OrchestratorRepository::new(&self.connection)
    }

    pub fn tracker_bindings(&self) -> TrackerBindingRepository<'_> {
        TrackerBindingRepository::new(&self.connection)
    }

    pub fn timeline(&self) -> TimelineRepository<'_> {
        TimelineRepository::new(&self.connection)
    }

    pub fn worktrees(&self) -> WorktreeRepository<'_> {
        WorktreeRepository::new(&self.connection)
    }

    pub fn cache(&self) -> CacheRepository<'_> {
        CacheRepository::new(&self.connection)
    }

    pub fn settings_repo(&self) -> SettingsRepository<'_> {
        SettingsRepository::new(&self.connection)
    }

    pub fn persistence(&self) -> PersistenceRepository<'_> {
        PersistenceRepository::new(&self.connection)
    }
}
