use hc_domain::TaskAutomationMode;
use hc_domain::inventory::{InventoryDisposition, WorktreeLifecycleState};
use hc_storage::{NewTaskRecord, NewWorktreeRecord, SqliteStore};

fn make_store() -> SqliteStore {
    SqliteStore::in_memory().expect("in-memory store")
}

fn seed_task(store: &SqliteStore, task_id: &str, project_id: &str) {
    store
        .tasks()
        .create(NewTaskRecord {
            id: task_id.to_string(),
            project_id: project_id.to_string(),
            display_key: task_id.to_uppercase(),
            title: format!("Task {task_id}"),
            description: String::new(),
            priority: "p2".to_string(),
            automation_mode: TaskAutomationMode::Manual,
            created_at: "2026-03-25T00:00:00Z".to_string(),
            updated_at: "2026-03-25T00:00:00Z".to_string(),
        })
        .expect("task create");
}

fn seed_worktree(store: &SqliteStore, worktree_id: &str, task_id: &str, project_id: &str) {
    store
        .worktrees()
        .create_or_replace(NewWorktreeRecord {
            id: worktree_id.to_string(),
            task_id: task_id.to_string(),
            project_id: project_id.to_string(),
            workspace_root: format!("/tmp/{worktree_id}"),
            base_root: ".".to_string(),
            branch_name: format!("hc/{task_id}"),
            status: "ready".to_string(),
            created_at: "2026-03-25T00:00:00Z".to_string(),
            updated_at: "2026-03-25T00:00:00Z".to_string(),
        })
        .expect("worktree create");
}

#[test]
fn inventory_groups_in_use_worktrees_correctly() {
    let store = make_store();
    seed_task(&store, "task_in_use", "proj_test");
    seed_worktree(&store, "wt_in_use", "task_in_use", "proj_test");
    // lifecycle_state defaults to "in_use" on insert

    let rows = hc_control_plane::inventory::build_inventory_for_project(&store, "proj_test")
        .expect("inventory");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].task_id, "task_in_use");
    assert_eq!(rows[0].disposition, InventoryDisposition::InUse);
    assert_eq!(rows[0].lifecycle_state, WorktreeLifecycleState::InUse);
}

#[test]
fn inventory_groups_recoverable_worktrees_correctly() {
    let store = make_store();
    seed_task(&store, "task_recoverable", "proj_test");
    seed_worktree(&store, "wt_recoverable", "task_recoverable", "proj_test");
    store
        .worktrees()
        .update_lifecycle("wt_recoverable", "recoverable", "2026-03-25T00:01:00Z")
        .expect("update lifecycle");

    let rows = hc_control_plane::inventory::build_inventory_for_project(&store, "proj_test")
        .expect("inventory");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].disposition, InventoryDisposition::Recoverable);
    assert_eq!(rows[0].lifecycle_state, WorktreeLifecycleState::Recoverable);
}

#[test]
fn inventory_groups_safe_to_delete_correctly() {
    let store = make_store();
    seed_task(&store, "task_safe", "proj_test");
    seed_worktree(&store, "wt_safe", "task_safe", "proj_test");
    store
        .worktrees()
        .update_lifecycle("wt_safe", "safe_to_delete", "2026-03-25T00:01:00Z")
        .expect("update lifecycle");

    let rows = hc_control_plane::inventory::build_inventory_for_project(&store, "proj_test")
        .expect("inventory");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].disposition, InventoryDisposition::SafeToDelete);
    assert_eq!(
        rows[0].lifecycle_state,
        WorktreeLifecycleState::SafeToDelete
    );
}

#[test]
fn inventory_groups_stale_worktrees_correctly() {
    let store = make_store();
    seed_task(&store, "task_stale", "proj_test");
    seed_worktree(&store, "wt_stale", "task_stale", "proj_test");
    store
        .worktrees()
        .update_lifecycle("wt_stale", "stale", "2026-03-25T00:01:00Z")
        .expect("update lifecycle");

    let rows = hc_control_plane::inventory::build_inventory_for_project(&store, "proj_test")
        .expect("inventory");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].disposition, InventoryDisposition::Stale);
    assert_eq!(rows[0].lifecycle_state, WorktreeLifecycleState::Stale);
}
