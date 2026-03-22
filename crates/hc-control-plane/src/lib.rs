//! Orchestrator and snapshot scaffold.

mod attention;
mod commands;
mod eligibility;
mod orchestrator;
mod reviews;
mod session_projection;
mod snapshot;
mod tasks;
mod timeline;
mod workflow_projection;
mod worktrees;

pub use commands::{ControlPlaneError, ControlPlaneState, reload_workflow, validate_workflow};
pub use eligibility::{EligibilityContext, evaluate_task_eligibility};
pub use orchestrator::AutomationStatusSummary;
pub use reviews::{
    ReviewDecision, ReviewDecisionResult, ReviewQueueError, ReviewQueueProjection,
    ReviewQueueService, reset_review_queue_for_tests, shared_review_decision,
    shared_review_ready_projection,
};
pub use snapshot::{SnapshotBuildError, SnapshotSeed, build_authoritative_snapshot, project_snapshot};
pub use tasks::{
    TaskBoardColumnSummary, TaskBoardError, TaskBoardMutationResult, TaskBoardProjection,
    TaskBoardService, reset_task_board_for_tests, shared_attach_session,
    shared_automation_details, shared_detach_session, shared_set_automation_mode, shared_task,
    shared_task_board_projection, shared_task_drawer, shared_task_move,
};
pub use timeline::{TaskTimelineEntry, project_task_timeline};
pub use worktrees::{
    ProvisionedTaskWorkspace, WorktreeProvisionError, shared_provision_task_workspace,
};
