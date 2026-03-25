use hc_control_plane::{
    DispatchFailureClass, DispatchLifecycleState, classify_dispatch_failure, dispatch_snapshot,
    dispatch_to_session,
};
use hc_domain::{
    AppSnapshot, SessionRuntimeState, SessionSummary, TrackerStatus, WorkflowHealth,
    WorkflowRuntimeStatus,
};

#[test]
fn dispatch_lifecycle_reports_queued_started_and_sent_for_dispatchable_session() {
    let mut session = SessionSummary::new("ses_dispatch", "proj_demo", "Dispatchable");
    session.runtime_state = SessionRuntimeState::Running;
    session.dispatch_state = "dispatchable".to_string();
    session.manual_control = "none".to_string();

    let events = dispatch_to_session(&mut session, true, "rerun tests");

    assert_eq!(
        events.iter().map(|event| event.state).collect::<Vec<_>>(),
        vec![
            DispatchLifecycleState::Queued,
            DispatchLifecycleState::Started,
            DispatchLifecycleState::Sent,
        ]
    );
    assert_eq!(session.dispatch_state, "dispatchable");
    assert!(
        events
            .iter()
            .all(|event| event.target_session_id == "ses_dispatch")
    );
}

#[test]
fn dispatch_fails_for_stale_or_takeover_targets_and_classifies_retryability() {
    let mut stale = SessionSummary::new("ses_stale", "proj_demo", "Stale target");
    stale.runtime_state = SessionRuntimeState::Running;
    stale.dispatch_state = "dispatchable".to_string();

    let stale_events = dispatch_to_session(&mut stale, false, "rerun tests");
    assert_eq!(
        stale_events.last().expect("stale dispatch failure").state,
        DispatchLifecycleState::Failed
    );
    assert_eq!(
        stale_events
            .last()
            .and_then(|event| event.reason_code.as_deref()),
        Some("stale_target_session")
    );
    assert_eq!(stale.dispatch_state, "dispatch_failed");

    let mut manual = SessionSummary::new("ses_manual", "proj_demo", "Manual takeover");
    manual.runtime_state = SessionRuntimeState::Running;
    manual.dispatch_state = "dispatchable".to_string();
    manual.manual_control = "takeover".to_string();

    let manual_events = dispatch_to_session(&mut manual, true, "rerun tests");
    assert_eq!(
        manual_events.last().expect("manual dispatch failure").state,
        DispatchLifecycleState::Failed
    );
    assert_eq!(
        manual_events
            .last()
            .and_then(|event| event.reason_code.as_deref()),
        Some("manual_takeover_active")
    );

    assert_eq!(
        classify_dispatch_failure("adapter_timeout"),
        DispatchFailureClass::Retryable
    );
    assert_eq!(
        classify_dispatch_failure("workflow_invalid"),
        DispatchFailureClass::NonRetryable
    );
    assert_eq!(
        classify_dispatch_failure("stale_target_session"),
        DispatchFailureClass::NonRetryable
    );
}

#[test]
fn dispatch_snapshot_emits_failed_notification_without_silent_reroute() {
    let mut snapshot = AppSnapshot::new(
        WorkflowRuntimeStatus {
            state: WorkflowHealth::Ok,
            path: "/tmp/demo/WORKFLOW.md".to_string(),
            last_good_hash: Some("sha256:dispatch".to_string()),
            last_reload_at: Some("2026-03-23T16:00:00Z".to_string()),
            last_error: None,
        },
        TrackerStatus {
            state: "local_only".to_string(),
            last_sync_at: None,
            health: "ok".to_string(),
        },
    );
    let mut session = SessionSummary::new("ses_stale", "proj_demo", "Stale target");
    session.runtime_state = SessionRuntimeState::Running;
    session.dispatch_state = "dispatchable".to_string();
    snapshot.sessions = vec![session];

    let events = dispatch_snapshot(
        &mut snapshot,
        "ses_stale",
        Some("task_review"),
        false,
        "rerun tests",
    );

    assert_eq!(
        events.last().expect("failed dispatch").state,
        DispatchLifecycleState::Failed
    );
    assert_eq!(
        events.last().and_then(|event| event.reason_code.as_deref()),
        Some("stale_target_session")
    );
    assert_eq!(
        events.last().map(|event| event.task_id.as_deref()),
        Some(Some("task_review"))
    );
    assert_eq!(snapshot.attention.len(), 1);
    assert_eq!(snapshot.attention[0].kind, "session_error");
    assert_eq!(
        snapshot.attention[0].session_id.as_deref(),
        Some("ses_stale")
    );
}
