use hc_domain::{
    AppSnapshot, AppSnapshotMeta, AppState, OpsSummary, ProjectSummary, SessionFocusState,
    SessionRuntimeState, SessionSummary, TrackerStatus, WarningSummary, WorkflowRuntimeStatus,
};

use crate::attention::derive_attention;

#[derive(Clone, Debug)]
pub struct SnapshotSeed {
    pub workflow: WorkflowRuntimeStatus,
    pub tracker: TrackerStatus,
    pub projects: Vec<ProjectSummary>,
    pub sessions: Vec<SessionSummary>,
}

pub fn project_snapshot(seed: SnapshotSeed) -> AppSnapshot {
    let running_slots = seed
        .sessions
        .iter()
        .filter(|session| session.runtime_state == SessionRuntimeState::Running)
        .count() as u32;
    let focused_session_id = seed
        .sessions
        .iter()
        .find(|session| session.focus_state == SessionFocusState::Focused)
        .map(|session| session.session_id.clone())
        .or_else(|| seed.sessions.first().map(|session| session.session_id.clone()));
    let attention = derive_attention(&seed.workflow, &seed.sessions);

    AppSnapshot {
        meta: AppSnapshotMeta {
            snapshot_rev: 1,
            runtime_rev: 1,
            projection_rev: 1,
            snapshot_at: Some("2026-03-22T00:00:00Z".to_string()),
        },
        ops: OpsSummary {
            running_slots,
            max_slots: running_slots.max(1),
            retry_queue_count: 0,
            workflow_health: seed.workflow.state,
        },
        workflow: seed.workflow,
        tracker: seed.tracker,
        app: AppState {
            active_route: "project_focus".to_string(),
            focused_session_id,
            degraded_flags: Vec::new(),
        },
        projects: seed.projects,
        sessions: seed.sessions,
        attention,
        retry_queue: Vec::new(),
        warnings: Vec::<WarningSummary>::new(),
    }
}
