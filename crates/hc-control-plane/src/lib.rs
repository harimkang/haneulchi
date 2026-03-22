//! Orchestrator and snapshot scaffold.

mod attention;
mod commands;
mod session_projection;
mod snapshot;
mod tasks;
mod workflow_projection;
mod worktrees;

pub use commands::{ControlPlaneError, ControlPlaneState, reload_workflow, validate_workflow};
pub use snapshot::{SnapshotSeed, project_snapshot};
pub use tasks::{
    TaskBoardColumnSummary, TaskBoardError, TaskBoardMutationResult, TaskBoardProjection,
    TaskBoardService, reset_task_board_for_tests, shared_attach_session, shared_detach_session,
    shared_task, shared_task_board_projection, shared_task_drawer, shared_task_move,
};
pub use worktrees::{
    ProvisionedTaskWorkspace, WorktreeProvisionError, shared_provision_task_workspace,
};
