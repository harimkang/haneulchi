use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::tasks::lock_shared_board_service;

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

    let workspace_root = repo_root.join("worktrees").join(sanitize(task_id));
    fs::create_dir_all(&workspace_root)
        .map_err(|error| WorktreeProvisionError::ProvisioningFailed(error.to_string()))?;

    let service =
        lock_shared_board_service().map_err(|_| WorktreeProvisionError::TaskBoardUnavailable)?;
    service
        .provision_task_workspace(
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
