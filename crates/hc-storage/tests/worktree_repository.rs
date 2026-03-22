use hc_domain::TaskAutomationMode;
use hc_storage::{NewTaskRecord, NewWorktreeRecord, SqliteStore};

#[test]
fn worktree_repository_persists_project_task_and_workspace_linkage() {
    let store = SqliteStore::in_memory().expect("sqlite store");
    store
        .tasks()
        .create(NewTaskRecord {
            id: "task_104".to_string(),
            project_id: "proj_demo".to_string(),
            display_key: "TASK-104".to_string(),
            title: "Provision worktree".to_string(),
            description: "Need a task worktree".to_string(),
            priority: "p1".to_string(),
            automation_mode: TaskAutomationMode::Assisted,
            created_at: "2026-03-23T08:00:00Z".to_string(),
            updated_at: "2026-03-23T08:00:00Z".to_string(),
        })
        .expect("task row");

    let worktree = store
        .worktrees()
        .create_or_replace(NewWorktreeRecord {
            id: "wt_task_104".to_string(),
            task_id: "task_104".to_string(),
            project_id: "proj_demo".to_string(),
            workspace_root: "/tmp/demo/worktrees/task-104".to_string(),
            base_root: ".".to_string(),
            branch_name: "hc/task-104".to_string(),
            status: "ready".to_string(),
            created_at: "2026-03-23T08:01:00Z".to_string(),
            updated_at: "2026-03-23T08:01:00Z".to_string(),
        })
        .expect("worktree row");

    assert_eq!(worktree.task_id, "task_104");
    assert_eq!(worktree.project_id, "proj_demo");
    assert_eq!(worktree.workspace_root, "/tmp/demo/worktrees/task-104");
    assert_eq!(worktree.branch_name, "hc/task-104");

    let persisted = store
        .worktrees()
        .find_by_task("task_104")
        .expect("lookup")
        .expect("persisted worktree");
    assert_eq!(persisted, worktree);

    let task = store
        .tasks()
        .get("task_104")
        .expect("task lookup")
        .expect("task row");
    assert_eq!(task.linked_worktree_id.as_deref(), Some("wt_task_104"));
}
