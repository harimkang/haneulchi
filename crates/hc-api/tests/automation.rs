use hc_api::reconcile_now_json;
use hc_control_plane::{
    reset_shared_control_plane_snapshot_for_tests, reset_task_board_for_tests,
    shared_set_automation_mode,
};
use hc_domain::{
    AppSnapshot, AppSnapshotMeta, AppState, ClaimState, OpsSummary, RetryQueueEntry,
    TaskAutomationMode, TrackerStatus, WorkflowHealth, WorkflowRuntimeStatus,
};

#[test]
fn reconcile_now_uses_current_shared_snapshot_slot_state() {
    reset_task_board_for_tests();
    shared_set_automation_mode("task_ready", TaskAutomationMode::AutoEligible)
        .expect("automation mode");
    let workflow = WorkflowRuntimeStatus {
        state: WorkflowHealth::Ok,
        path: "/tmp/demo/WORKFLOW.md".to_string(),
        last_good_hash: Some("sha256:abc123".to_string()),
        last_reload_at: Some("2026-03-23T09:29:00Z".to_string()),
        last_error: None,
    };
    let tracker = TrackerStatus {
        state: "local_only".to_string(),
        last_sync_at: None,
        health: "ok".to_string(),
    };
    let mut snapshot = AppSnapshot::new(workflow, tracker)
        .with_automation(OpsSummary {
            status: "running".to_string(),
            cadence_ms: 15_000,
            last_tick_at: Some("2026-03-23T09:29:30Z".to_string()),
            last_reconcile_at: None,
            running_slots: 1,
            max_slots: 1,
            retry_due_count: 0,
            queued_claim_count: 0,
            paused: false,
        })
        .with_app_state(AppState {
            active_route: "project_focus".to_string(),
            focused_session_id: None,
            degraded_flags: vec![],
        });
    snapshot.meta = AppSnapshotMeta {
        snapshot_rev: 9,
        runtime_rev: 9,
        projection_rev: 9,
        snapshot_at: Some("2026-03-23T09:30:00Z".to_string()),
    };
    snapshot.retry_queue = vec![RetryQueueEntry {
        task_id: "task_retry".to_string(),
        project_id: "proj_demo".to_string(),
        attempt: 1,
        reason_code: "adapter_timeout".to_string(),
        due_at: Some("2026-03-23T09:31:00Z".to_string()),
        backoff_ms: 30_000,
        claim_state: ClaimState::Claimed,
        retry_state: hc_domain::RetryState::Due,
    }];
    reset_shared_control_plane_snapshot_for_tests(snapshot);

    let payload = reconcile_now_json(None).expect("reconcile payload");
    let value: serde_json::Value = serde_json::from_str(&payload).expect("json");

    assert_eq!(value["result"]["launched_task_ids"], serde_json::json!([]));
    assert_eq!(
        value["result"]["queued"][0]["reason_code"],
        "slot_capacity_exhausted"
    );
    assert_eq!(
        value["reconcile"]["released_session_ids"],
        serde_json::json!([])
    );
    assert_eq!(value["snapshot"]["attention"][0]["kind"], "retry_due");
}
