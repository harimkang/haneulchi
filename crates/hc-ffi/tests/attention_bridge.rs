use std::sync::Mutex;

use hc_control_plane::reset_shared_control_plane_snapshot_for_tests;
use hc_domain::{AppSnapshot, AttentionSummary, TrackerStatus, WorkflowHealth, WorkflowRuntimeStatus};
use hc_ffi::{attention_dismiss_json, attention_resolve_json, attention_snooze_json, reset_test_state, state_snapshot_json};
use serde_json::Value;

static TEST_LOCK: Mutex<()> = Mutex::new(());

fn snapshot_with_attention() -> AppSnapshot {
    let mut snapshot = AppSnapshot::new(
        WorkflowRuntimeStatus {
            state: WorkflowHealth::Ok,
            path: "/tmp/demo/WORKFLOW.md".to_string(),
            last_good_hash: Some("sha256:attention".to_string()),
            last_reload_at: Some("2026-03-23T13:00:00Z".to_string()),
            last_error: None,
        },
        TrackerStatus {
            state: "local_only".to_string(),
            last_sync_at: None,
            health: "ok".to_string(),
        },
    );
    snapshot.attention = vec![
        AttentionSummary {
            attention_id: "att_waiting".to_string(),
            kind: "waiting_input".to_string(),
            project_id: "proj_demo".to_string(),
            session_id: Some("ses_waiting".to_string()),
            task_id: None,
            title: "Needs input".to_string(),
            summary: "Operator answer required.".to_string(),
            created_at: Some("2026-03-23T13:00:01Z".to_string()),
            severity: "warn".to_string(),
            action_hint: Some("focus_session".to_string()),
        },
        AttentionSummary {
            attention_id: "att_review".to_string(),
            kind: "review_ready".to_string(),
            project_id: "proj_demo".to_string(),
            session_id: None,
            task_id: Some("task_review".to_string()),
            title: "Review ready".to_string(),
            summary: "Evidence pack ready.".to_string(),
            created_at: Some("2026-03-23T13:00:02Z".to_string()),
            severity: "info".to_string(),
            action_hint: Some("open_review".to_string()),
        },
        AttentionSummary {
            attention_id: "att_snooze".to_string(),
            kind: "retry_due".to_string(),
            project_id: "proj_demo".to_string(),
            session_id: None,
            task_id: Some("task_retry".to_string()),
            title: "Retry due".to_string(),
            summary: "Retry queue is ready.".to_string(),
            created_at: Some("2026-03-23T13:00:03Z".to_string()),
            severity: "warn".to_string(),
            action_hint: Some("open_retry_queue".to_string()),
        },
    ];
    snapshot
}

#[test]
fn attention_bridge_resolve_dismiss_and_snooze_mutate_exported_snapshot() {
    let _guard = TEST_LOCK.lock().unwrap();
    reset_test_state();
    reset_shared_control_plane_snapshot_for_tests(snapshot_with_attention());

    attention_resolve_json("att_review").expect("resolve attention");
    attention_dismiss_json("att_waiting").expect("dismiss attention");
    attention_snooze_json("att_snooze").expect("snooze attention");

    let snapshot: Value =
        serde_json::from_str(&state_snapshot_json().expect("snapshot json")).expect("valid json");
    let attention = snapshot["attention"].as_array().expect("attention array");

    assert!(attention.is_empty());
}
