use std::ffi::{CStr, CString};
use std::sync::Mutex;

use hc_ffi::{
    hc_session_focus, hc_session_release_takeover, hc_session_takeover, hc_state_snapshot_json,
    hc_string_free, hc_sessions_list_json, session_focus, session_release_takeover,
    session_takeover, state_snapshot_json, sessions_list_json, terminal_session_spawn_json,
    reset_test_state,
};
use serde_json::Value;

static TEST_LOCK: Mutex<()> = Mutex::new(());

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
    assert!(value.get("workflow").is_some());
    assert!(value.get("tracker").is_some());

    let sessions_json = sessions_list_json().expect("sessions list");
    let sessions_value: Value = serde_json::from_str(&sessions_json).expect("sessions array");
    assert!(sessions_value.is_array());
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
    let session_id = serde_json::from_str::<Value>(&spawned)
        .expect("spawn response")
        ["session_id"]
        .as_str()
        .unwrap()
        .to_string();

    session_focus(&session_id).expect("focus succeeds");
    session_takeover(&session_id).expect("takeover succeeds");
    session_release_takeover(&session_id).expect("release succeeds");

    let snapshot: Value =
        serde_json::from_str(&state_snapshot_json().expect("snapshot json")).expect("valid json");
    assert_eq!(snapshot["app"]["focused_session_id"], session_id);
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
        serde_json::from_str::<Value>(&spawned)
            .expect("spawn response")
            ["session_id"]
            .as_str()
            .unwrap()
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
    let spawned_id = serde_json::from_str::<Value>(&spawned)
        .expect("spawn response")
        ["session_id"]
        .as_str()
        .unwrap()
        .to_string();

    let snapshot: Value =
        serde_json::from_str(&state_snapshot_json().expect("snapshot json")).expect("valid json");
    let sessions = snapshot["sessions"].as_array().expect("sessions array");

    assert!(sessions.iter().any(|session| session["session_id"] == spawned_id));
    assert!(!sessions.iter().any(|session| session["session_id"] == "ses_01"));
}
