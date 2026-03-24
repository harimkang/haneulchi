//! Orchestrator and snapshot scaffold.

mod adapter_watch;
mod attention;
mod commands;
mod control_tower;
mod dispatch;
mod eligibility;
mod orchestrator;
mod reconcile;
mod reviews;
mod retry_queue;
mod scheduler;
mod session_projection;
mod shared_store;
mod snapshot;
mod tasks;
mod timeline;
mod workflow_projection;
mod worktrees;

pub use commands::{
    ControlPlaneError, ControlPlaneState, lock_shared_control_plane,
    reset_shared_control_plane_for_tests, reset_shared_control_plane_snapshot_for_tests,
    reload_workflow, validate_workflow,
};
pub use adapter_watch::{AdapterWatchSummary, adapter_watch_for_session};
pub use control_tower::{
    ControlTowerProjection, ProjectCardProjection, RecentArtifactProjection,
    build_control_tower_projection,
};
pub use dispatch::{DispatchEvent, DispatchLifecycleState, dispatch_snapshot, dispatch_to_session};
pub use eligibility::{EligibilityContext, evaluate_task_eligibility};
pub use orchestrator::AutomationStatusSummary;
pub use reconcile::{ReconcileReport, reconcile_snapshot};
pub use reviews::{
    ReviewDecision, ReviewDecisionResult, ReviewQueueError, ReviewQueueItem,
    ReviewQueueProjection, ReviewQueueService, reset_review_queue_for_tests,
    shared_review_decision, shared_review_ready_projection,
};
pub use retry_queue::{DispatchFailureClass, classify_dispatch_failure};
pub use scheduler::{BoundedScheduler, SchedulerIssue, SchedulerResult, SchedulerTask, shared_scheduler_tick};
pub use snapshot::{SnapshotBuildError, SnapshotSeed, build_authoritative_snapshot, project_snapshot};
pub use tasks::{
    TaskBoardColumnSummary, TaskBoardError, TaskBoardMutationResult, TaskBoardProjection,
    TaskBoardService, reset_task_board_for_tests, shared_attach_session,
    shared_automation_details, shared_detach_session, shared_set_automation_mode, shared_task,
    shared_create_task, shared_task_board_projection, shared_task_drawer, shared_task_move,
};
pub use timeline::{TaskTimelineEntry, project_task_timeline};
pub use worktrees::{
    ProvisionedTaskWorkspace, WorktreeProvisionError, shared_provision_task_workspace,
};
