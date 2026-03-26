use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

use hc_control_plane::{reset_shared_control_plane_snapshot_for_tests, reset_task_board_for_tests};
use hc_domain::{
    AppSnapshot, AppState, AttentionSummary, ClaimState, OpsSummary, RetryQueueEntry, RetryState,
    SessionFocusState, SessionRuntimeState, SessionSummary, TrackerStatus, WorkflowHealth,
    WorkflowRuntimeStatus,
};

fn temp_socket_path() -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_nanos();
    std::env::temp_dir().join(format!("hc-api-contract-{unique}.sock"))
}

fn seeded_snapshot() -> AppSnapshot {
    let workflow = WorkflowRuntimeStatus {
        state: WorkflowHealth::Ok,
        path: "/tmp/demo/WORKFLOW.md".to_string(),
        last_good_hash: Some("sha256:api".to_string()),
        last_reload_at: Some("2026-03-23T18:00:00Z".to_string()),
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
            last_tick_at: Some("2026-03-23T18:00:00Z".to_string()),
            last_reconcile_at: None,
            running_slots: 1,
            max_slots: 2,
            retry_due_count: 0,
            queued_claim_count: 0,
            paused: false,
        })
        .with_app_state(AppState {
            active_route: "project_focus".to_string(),
            focused_session_id: Some("ses_api".to_string()),
            degraded_flags: vec![],
        });
    snapshot.sessions = vec![
        SessionSummary::new("ses_api", "proj_demo", "Primary API session")
            .with_runtime_state(SessionRuntimeState::Running)
            .with_focus_state(SessionFocusState::Focused)
            .with_cwd("/tmp/demo")
            .with_workspace_root("/tmp/demo")
            .with_base_root("."),
        SessionSummary::new("ses_other", "proj_other", "Secondary API session")
            .with_runtime_state(SessionRuntimeState::WaitingInput)
            .with_focus_state(SessionFocusState::Background)
            .with_cwd("/tmp/other")
            .with_workspace_root("/tmp/other")
            .with_base_root("."),
    ];
    snapshot.attention = vec![AttentionSummary {
        attention_id: "att_api".to_string(),
        kind: "retry_due".to_string(),
        project_id: "proj_demo".to_string(),
        session_id: None,
        task_id: Some("task_ready".to_string()),
        title: "Retry due".to_string(),
        summary: "Task is ready to retry".to_string(),
        created_at: Some("2026-03-23T18:00:30Z".to_string()),
        severity: "warn".to_string(),
        action_hint: Some("open_retry_queue".to_string()),
    }];
    snapshot.retry_queue = vec![RetryQueueEntry {
        task_id: "task_ready".to_string(),
        project_id: "proj_demo".to_string(),
        attempt: 2,
        reason_code: "adapter_timeout".to_string(),
        due_at: Some("2026-03-23T18:01:00Z".to_string()),
        backoff_ms: 60_000,
        claim_state: ClaimState::Claimed,
        retry_state: RetryState::Due,
    }];
    snapshot
}

fn request(method: &str, path: &str, body: Option<&str>) -> (u16, serde_json::Value) {
    let (status, body) =
        hc_api::server::route_for_test(method, path, body.unwrap_or("")).expect("route response");
    let value: serde_json::Value = serde_json::from_str(&body).expect("json body");
    (status, value)
}

