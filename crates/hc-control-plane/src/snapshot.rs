use hc_domain::{
    AppSnapshot, AppSnapshotMeta, AppState, OpsSummary, OrchestratorRuntime, ProjectSummary,
    RetryQueueEntry, RetryState, SessionFocusState, SessionRuntimeState, SessionSummary,
    TrackerStatus, WarningSummary, WorkflowRuntimeStatus, time::now_iso8601,
};

use crate::attention::derive_attention;
use crate::recovery::{RecoveryContext, detect_degraded_issues};

#[derive(Clone, Debug)]
pub struct SnapshotSeed {
    pub orchestrator_runtime: OrchestratorRuntime,
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
        .or_else(|| {
            seed.sessions
                .first()
                .map(|session| session.session_id.clone())
        });
    let attention = derive_attention(&seed.workflow, &seed.sessions, &seed.retry_queue);
    let queued_claim_count = seed
        .sessions
        .iter()
        .filter(|session| session.claim_state == hc_domain::ClaimState::Claimed)
        .count() as u32;
    let retry_due_count = seed
        .retry_queue
        .iter()
        .filter(|entry| entry.retry_state == RetryState::Due)
        .count() as u32;

    // Detect degraded issues from the current workflow health.  Details are
    // reduced to issue codes only — secret values must never appear in flags.
    let recovery_context = RecoveryContext {
        workflow_health: seed.workflow.state,
        ..Default::default()
    };
    let degraded_flags: Vec<String> = detect_degraded_issues(&recovery_context)
        .into_iter()
        .map(|issue| issue.issue_code)
        .collect();

    let mut snapshot = AppSnapshot::new(seed.workflow.clone(), seed.tracker.clone())
        .with_automation(OpsSummary {
            status: "running".to_string(),
            cadence_ms: seed.orchestrator_runtime.cadence_ms,
            last_tick_at: seed.orchestrator_runtime.last_tick_at.clone(),
            last_reconcile_at: seed.orchestrator_runtime.last_reconcile_at.clone(),
            running_slots: seed.orchestrator_runtime.running_slots.max(running_slots),
            max_slots: seed.orchestrator_runtime.max_slots.max(1),
            retry_due_count,
            queued_claim_count,
            paused: false,
        })
        .with_app_state(AppState {
            active_route: "project_focus".to_string(),
            focused_session_id,
            degraded_flags,
        });

    snapshot.meta = AppSnapshotMeta {
        snapshot_rev: 1,
        runtime_rev: 1,
        projection_rev: 1,
        snapshot_at: Some(now_iso8601()),
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
