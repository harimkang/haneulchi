use std::ffi::CStr;

use hc_ffi::{hc_runtime_info_json, hc_string_free, runtime_info_json};
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
