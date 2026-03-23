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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SnapshotBuildError {
    SnapshotUnavailable,
}

pub fn build_authoritative_snapshot(seed: SnapshotSeed) -> Result<AppSnapshot, SnapshotBuildError> {
    if seed.tracker.health == "snapshot_unavailable" {
        return Err(SnapshotBuildError::SnapshotUnavailable);
    }

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

    Ok(AppSnapshot {
        meta: AppSnapshotMeta {
            snapshot_rev: 1,
            runtime_rev: 1,
            projection_rev: 1,
            snapshot_at: Some("2026-03-22T00:00:00Z".to_string()),
        },
        ops: OpsSummary {
            cadence_ms: 15_000,
            last_tick_at: Some("2026-03-22T00:00:00Z".to_string()),
            last_reconcile_at: None,
            running_slots,
            max_slots: running_slots.max(1),
            retry_queue_count: 0,
            queued_claim_count: 0,
            workflow_health: seed.workflow.state,
            tracker_health: seed.tracker.health.clone(),
            paused: false,
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
    })
}

pub fn project_snapshot(seed: SnapshotSeed) -> AppSnapshot {
    build_authoritative_snapshot(seed).expect("snapshot builder")
}
