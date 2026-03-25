use hc_domain::inventory::{
    InventoryDisposition, InventoryRow, InventorySummary, WorktreeLifecycleState,
};
use hc_storage::WorktreeRecord;

/// Map a storage `WorktreeRecord` to an `InventoryRow`.
/// The disposition is derived from the lifecycle_state string.
pub fn worktree_record_to_inventory_row(record: &WorktreeRecord) -> InventoryRow {
    let lifecycle_state = parse_lifecycle_state(&record.lifecycle_state);
    let disposition = lifecycle_to_disposition(&lifecycle_state);
    InventoryRow {
        worktree_id: record.id.clone(),
        task_id: record.task_id.clone(),
        path: record.workspace_root.clone(),
        project_name: record.project_id.clone(), // project_id as name proxy
        branch: Some(record.branch_name.clone()),
        disposition,
        lifecycle_state,
        size_bytes: record.size_bytes,
        last_accessed_at: record.last_accessed_at.clone(),
        is_pinned: record.is_pinned,
        is_degraded: false, // degraded flag set by recovery layer
    }
}

pub fn build_inventory_summary(rows: &[InventoryRow]) -> InventorySummary {
    let mut summary = InventorySummary {
        total: rows.len() as u32,
        in_use: 0,
        recoverable: 0,
        safe_to_delete: 0,
        stale: 0,
    };
    for row in rows {
        match row.disposition {
            InventoryDisposition::InUse => summary.in_use += 1,
            InventoryDisposition::Recoverable => summary.recoverable += 1,
            InventoryDisposition::SafeToDelete => summary.safe_to_delete += 1,
            InventoryDisposition::Stale => summary.stale += 1,
        }
    }
    summary
}

/// Build a full inventory projection from storage for a given project.
pub fn build_inventory_for_project(
    store: &hc_storage::SqliteStore,
    project_id: &str,
) -> Result<Vec<InventoryRow>, hc_storage::StorageError> {
    let records = store.worktrees().list_by_project(project_id)?;
    Ok(records
        .iter()
        .map(worktree_record_to_inventory_row)
        .collect())
}

/// Set the lifecycle state of a worktree (pin, transition, etc.)
pub fn set_worktree_lifecycle(
    store: &hc_storage::SqliteStore,
    worktree_id: &str,
    new_state: &str,
    updated_at: &str,
) -> Result<(), hc_storage::StorageError> {
    store
        .worktrees()
        .update_lifecycle(worktree_id, new_state, updated_at)
}

/// Check all cache roots for quota violations and return attention-style items.
/// Returns a Vec of (cache_root_id, QuotaStatus) for roots that are over quota.
pub fn check_cache_quotas(
    store: &hc_storage::SqliteStore,
    cache_root_ids: &[String],
) -> Vec<(String, hc_storage::QuotaStatus)> {
    cache_root_ids
        .iter()
        .filter_map(|root_id| {
            match store.cache().check_quota(root_id) {
                Ok(Some(status)) if status.is_over => Some((root_id.clone(), status)),
                Ok(_) => None,
                Err(_e) => {
                    // Storage error for root; skip to avoid aborting the whole sweep.
                    // Callers should schedule a retry if enforcement is critical.
                    None
                }
            }
        })
        .collect()
}

fn parse_lifecycle_state(s: &str) -> WorktreeLifecycleState {
    match s {
        "in_use" => WorktreeLifecycleState::InUse,
        "recoverable" => WorktreeLifecycleState::Recoverable,
        "safe_to_delete" => WorktreeLifecycleState::SafeToDelete,
        "stale" => WorktreeLifecycleState::Stale,
        _ => WorktreeLifecycleState::InUse, // safe fallback
    }
}

fn lifecycle_to_disposition(state: &WorktreeLifecycleState) -> InventoryDisposition {
    match state {
        WorktreeLifecycleState::InUse => InventoryDisposition::InUse,
        WorktreeLifecycleState::Recoverable => InventoryDisposition::Recoverable,
        WorktreeLifecycleState::SafeToDelete => InventoryDisposition::SafeToDelete,
        WorktreeLifecycleState::Stale => InventoryDisposition::Stale,
    }
}
