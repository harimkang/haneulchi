use std::sync::Mutex;

use hc_control_plane::reset_shared_control_plane_snapshot_for_tests;
use hc_domain::{
    AppSnapshot, SessionRuntimeState, SessionSummary, TrackerStatus, WorkflowHealth,
    WorkflowRuntimeStatus,
};
use hc_ffi::{dispatch_send_json, reset_test_state, state_snapshot_json};
use serde_json::Value;

static TEST_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn dispatch_bridge_updates_snapshot_and_attention_for_failed_target() {
    let _guard = TEST_LOCK.lock().unwrap();
    reset_test_state();

    let mut snapshot = AppSnapshot::new(
        WorkflowRuntimeStatus {
            state: WorkflowHealth::Ok,
            path: "/tmp/demo/WORKFLOW.md".to_string(),
            last_good_hash: Some("sha256:dispatch-ffi".to_string()),
            last_reload_at: Some("2026-03-23T16:30:00Z".to_string()),
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
    reset_shared_control_plane_snapshot_for_tests(snapshot);

    let response = dispatch_send_json("ses_stale", Some("task_review"), false, "rerun tests")
        .expect("dispatch response");
    let response_value: Value = serde_json::from_str(&response).expect("dispatch json");
    assert_eq!(response_value["events"][2]["state"], "failed");

    let snapshot_value: Value =
        serde_json::from_str(&state_snapshot_json().expect("snapshot json"))
            .expect("valid snapshot");
    assert_eq!(snapshot_value["attention"][0]["kind"], "session_error");
    assert_eq!(snapshot_value["attention"][0]["session_id"], "ses_stale");
}
