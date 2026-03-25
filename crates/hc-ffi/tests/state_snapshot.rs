use std::ffi::{CStr, CString};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use hc_control_plane::reset_shared_control_plane_snapshot_for_tests;
use hc_domain::{
    AppSnapshot, ClaimState, RetryState, SessionRuntimeState, SessionSummary, TrackerStatus,
    WorkflowHealth, WorkflowRuntimeStatus,
};
use hc_ffi::{
    hc_session_focus, hc_session_release_takeover, hc_session_takeover, hc_sessions_list_json,
    hc_state_snapshot_json, hc_string_free, reset_test_state, session_focus,
    session_release_takeover, session_takeover, sessions_list_json, state_snapshot_json,
    terminal_session_spawn_json, workflow_reload_json,
};
use serde_json::Value;

static TEST_LOCK: Mutex<()> = Mutex::new(());

fn temp_dir(label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("hc-state-snapshot-{label}-{unique}"));
    fs::create_dir_all(&path).expect("temp dir");
    path
}

#[test]
fn state_snapshot_json_contains_authoritative_top_level_groups() {
    let _guard = TEST_LOCK.lock().unwrap();
    reset_test_state();
    let json = state_snapshot_json().expect("snapshot json");
    let value: Value = serde_json::from_str(&json).expect("valid json");

    assert!(value.get("ops").is_some());
    assert!(value.get("projects").is_some());
    assert!(value.get("sessions").is_some());
    assert!(value.get("attention").is_some());
    assert!(value.get("retry_queue").is_some());
    assert!(value.get("workflow").is_none());
    assert!(value.get("tracker").is_none());
    assert!(value.get("app").is_none());
    assert!(value["ops"].get("automation").is_some());
    assert!(value["ops"].get("workflow").is_some());
    assert!(value["ops"].get("tracker").is_some());
    assert!(value["ops"].get("app").is_some());

    let sessions_json = sessions_list_json().expect("sessions list");
    let sessions_value: Value = serde_json::from_str(&sessions_json).expect("sessions array");
    assert!(sessions_value.is_array());
}

#[test]
fn state_snapshot_json_exposes_retry_claim_state_and_adapter_watch_fields() {
    let _guard = TEST_LOCK.lock().unwrap();
    reset_test_state();

    let workflow = WorkflowRuntimeStatus {
        state: WorkflowHealth::Ok,
        path: "/tmp/demo/WORKFLOW.md".to_string(),
        last_good_hash: Some("sha256:ffi".to_string()),
        last_reload_at: Some("2026-03-23T12:00:00Z".to_string()),
        last_error: None,
    };
    let tracker = TrackerStatus {
        state: "local_only".to_string(),
        last_sync_at: None,
        health: "ok".to_string(),
    };
    let mut session = SessionSummary::new("ses_watch", "proj_demo", "Watched adapter");
    session.runtime_state = SessionRuntimeState::WaitingInput;
    session.dispatch_state = "dispatchable".to_string();
    session.provider_id = Some("anthropic".to_string());
    session.model_id = Some("claude-sonnet-4".to_string());
    session.latest_commentary = Some("Need approval before rerunning the suite.".to_string());
    session.active_window_title = Some("Terminal 1".to_string());
    session.dispatch_reason = Some("dispatchable".to_string());

    reset_shared_control_plane_snapshot_for_tests(
        AppSnapshot::new(workflow, tracker)
            .with_session(session)
            .with_retry_entry(hc_domain::RetryQueueEntry {
                task_id: "task_retry".to_string(),
                project_id: "proj_demo".to_string(),
                attempt: 2,
                reason_code: "adapter_timeout".to_string(),
                due_at: Some("2026-03-23T12:05:00Z".to_string()),
                backoff_ms: 45_000,
                claim_state: ClaimState::Claimed,
                retry_state: RetryState::Due,
            }),
    );

    let snapshot: Value =
        serde_json::from_str(&state_snapshot_json().expect("snapshot json")).expect("valid json");
    let session = &snapshot["sessions"][0];

    assert_eq!(session["provider_id"], "anthropic");
    assert_eq!(session["model_id"], "claude-sonnet-4");
    assert_eq!(
        session["latest_commentary"],
        "Need approval before rerunning the suite."
    );
    assert_eq!(session["active_window_title"], "Terminal 1");
    assert_eq!(session["dispatch_reason"], "dispatchable");
    assert_eq!(snapshot["retry_queue"][0]["claim_state"], "claimed");
}

#[test]
fn session_commands_mutate_exported_snapshot() {
    let _guard = TEST_LOCK.lock().unwrap();
    reset_test_state();
    let spawned = terminal_session_spawn_json(
        r#"{
            "program": "/bin/sh",
            "args": ["-lc", "sleep 1"],
            "current_directory": "/tmp/demo",
            "geometry": { "cols": 80, "rows": 24 }
        }"#,
    )
    .expect("spawned session");
    let session_id = serde_json::from_str::<Value>(&spawned).expect("spawn response")["session_id"]
        .as_str()
        .unwrap()
        .to_string();

    session_focus(&session_id).expect("focus succeeds");
    session_takeover(&session_id).expect("takeover succeeds");
    session_release_takeover(&session_id).expect("release succeeds");

    let snapshot: Value =
        serde_json::from_str(&state_snapshot_json().expect("snapshot json")).expect("valid json");
    assert_eq!(snapshot["ops"]["app"]["focused_session_id"], session_id);
}

