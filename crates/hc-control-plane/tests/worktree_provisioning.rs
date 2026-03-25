use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use hc_control_plane::{reset_task_board_for_tests, shared_inventory_for_project, shared_provision_task_workspace};
use hc_domain::inventory::InventoryDisposition;

fn temp_dir(label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("hc-worktree-provision-{label}-{unique}"));
    fs::create_dir_all(&path).expect("temp dir");
    path
}

fn git(root: &Path, args: &[&str]) {
    let status = Command::new("git")
        .args(args)
        .current_dir(root)
        .status()
        .expect("git command");
    assert!(status.success(), "git {:?} failed", args);
}

#[test]
fn task_worktree_provisioning_creates_real_git_worktree_and_branch() {
    reset_task_board_for_tests();
    let repo = temp_dir("repo");
    let workspace_base = temp_dir("workspaces");
    unsafe {
        std::env::set_var("HC_WORKSPACE_ROOT", &workspace_base);
    }
    git(&repo, &["init"]);
    git(&repo, &["config", "user.email", "review@example.com"]);
    git(&repo, &["config", "user.name", "Review Bot"]);
    fs::write(repo.join("README.md"), "demo\n").expect("readme");
    git(&repo, &["add", "README.md"]);
    git(&repo, &["commit", "-m", "init"]);

    let workspace = shared_provision_task_workspace(
        repo.to_str().expect("utf8"),
        "task_ready",
        Some("."),
    )
    .expect("workspace");

    assert!(Path::new(&workspace.workspace_root).join(".git").exists());
    let output = Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(&workspace.workspace_root)
        .output()
        .expect("git branch");
    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert_eq!(branch, "hc/task_ready");
    unsafe {
        std::env::remove_var("HC_WORKSPACE_ROOT");
    }
}

#[test]
fn provisioned_worktree_starts_with_in_use_lifecycle() {
    reset_task_board_for_tests();
    let repo = temp_dir("repo-lifecycle");
    let workspace_base = temp_dir("workspaces-lifecycle");
    unsafe {
        std::env::set_var("HC_WORKSPACE_ROOT", &workspace_base);
    }
    git(&repo, &["init"]);
    git(&repo, &["config", "user.email", "review@example.com"]);
    git(&repo, &["config", "user.name", "Review Bot"]);
    fs::write(repo.join("README.md"), "lifecycle\n").expect("readme");
    git(&repo, &["add", "README.md"]);
    git(&repo, &["commit", "-m", "init"]);

    let workspace = shared_provision_task_workspace(
        repo.to_str().expect("utf8"),
        "task_ready",
        Some("."),
    )
    .expect("workspace");

    // The worktree record should have lifecycle_state = "in_use" (default on insert).
    // task_ready is seeded with project_id = "proj_demo" in the shared store.
    let rows = shared_inventory_for_project("proj_demo").expect("inventory");
    let row = rows
        .iter()
        .find(|r| r.worktree_id == workspace.worktree_id)
        .expect("worktree row in inventory");

    assert_eq!(row.disposition, InventoryDisposition::InUse);

    unsafe {
        std::env::remove_var("HC_WORKSPACE_ROOT");
    }
}
