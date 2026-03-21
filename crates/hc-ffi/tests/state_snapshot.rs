use std::ffi::{CStr, CString};

use hc_ffi::{
    hc_session_focus, hc_session_release_takeover, hc_session_takeover, hc_state_snapshot_json,
    hc_string_free, hc_sessions_list_json, session_focus, session_release_takeover,
    session_takeover, state_snapshot_json, sessions_list_json,
};
use serde_json::Value;

#[test]
fn state_snapshot_json_contains_authoritative_top_level_groups() {
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
    session_focus("ses_02").expect("focus succeeds");
    session_takeover("ses_02").expect("takeover succeeds");
    session_release_takeover("ses_02").expect("release succeeds");

    let snapshot: Value =
        serde_json::from_str(&state_snapshot_json().expect("snapshot json")).expect("valid json");
    assert_eq!(snapshot["app"]["focused_session_id"], "ses_02");
}

#[test]
fn c_abi_exports_state_and_session_command_surface() {
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

    let session_id = CString::new("ses_02").unwrap();
    assert_eq!(hc_session_focus(session_id.as_ptr()), 0);
    assert_eq!(hc_session_takeover(session_id.as_ptr()), 0);
    assert_eq!(hc_session_release_takeover(session_id.as_ptr()), 0);
}
