use std::ffi::CStr;

use hc_ffi::{hc_runtime_info_json, hc_string_free, runtime_info_json, runtime_info_summary_json};
use hc_runtime::terminal::backend::TerminalBackendDescriptor;
use serde_json::Value;

#[test]
fn runtime_info_json_helper_matches_contract() {
    let json = runtime_info_json();
    let value: Value = serde_json::from_str(&json).expect("valid runtime info json");

    assert_eq!(
        value["renderer_id"],
        TerminalBackendDescriptor::recommended().id
    );
    assert_eq!(value["transport"], "ffi_c_abi");
    assert_eq!(value["demo_mode"], true);
}

#[test]
fn c_abi_returns_json_string_and_can_be_freed() {
    let string = hc_runtime_info_json();
    assert!(!string.ptr.is_null());

    let json = unsafe { CStr::from_ptr(string.ptr) }
        .to_str()
        .expect("utf8");
    let value: Value = serde_json::from_str(json).expect("valid runtime info json");

    assert_eq!(
        value["renderer_id"],
        TerminalBackendDescriptor::recommended().id
    );
    assert_eq!(value["transport"], "ffi_c_abi");
    assert_eq!(value["demo_mode"], true);

    hc_string_free(string);
}

#[test]
fn runtime_info_includes_socket_path_when_server_started() {
    let json = runtime_info_summary_json();
    let value: Value =
        serde_json::from_str(&json).expect("runtime_info_summary_json must return valid JSON");

    // The summary must always include these fields.
    assert!(
        value.get("socket_path").is_some(),
        "runtime info summary must include socket_path field"
    );
    assert_eq!(value["transport"], "unix_domain_socket_local_only");
    assert!(
        value["status"] == "running" || value["status"] == "not_started",
        "status must be 'running' or 'not_started', got: {}",
        value["status"]
    );

    // When server has not been started in this test process the socket_path
    // field should be null (or absent/empty).
    // We do not start the server here so we expect not_started or running
    // (depending on whether another test in the binary started it first).
    // Either way: the field must be present and parseable.
    let _ = value["socket_path"].clone();
}