#[test]
fn uds_server_contract_covers_state_sessions_tasks_workflow_dispatch_and_reconcile() {
    reset_task_board_for_tests();
    reset_shared_control_plane_snapshot_for_tests(seeded_snapshot());

    let socket_path = temp_socket_path();
    let workflow_root = std::env::temp_dir().join("hc-api-workflow-contract");
    std::fs::create_dir_all(&workflow_root).expect("workflow root");
    std::fs::write(
        workflow_root.join("WORKFLOW.md"),
        "---\nworkflow:\n  name: API Workflow\n---\n{{task.title}}\n",
    )
    .expect("workflow file");

    let _server = hc_api::server::ApiServer::bind(&socket_path).expect("bind server");

    let (status, value) = request("GET", "/v1/state", None);
    assert_eq!(status, 200);
    assert_eq!(value["ok"], true);
    assert!(value["data"]["sessions"].is_array());
    assert!(value["meta"]["request_id"].is_string());

    let (status, project_state) =
        request("GET", "/v1/state?project_id=proj_demo&view=compact", None);
    assert_eq!(status, 200);
    assert_eq!(
        project_state["data"]["sessions"]
            .as_array()
            .expect("sessions")
            .len(),
        1
    );
    assert_eq!(
        project_state["data"]["sessions"][0]["project_id"],
        "proj_demo"
    );
    assert_eq!(
        project_state["data"]["attention"]
            .as_array()
            .expect("attention")
            .len(),
        0
    );
    assert_eq!(
        project_state["data"]["retry_queue"]
            .as_array()
            .expect("retry queue")
            .len(),
        0
    );

    let (status, filtered_state) = request(
        "GET",
        "/v1/state?project_id=proj_demo&include_attention=false&include_retry_queue=false",
        None,
    );
    assert_eq!(status, 200);
    assert_eq!(
        filtered_state["data"]["attention"]
            .as_array()
            .expect("attention")
            .len(),
        0
    );
    assert_eq!(
        filtered_state["data"]["retry_queue"]
            .as_array()
            .expect("retry queue")
            .len(),
        0
    );

    let (status, automation) = request("GET", "/v1/automation", None);
    assert_eq!(status, 200);
    assert_eq!(automation["data"]["automation"]["running_slots"], 1);
    assert_eq!(automation["data"]["workflow"]["state"], "ok");

    let (status, sessions) = request("GET", "/v1/sessions", None);
    assert_eq!(status, 200);
    assert_eq!(sessions["data"][0]["session_id"], "ses_api");

    let (status, filtered_sessions) = request(
        "GET",
        "/v1/sessions?project_id=proj_demo&state=running",
        None,
    );
    assert_eq!(status, 200);
    assert_eq!(
        filtered_sessions["data"]
            .as_array()
            .expect("sessions")
            .len(),
        1
    );
    assert_eq!(filtered_sessions["data"][0]["session_id"], "ses_api");

    let (status, session_details) = request("GET", "/v1/sessions/ses_api", None);
    assert_eq!(status, 200);
    assert_eq!(session_details["data"]["session_id"], "ses_api");
    assert!(session_details["data"]["recent_events"].is_array());
    assert_eq!(session_details["data"]["workflow_binding"]["state"], "ok");

    let (status, _) = request(
        "POST",
        "/v1/sessions/ses_api/focus",
        Some(r#"{"activate_app":true}"#),
    );
    assert_eq!(status, 202);

    let (status, _) = request(
        "POST",
        "/v1/sessions/ses_api/takeover",
        Some(r#"{"reason":"manual review"}"#),
    );
    assert_eq!(status, 200);

    let (status, _) = request(
        "POST",
        "/v1/sessions/ses_api/release-takeover",
        Some(r#"{"resume_mode":"normal"}"#),
    );
    assert_eq!(status, 200);

    let (status, _) = request(
        "POST",
        "/v1/sessions/ses_api/attach-task",
        Some(r#"{"task_id":"task_ready"}"#),
    );
    assert_eq!(status, 200);

    let (status, _) = request("POST", "/v1/sessions/ses_api/detach-task", Some("{}"));
    assert_eq!(status, 200);

    let (status, tasks) = request("GET", "/v1/tasks", None);
    assert_eq!(status, 200);
    assert!(tasks["data"]["columns"].is_array());

    let (status, created_task) = request(
        "POST",
        "/v1/tasks",
        Some(r#"{"project_id":"proj_demo","title":"API created task"}"#),
    );
    assert_eq!(status, 200);
    let created_task_id = created_task["data"]["id"]
        .as_str()
        .expect("task id")
        .to_string();

    let (status, _) = request(
        "POST",
        &format!("/v1/tasks/{created_task_id}/move"),
        Some(r#"{"column":"Review"}"#),
    );
    assert_eq!(status, 200);

    let (status, _) = request(
        "POST",
        &format!("/v1/tasks/{created_task_id}/automation-mode"),
        Some(r#"{"mode":"assisted"}"#),
    );
    assert_eq!(status, 200);

    let (status, dispatch) = request(
        "POST",
        "/v1/dispatch",
        Some(
            r#"{"target_session_id":"ses_api","task_id":"task_ready","target_live":true,"payload":"run tests"}"#,
        ),
    );
    assert_eq!(status, 200);
    assert_eq!(dispatch["data"]["events"][2]["state"], "sent");

    let (status, workflow_validate) = request(
        "POST",
        "/v1/workflow/validate",
        Some(&format!(
            r#"{{"project_root":"{}"}}"#,
            workflow_root.display()
        )),
    );
    assert_eq!(status, 200);
    assert_eq!(workflow_validate["data"]["state"], "ok");

    let (status, workflow_reload) = request(
        "POST",
        "/v1/workflow/reload",
        Some(&format!(
            r#"{{"project_root":"{}"}}"#,
            workflow_root.display()
        )),
    );
    assert_eq!(status, 200);
    assert!(workflow_reload["data"]["state"].is_string());

    let (status, reconcile) = request(
        "POST",
        "/v1/reconcile",
        Some(r#"{"project_id":"proj_demo"}"#),
    );
    assert_eq!(status, 200);
    assert_eq!(reconcile["data"]["ok"], true);
    assert_eq!(reconcile["data"]["project_id"], "proj_demo");
    let _ = std::fs::remove_file(socket_path);
}

#[test]
fn uds_server_writes_real_http_response_over_unix_socket() {
    reset_task_board_for_tests();
    reset_shared_control_plane_snapshot_for_tests(seeded_snapshot());

    let socket_path = temp_socket_path();
    let server = hc_api::server::ApiServer::bind(&socket_path).expect("bind server");
    let handle = thread::spawn(move || server.serve_requests(1));
    thread::sleep(Duration::from_millis(50));

    let mut stream = UnixStream::connect(&socket_path).expect("connect uds");
    stream
        .write_all(b"GET /v1/state HTTP/1.1\r\nHost: localhost\r\nContent-Length: 0\r\n\r\n")
        .expect("write request");
    stream
        .shutdown(std::net::Shutdown::Write)
        .expect("shutdown write");
    let mut response = String::new();
    stream.read_to_string(&mut response).expect("read response");

    assert!(response.starts_with("HTTP/1.1 200"));
    assert!(response.contains("\r\n\r\n"));
    assert!(response.contains("X-HC-Api-Version: 1"));
    assert!(response.contains("X-HC-Snapshot-Rev: "));

    handle.join().expect("server join").expect("server result");
    let _ = std::fs::remove_file(socket_path);
}

#[test]
fn uds_bind_rejects_live_socket_owner_and_recovers_stale_socket() {
    let live_socket_path = temp_socket_path();
    let live_owner = hc_api::server::ApiServer::bind(&live_socket_path).expect("bind live owner");

    let conflict = match hc_api::server::ApiServer::bind(&live_socket_path) {
        Ok(_) => panic!("expected live owner rejection"),
        Err(error) => error,
    };
    assert!(conflict.contains("socket_already_owned"));

    drop(live_owner);
    let _ = std::fs::remove_file(&live_socket_path);

    let stale_socket_path = temp_socket_path();
    let stale_listener =
        std::os::unix::net::UnixListener::bind(&stale_socket_path).expect("create stale socket");
    drop(stale_listener);
    assert!(stale_socket_path.exists());

    let rebound =
        hc_api::server::ApiServer::bind(&stale_socket_path).expect("bind after stale cleanup");
    drop(rebound);
    let _ = std::fs::remove_file(stale_socket_path);
}

#[test]
fn uds_bind_creates_parent_directories_for_nested_socket_paths() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_nanos();
    let socket_path = std::env::temp_dir()
        .join(format!("hc-api-bind-{unique}"))
        .join("run")
        .join("control.sock");

    let server = hc_api::server::ApiServer::bind(&socket_path).expect("bind nested socket");

    assert!(socket_path.exists());

    drop(server);
    let _ = std::fs::remove_file(&socket_path);
    let _ = std::fs::remove_dir_all(socket_path.parent().and_then(|path| path.parent()).unwrap());
}

#[test]
fn uds_server_returns_typed_status_codes_for_domain_failures() {
    reset_task_board_for_tests();
    reset_shared_control_plane_snapshot_for_tests(seeded_snapshot());

    let (status, focus_error) = request("POST", "/v1/sessions/ses_missing/focus", Some("{}"));
    assert_eq!(status, 404);
    assert_eq!(focus_error["ok"], false);
    assert_eq!(focus_error["error"]["code"], "session_not_found");
    assert_eq!(focus_error["error"]["retryable"], false);
    assert!(focus_error["error"]["details"].is_object());

    let (status, conflict) = request(
        "POST",
        "/v1/dispatch",
        Some(
            r#"{"target_session_id":"ses_api","task_id":"task_ready","target_live":false,"payload":"run tests"}"#,
        ),
    );
    assert_eq!(status, 409);
    assert_eq!(conflict["ok"], false);
    assert_eq!(conflict["error"]["code"], "stale_target_session");
}
