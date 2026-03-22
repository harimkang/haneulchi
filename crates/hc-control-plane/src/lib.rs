//! Orchestrator and snapshot scaffold.

mod attention;
mod commands;
mod session_projection;
mod snapshot;
mod tasks;
mod workflow_projection;

pub use commands::{ControlPlaneError, ControlPlaneState, reload_workflow, validate_workflow};
pub use snapshot::{SnapshotSeed, project_snapshot};
pub use tasks::{
    TaskBoardColumnSummary, TaskBoardError, TaskBoardMutationResult, TaskBoardProjection,
    TaskBoardService, reset_task_board_for_tests, shared_task_board_projection, shared_task_move,
};
