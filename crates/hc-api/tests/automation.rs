use hc_api::reconcile_now_json;
use hc_control_plane::{
    reset_shared_control_plane_snapshot_for_tests, reset_task_board_for_tests,
    shared_set_automation_mode,
};
use hc_domain::{
    AppSnapshot, AppSnapshotMeta, AppState, OpsSummary, TaskAutomationMode, TrackerStatus,
    WorkflowHealth, WorkflowRuntimeStatus,
};

#[test]
fn reconcile_now_uses_current_shared_snapshot_slot_state() {
    reset_task_board_for_tests();
    shared_set_automation_mode("task_ready", TaskAutomationMode::AutoEligible)
        .expect("automation mode");
    reset_shared_control_plane_snapshot_for_tests(AppSnapshot {
        meta: AppSnapshotMeta {
            snapshot_rev: 9,
            runtime_rev: 9,
            projection_rev: 9,
            snapshot_at: Some("2026-03-23T09:30:00Z".to_string()),
        },
        ops: OpsSummary {
            cadence_ms: 15_000,
            last_tick_at: Some("2026-03-23T09:29:30Z".to_string()),
            last_reconcile_at: None,
            running_slots: 1,
            max_slots: 1,
            retry_queue_count: 0,
            queued_claim_count: 0,
            workflow_health: WorkflowHealth::Ok,
            tracker_health: "ok".to_string(),
            paused: false,
        },
        workflow: WorkflowRuntimeStatus {
            state: WorkflowHealth::Ok,
            path: "/tmp/demo/WORKFLOW.md".to_string(),
            last_good_hash: Some("sha256:abc123".to_string()),
            last_reload_at: Some("2026-03-23T09:29:00Z".to_string()),
            last_error: None,
        },
        tracker: TrackerStatus {
            state: "local_only".to_string(),
            last_sync_at: None,
            health: "ok".to_string(),
        },
        app: AppState {
            active_route: "project_focus".to_string(),
            focused_session_id: None,
            degraded_flags: vec![],
        },
        projects: vec![],
        sessions: vec![],
        attention: vec![],
        retry_queue: vec![],
        warnings: vec![],
    });

    let payload = reconcile_now_json().expect("reconcile payload");
    let value: serde_json::Value = serde_json::from_str(&payload).expect("json");

    assert_eq!(value["result"]["launched_task_ids"], serde_json::json!([]));
    assert_eq!(
        value["result"]["queued"][0]["reason_code"],
        "slot_capacity_exhausted"
    );
}
