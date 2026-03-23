use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};

use crate::shared_store::lock_shared_store;
use crate::tasks::provision_task_workspace_for_store;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ProvisionedTaskWorkspace {
    pub task_id: String,
    pub worktree_id: String,
    pub workspace_root: String,
    pub base_root: String,
    pub branch_name: String,
}

#[derive(Debug, thiserror::Error)]
pub enum WorktreeProvisionError {
    #[error("task board unavailable")]
    TaskBoardUnavailable,
    #[error("non git project: {0}")]
    NonGitProject(String),
    #[error("worktree provisioning failed: {0}")]
    ProvisioningFailed(String),
    #[error(transparent)]
    TaskBoard(#[from] crate::tasks::TaskBoardError),
}

pub fn shared_provision_task_workspace(
    project_root: &str,
    task_id: &str,
    base_root: Option<&str>,
) -> Result<ProvisionedTaskWorkspace, WorktreeProvisionError> {
    let repo_root = Path::new(project_root);
    if !repo_root.join(".git").exists() {
        return Err(WorktreeProvisionError::NonGitProject(
            project_root.to_string(),
        ));
    }

    let workspace_root = workspace_root_for_repo(repo_root, task_id);
    materialize_git_worktree(repo_root, &workspace_root, task_id)?;

    let store = lock_shared_store().map_err(|_| WorktreeProvisionError::TaskBoardUnavailable)?;
    provision_task_workspace_for_store(
        &store,
        task_id,
        project_root,
        workspace_root.to_str().ok_or_else(|| {
            WorktreeProvisionError::ProvisioningFailed("workspace path is not utf8".to_string())
        })?,
        base_root.unwrap_or("."),
    )
    .map_err(Into::into)
}

fn sanitize(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '-' || character == '_' {
                character
            } else {
                '-'
            }
        })
        .collect()
}

fn workspace_root_for_repo(repo_root: &Path, task_id: &str) -> PathBuf {
    let project_slug = repo_root
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("project");
    workspace_root_base().join(project_slug).join(sanitize(task_id))
}

fn workspace_root_base() -> PathBuf {
    if let Some(path) = std::env::var_os("HC_WORKSPACE_ROOT") {
        return PathBuf::from(path);
    }

    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    home.join("Library")
        .join("Application Support")
        .join("Haneulchi")
        .join("workspaces")
}

fn materialize_git_worktree(
    repo_root: &Path,
    workspace_root: &Path,
    task_id: &str,
) -> Result<(), WorktreeProvisionError> {
    if workspace_root.join(".git").exists() {
        return Ok(());
    }
    if workspace_root.exists() {
        fs::remove_dir_all(workspace_root)
            .map_err(|error| WorktreeProvisionError::ProvisioningFailed(error.to_string()))?;
    }
    if let Some(parent) = workspace_root.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| WorktreeProvisionError::ProvisioningFailed(error.to_string()))?;
    }

    let branch_name = format!("hc/{}", sanitize(task_id));
    let branch_exists = Command::new("git")
        .current_dir(repo_root)
        .args([
            "show-ref",
            "--verify",
            "--quiet",
            &format!("refs/heads/{branch_name}"),
        ])
        .status()
        .map_err(|error| WorktreeProvisionError::ProvisioningFailed(error.to_string()))?
        .success();

    let status = if branch_exists {
        Command::new("git")
            .current_dir(repo_root)
            .args([
                "worktree",
                "add",
                workspace_root
                    .to_str()
                    .ok_or_else(|| WorktreeProvisionError::ProvisioningFailed("workspace path is not utf8".to_string()))?,
                &branch_name,
            ])
            .status()
    } else {
        Command::new("git")
            .current_dir(repo_root)
            .args([
                "worktree",
                "add",
                "-b",
                &branch_name,
                workspace_root
                    .to_str()
                    .ok_or_else(|| WorktreeProvisionError::ProvisioningFailed("workspace path is not utf8".to_string()))?,
                "HEAD",
            ])
            .status()
    }
    .map_err(|error| WorktreeProvisionError::ProvisioningFailed(error.to_string()))?;

    if !status.success() {
        return Err(WorktreeProvisionError::ProvisioningFailed(format!(
            "git worktree add failed for {}",
            workspace_root.display()
        )));
    }

    Ok(())
}
