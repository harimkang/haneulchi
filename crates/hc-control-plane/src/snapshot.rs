use hc_domain::{
    AppSnapshot, AppSnapshotMeta, AppState, OpsSummary, ProjectSummary, RetryQueueEntry,
    SessionFocusState, SessionRuntimeState, SessionSummary, TrackerStatus, WarningSummary,
    WorkflowRuntimeStatus,
};

use crate::attention::derive_attention;

#[derive(Clone, Debug)]
pub struct SnapshotSeed {
    pub workflow: WorkflowRuntimeStatus,
    pub tracker: TrackerStatus,
    pub projects: Vec<ProjectSummary>,
    pub sessions: Vec<SessionSummary>,
    pub retry_queue: Vec<RetryQueueEntry>,
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
    let attention = derive_attention(&seed.workflow, &seed.sessions, &seed.retry_queue);
    let queued_claim_count = seed
        .sessions
        .iter()
        .filter(|session| session.claim_state == hc_domain::ClaimState::Claimed)
        .count() as u32;

    let mut snapshot = AppSnapshot::new(seed.workflow.clone(), seed.tracker.clone())
        .with_automation(OpsSummary {
            status: "running".to_string(),
            cadence_ms: 15_000,
            last_tick_at: Some("2026-03-22T00:00:00Z".to_string()),
            last_reconcile_at: None,
            running_slots,
            max_slots: running_slots.max(1),
            retry_due_count: seed.retry_queue.len() as u32,
            queued_claim_count,
            paused: false,
        })
        .with_app_state(AppState {
            active_route: "project_focus".to_string(),
            focused_session_id,
            degraded_flags: Vec::new(),
        });

    snapshot.meta = AppSnapshotMeta {
        snapshot_rev: 1,
        runtime_rev: 1,
        projection_rev: 1,
        snapshot_at: Some("2026-03-22T00:00:00Z".to_string()),
    };
    snapshot.projects = seed.projects;
    snapshot.sessions = seed.sessions;
    snapshot.attention = attention;
    snapshot.retry_queue = seed.retry_queue;
    snapshot.warnings = Vec::<WarningSummary>::new();

    Ok(snapshot)
}

pub fn project_snapshot(seed: SnapshotSeed) -> AppSnapshot {
    build_authoritative_snapshot(seed).expect("snapshot builder")
}
