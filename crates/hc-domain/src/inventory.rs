use serde::{Deserialize, Serialize};

/// Raw technical lifecycle state of a worktree directory.
///
/// This is a low-level value that reflects filesystem and git reality — it
/// describes *what the worktree actually is* at the infrastructure layer,
/// independent of any UI or action-gating concerns.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WorktreeLifecycleState {
    /// The worktree is actively checked out and in use. (default)
    #[default]
    InUse,
    Recoverable,
    SafeToDelete,
    Stale,
}

impl WorktreeLifecycleState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InUse => "in_use",
            Self::Recoverable => "recoverable",
            Self::SafeToDelete => "safe_to_delete",
            Self::Stale => "stale",
        }
    }

    pub const fn all() -> [&'static str; 4] {
        [
            Self::InUse.as_str(),
            Self::Recoverable.as_str(),
            Self::SafeToDelete.as_str(),
            Self::Stale.as_str(),
        ]
    }
}

/// Computed high-level classification of a worktree for display and action-gating.
///
/// This is derived from `WorktreeLifecycleState` and other contextual signals.
/// It is the value the UI renders and uses to decide which actions are available
/// to the user — it does *not* directly reflect filesystem or git state.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InventoryDisposition {
    /// The worktree is actively in use; no cleanup actions are offered. (default)
    #[default]
    InUse,
    Recoverable,
    SafeToDelete,
    Stale,
}

impl InventoryDisposition {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InUse => "in_use",
            Self::Recoverable => "recoverable",
            Self::SafeToDelete => "safe_to_delete",
            Self::Stale => "stale",
        }
    }

    pub const fn all() -> [&'static str; 4] {
        [
            Self::InUse.as_str(),
            Self::Recoverable.as_str(),
            Self::SafeToDelete.as_str(),
            Self::Stale.as_str(),
        ]
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct InventorySummary {
    pub total: u32,
    pub in_use: u32,
    pub recoverable: u32,
    pub safe_to_delete: u32,
    pub stale: u32,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct InventoryRow {
    pub worktree_id: String,
    pub task_id: String,
    pub path: String,
    pub project_name: String,
    pub branch: Option<String>,
    pub disposition: InventoryDisposition,
    pub lifecycle_state: WorktreeLifecycleState,
    pub size_bytes: Option<u64>,
    pub last_accessed_at: Option<String>,
    pub is_pinned: bool,
    pub is_degraded: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct RestorePointSummary {
    pub restore_id: String,
    pub project_id: String,
    pub snapshot_at: Option<String>,
    pub is_complete: bool,
}
