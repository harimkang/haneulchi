//! Orchestrator and snapshot scaffold.

mod adapter_watch;
mod attention;
mod commands;
mod control_tower;
mod dispatch;
mod eligibility;
pub mod inventory;
mod orchestrator;
pub mod persistence;
mod reconcile;
pub mod recovery;
mod retry_queue;
mod reviews;
mod scheduler;
mod session_projection;
pub mod settings;
mod shared_store;
mod snapshot;
mod tasks;
mod timeline;
mod workflow_projection;
mod worktrees;

pub use adapter_watch::{AdapterWatchSummary, adapter_watch_for_session};
pub use commands::{
    ControlPlaneError, ControlPlaneState, lock_shared_control_plane, prepare_isolated_launch,
    reload_workflow, reset_shared_control_plane_for_tests,
    reset_shared_control_plane_snapshot_for_tests, shared_inventory_for_project,
    shared_inventory_summary, shared_set_worktree_pinned, shared_update_worktree_lifecycle,
    validate_workflow,
};
pub use control_tower::{
    ControlTowerProjection, ProjectCardProjection, RecentArtifactProjection,
    build_control_tower_projection,
};
pub use dispatch::{DispatchEvent, DispatchLifecycleState, dispatch_snapshot, dispatch_to_session};
pub use eligibility::{EligibilityContext, evaluate_task_eligibility};
pub use orchestrator::AutomationStatusSummary;
pub use persistence::{
    shared_list_recoverable_sessions, shared_load_app_state, shared_load_latest_layout,
    shared_save_app_state, shared_upsert_layout, shared_upsert_session_metadata,
};
pub use reconcile::{ReconcileReport, reconcile_snapshot};
pub use recovery::{
    RecoveryContext, detect_degraded_issues, recovery_action_for_issue,
    workflow_health_to_recovery_issue,
};
pub use retry_queue::{DispatchFailureClass, classify_dispatch_failure};
pub use reviews::{
    ReviewDecision, ReviewDecisionResult, ReviewQueueError, ReviewQueueItem, ReviewQueueProjection,
    ReviewQueueService, reset_review_queue_for_tests, shared_review_decision,
    shared_review_ready_projection,
};
pub use scheduler::{
    BoundedScheduler, SchedulerIssue, SchedulerResult, SchedulerTask, shared_scheduler_tick,
};
pub use settings::{
    shared_delete_secret_ref, shared_get_terminal_settings, shared_list_secret_refs,
    shared_resolve_secret_env, shared_resolve_secret_env_filtered, shared_upsert_secret_ref,
    shared_upsert_terminal_settings,
};
pub use snapshot::{
    SnapshotBuildError, SnapshotSeed, build_authoritative_snapshot, project_snapshot,
};
pub use tasks::{
    TaskBoardColumnSummary, TaskBoardError, TaskBoardMutationResult, TaskBoardProjection,
    TaskBoardService, reset_task_board_for_tests, shared_attach_session, shared_automation_details,
    shared_create_task, shared_detach_session, shared_set_automation_mode, shared_task,
    shared_task_board_projection, shared_task_drawer, shared_task_move,
};
pub use timeline::{TaskTimelineEntry, project_task_timeline};
pub use worktrees::{
    ProvisionedTaskWorkspace, WorktreeProvisionError, shared_provision_task_workspace,
    shared_set_worktree_pinned as worktree_set_pinned,
    shared_update_worktree_lifecycle as worktree_update_lifecycle,
};
