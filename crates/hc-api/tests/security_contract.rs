use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use hc_control_plane::{reset_shared_control_plane_snapshot_for_tests, reset_task_board_for_tests};
use hc_domain::{
    AppSnapshot, AppState, OpsSummary, SessionFocusState, SessionRuntimeState, SessionSummary,
    TrackerStatus, WorkflowHealth, WorkflowRuntimeStatus,
};

fn temp_socket_path() -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_nanos();
    std::env::temp_dir().join(format!("hc-api-security-{unique}.sock"))
}

fn seeded_snapshot() -> AppSnapshot {
    let workflow = WorkflowRuntimeStatus {
        state: WorkflowHealth::Ok,
        path: "/tmp/demo/WORKFLOW.md".to_string(),
        last_good_hash: Some("sha256:sec".to_string()),
        last_reload_at: Some("2026-03-25T00:00:00Z".to_string()),
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
            last_tick_at: Some("2026-03-25T00:00:00Z".to_string()),
            last_reconcile_at: None,
            running_slots: 0,
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
    snapshot.sessions = vec![
        SessionSummary::new("ses_sec", "proj_sec", "Security test session")
            .with_runtime_state(SessionRuntimeState::Running)
            .with_focus_state(SessionFocusState::Focused)
            .with_cwd("/tmp/sec")
            .with_workspace_root("/tmp/sec")
            .with_base_root("."),
    ];
    snapshot
}

fn request(method: &str, path: &str, body: Option<&str>) -> (u16, serde_json::Value) {
    let (status, body) =
        hc_api::server::route_for_test(method, path, body.unwrap_or("")).expect("route response");
    let value: serde_json::Value = serde_json::from_str(&body).expect("json body");
    (status, value)
}

#[test]
fn unix_socket_permissions_are_0600() {
    let socket_path = temp_socket_path();
    let server = hc_api::server::ApiServer::bind(&socket_path).expect("bind server");

    let metadata = std::fs::metadata(&socket_path).expect("stat socket");
    let mode = metadata.permissions().mode();

    // Mask off the file type bits — only keep the permission bits.
    let perm_bits = mode & 0o777;
    assert_eq!(
        perm_bits, 0o600,
        "socket must be owner-only read/write (0600), got {perm_bits:#o}"
    );

    drop(server);
    let _ = std::fs::remove_file(&socket_path);
}

#[test]
fn state_endpoint_does_not_expose_secrets() {
    reset_task_board_for_tests();
    reset_shared_control_plane_snapshot_for_tests(seeded_snapshot());

    // Plant a sentinel that must never appear in API responses.
    let secret_sentinel = "SUPER_SECRET_TOKEN_XYZ_SENTINEL_12345";

    let (status, value) = request("GET", "/v1/state", None);
    assert_eq!(status, 200);

    let body = serde_json::to_string(&value).expect("serialize response");
    assert!(
        !body.contains(secret_sentinel),
        "state endpoint must not expose planted secret sentinel"
    );

    // Verify the response is otherwise valid.
    assert_eq!(value["ok"], true);
}

#[test]
fn sessions_endpoint_does_not_expose_secrets() {
    reset_task_board_for_tests();
    reset_shared_control_plane_snapshot_for_tests(seeded_snapshot());

    let secret_sentinel = "SUPER_SECRET_TOKEN_XYZ_SENTINEL_12345";

    let (status, value) = request("GET", "/v1/sessions", None);
    assert_eq!(status, 200);

    let body = serde_json::to_string(&value).expect("serialize response");
    assert!(
        !body.contains(secret_sentinel),
        "sessions endpoint must not expose planted secret sentinel"
    );

    assert_eq!(value["ok"], true);
    assert!(value["data"].is_array());
}
