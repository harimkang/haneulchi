use hc_control_plane::reconcile_snapshot;
use hc_domain::{
    AppSnapshot, ClaimState, RetryQueueEntry, SessionRuntimeState, SessionSummary, TrackerStatus,
    WorkflowHealth, WorkflowRuntimeStatus,
};

fn snapshot() -> AppSnapshot {
    AppSnapshot::new(
        WorkflowRuntimeStatus {
            state: WorkflowHealth::Ok,
            path: "/tmp/demo/WORKFLOW.md".to_string(),
            last_good_hash: Some("sha256:reconcile".to_string()),
            last_reload_at: Some("2026-03-23T11:00:00Z".to_string()),
            last_error: None,
        },
        TrackerStatus {
            state: "local_only".to_string(),
            last_sync_at: None,
            health: "ok".to_string(),
        },
    )
}

#[test]
fn reconcile_releases_stale_claims_and_is_idempotent_without_touching_takeover_sessions() {
    let mut snapshot = snapshot();

    let mut stale = SessionSummary::new("ses_stale", "proj_demo", "Stale claim");
    stale.runtime_state = SessionRuntimeState::Running;
    stale.claim_state = ClaimState::Stale;
    stale.dispatch_state = "dispatchable".to_string();

    let mut manual = SessionSummary::new("ses_manual", "proj_demo", "Manual takeover");
    manual.runtime_state = SessionRuntimeState::WaitingInput;
    manual.claim_state = ClaimState::Stale;
    manual.dispatch_state = "dispatchable".to_string();
    manual.manual_control = "takeover".to_string();

    snapshot.sessions = vec![stale, manual];
    snapshot.retry_queue = vec![RetryQueueEntry {
        task_id: "task_retry".to_string(),
        project_id: "proj_demo".to_string(),
        attempt: 2,
        reason_code: "adapter_timeout".to_string(),
        due_at: Some("2026-03-23T11:05:00Z".to_string()),
        backoff_ms: 60_000,
        claim_state: ClaimState::Claimed,
        retry_state: hc_domain::RetryState::Due,
    }];
    let initial_rev = snapshot.meta.projection_rev;

    let first = reconcile_snapshot(&mut snapshot);
    assert_eq!(first.released_session_ids, vec!["ses_stale"]);
    assert_eq!(first.skipped_manual_takeover_ids, vec!["ses_manual"]);
    assert!(first.cleaned_exited_ids.is_empty());
    assert_eq!(snapshot.sessions[0].claim_state, ClaimState::Released);
    assert_eq!(snapshot.sessions[0].dispatch_state, "not_dispatchable");
    assert_eq!(snapshot.sessions[1].claim_state, ClaimState::Stale);
    assert_eq!(snapshot.meta.projection_rev, initial_rev + 1);
    assert!(snapshot.attention.iter().any(|item| item.kind == "retry_due"));

    let second = reconcile_snapshot(&mut snapshot);
    assert!(second.released_session_ids.is_empty());
    assert_eq!(second.skipped_manual_takeover_ids, vec!["ses_manual"]);
}

#[test]
fn reconcile_cleans_exited_sessions_with_active_claims() {
    let mut snapshot = snapshot();

    let mut exited_claimed = SessionSummary::new("ses_exited", "proj_demo", "Exited session");
    exited_claimed.runtime_state = SessionRuntimeState::Exited;
    exited_claimed.claim_state = ClaimState::Claimed;
    exited_claimed.dispatch_state = "dispatchable".to_string();

    let mut running_ok = SessionSummary::new("ses_running", "proj_demo", "Running session");
    running_ok.runtime_state = SessionRuntimeState::Running;
    running_ok.claim_state = ClaimState::Claimed;
    running_ok.dispatch_state = "dispatchable".to_string();

    snapshot.sessions = vec![exited_claimed, running_ok];
    let initial_rev = snapshot.meta.projection_rev;

    let report = reconcile_snapshot(&mut snapshot);
    assert_eq!(report.cleaned_exited_ids, vec!["ses_exited"]);
    assert!(report.released_session_ids.is_empty());
    assert_eq!(snapshot.sessions[0].claim_state, ClaimState::Released);
    assert_eq!(snapshot.sessions[0].dispatch_reason.as_deref(), Some("session_exited"));
    assert_eq!(snapshot.sessions[1].claim_state, ClaimState::Claimed);
    assert_eq!(snapshot.meta.projection_rev, initial_rev + 1);
    assert!(snapshot.ops.automation.last_reconcile_at.is_some());
}
