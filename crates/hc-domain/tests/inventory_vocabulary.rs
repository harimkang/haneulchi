use hc_domain::inventory::{
    InventoryDisposition, InventoryRow, InventorySummary, RestorePointSummary,
    WorktreeLifecycleState,
};

#[test]
fn worktree_lifecycle_state_all_string_values() {
    assert_eq!(
        WorktreeLifecycleState::all(),
        ["in_use", "recoverable", "safe_to_delete", "stale"]
    );
}

#[test]
fn inventory_disposition_all_string_values() {
    assert_eq!(
        InventoryDisposition::all(),
        ["in_use", "recoverable", "safe_to_delete", "stale"]
    );
}

#[test]
fn inventory_summary_field_construction() {
    let summary = InventorySummary {
        total: 10,
        in_use: 3,
        recoverable: 2,
        safe_to_delete: 4,
        stale: 1,
    };
    assert_eq!(summary.total, 10);
    assert_eq!(summary.in_use, 3);
    assert_eq!(summary.recoverable, 2);
    assert_eq!(summary.safe_to_delete, 4);
    assert_eq!(summary.stale, 1);
}

#[test]
fn inventory_row_field_construction() {
    let row = InventoryRow {
        worktree_id: "wt_abc".to_string(),
        task_id: "task_abc".to_string(),
        path: "/tmp/worktree".to_string(),
        project_name: "my-project".to_string(),
        branch: Some("main".to_string()),
        disposition: InventoryDisposition::InUse,
        lifecycle_state: WorktreeLifecycleState::InUse,
        size_bytes: Some(1024),
        last_accessed_at: Some("2026-03-24T00:00:00Z".to_string()),
        is_pinned: false,
        is_degraded: false,
    };
    assert_eq!(row.worktree_id, "wt_abc");
    assert_eq!(row.task_id, "task_abc");
    assert_eq!(row.path, "/tmp/worktree");
    assert_eq!(row.project_name, "my-project");
    assert_eq!(row.branch, Some("main".to_string()));
    assert_eq!(row.disposition, InventoryDisposition::InUse);
    assert_eq!(row.lifecycle_state, WorktreeLifecycleState::InUse);
    assert_eq!(row.size_bytes, Some(1024));
    assert!(!row.is_pinned);
    assert!(!row.is_degraded);
}

#[test]
fn restore_point_summary_field_construction() {
    let rps = RestorePointSummary {
        restore_id: "rp_001".to_string(),
        project_id: "proj_demo".to_string(),
        snapshot_at: Some("2026-03-24T00:00:00Z".to_string()),
        is_complete: true,
    };
    assert_eq!(rps.restore_id, "rp_001");
    assert_eq!(rps.project_id, "proj_demo");
    assert!(rps.is_complete);
}