#[test]
fn c_abi_exports_state_and_session_command_surface() {
    let _guard = TEST_LOCK.lock().unwrap();
    reset_test_state();
    let snapshot = hc_state_snapshot_json();
    let snapshot_json = unsafe { CStr::from_ptr(snapshot.ptr) }
        .to_str()
        .unwrap()
        .to_string();
    hc_string_free(snapshot);
    let snapshot_value: Value = serde_json::from_str(&snapshot_json).unwrap();
    assert!(snapshot_value.get("sessions").is_some());

    let sessions = hc_sessions_list_json();
    let sessions_json = unsafe { CStr::from_ptr(sessions.ptr) }
        .to_str()
        .unwrap()
        .to_string();
    hc_string_free(sessions);
    let sessions_value: Value = serde_json::from_str(&sessions_json).unwrap();
    assert!(sessions_value.is_array());

    let spawned = terminal_session_spawn_json(
        r#"{
            "program": "/bin/sh",
            "args": ["-lc", "sleep 1"],
            "current_directory": "/tmp/demo",
            "geometry": { "cols": 80, "rows": 24 }
        }"#,
    )
    .expect("spawned session");
    let session_id = CString::new(
        serde_json::from_str::<Value>(&spawned).expect("spawn response")["session_id"]
            .as_str()
            .unwrap(),
    )
    .unwrap();
    assert_eq!(hc_session_focus(session_id.as_ptr()), 0);
    assert_eq!(hc_session_takeover(session_id.as_ptr()), 0);
    assert_eq!(hc_session_release_takeover(session_id.as_ptr()), 0);
}

#[test]
fn state_snapshot_reflects_spawned_runtime_sessions_instead_of_sample_stub() {
    let _guard = TEST_LOCK.lock().unwrap();
    reset_test_state();
    let spawned = terminal_session_spawn_json(
        r#"{
            "program": "/bin/sh",
            "args": ["-lc", "sleep 1"],
            "current_directory": "/tmp/demo",
            "geometry": { "cols": 80, "rows": 24 }
        }"#,
    )
    .expect("spawned session");
    let spawned_id = serde_json::from_str::<Value>(&spawned).expect("spawn response")["session_id"]
        .as_str()
        .unwrap()
        .to_string();

    let snapshot: Value =
        serde_json::from_str(&state_snapshot_json().expect("snapshot json")).expect("valid json");
    let sessions = snapshot["sessions"].as_array().expect("sessions array");

    assert!(
        sessions
            .iter()
            .any(|session| session["session_id"] == spawned_id)
    );
    assert!(
        !sessions
            .iter()
            .any(|session| session["session_id"] == "ses_01")
    );
}

#[test]
fn state_snapshot_keeps_last_known_good_after_auto_polled_invalid_reload() {
    let _guard = TEST_LOCK.lock().unwrap();
    reset_test_state();

    let root = temp_dir("workflow-auto-reload");
    let workflow_path = root.join("WORKFLOW.md");
    fs::write(
        &workflow_path,
        "---\nworkflow:\n  name: Valid Snapshot Workflow\n---\n{{task.title}}\n",
    )
    .expect("valid workflow");

    let initial_reload: Value = serde_json::from_str(
        &workflow_reload_json(root.to_str().expect("utf8 path")).expect("initial reload"),
    )
    .expect("valid json");
    let initial_hash = initial_reload["last_good_hash"]
        .as_str()
        .unwrap()
        .to_string();

    let spawn_payload = format!(
        r#"{{
            "program": "/bin/sh",
            "args": ["-lc", "sleep 2"],
            "current_directory": "{}",
            "geometry": {{ "cols": 80, "rows": 24 }}
        }}"#,
        root.display()
    );
    terminal_session_spawn_json(&spawn_payload).expect("spawn session");

    fs::write(
        &workflow_path,
        "---\nworkflow:\n  name: Broken\n  max_slots: [\n---\n{{task.title}}\n",
    )
    .expect("invalid workflow write");

    thread::sleep(Duration::from_millis(1100));

    let snapshot: Value =
        serde_json::from_str(&state_snapshot_json().expect("snapshot json")).expect("valid json");

    assert_eq!(
        snapshot["ops"]["workflow"]["state"],
        "invalid_kept_last_good"
    );
    assert_eq!(snapshot["ops"]["workflow"]["last_good_hash"], initial_hash);
}

#[test]
fn state_snapshot_surfaces_snapshot_unavailable_instead_of_empty_payload() {
    let _guard = TEST_LOCK.lock().unwrap();
    reset_test_state();
    unsafe {
        std::env::set_var("HC_FORCE_SNAPSHOT_FAILURE", "1");
    }

    let error = state_snapshot_json().expect_err("forced snapshot failure");
    assert_eq!(error, "snapshot_unavailable");

    let payload = hc_state_snapshot_json();
    let json = unsafe { CStr::from_ptr(payload.ptr) }
        .to_str()
        .unwrap()
        .to_string();
    hc_string_free(payload);
    let value: Value = serde_json::from_str(&json).expect("error json");
    assert_eq!(value["error"], "snapshot_unavailable");

    unsafe {
        std::env::remove_var("HC_FORCE_SNAPSHOT_FAILURE");
    }
}
