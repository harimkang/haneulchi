//! Swift bridge scaffold aligned with the single-authority snapshot contract.

use std::ffi::CString;
use std::os::raw::c_char;

use hc_runtime::terminal::backend::TerminalBackendDescriptor;

mod session_bridge;
mod state_bridge;
mod workflow_bridge;

#[repr(C)]
pub struct HcString {
    pub ptr: *mut c_char,
}

#[repr(C)]
pub struct HcBytes {
    pub ptr: *mut u8,
    pub len: usize,
}

pub fn runtime_info_json() -> String {
    let backend = TerminalBackendDescriptor::recommended();
    serde_json::json!({
        "renderer_id": backend.id,
        "transport": "ffi_c_abi",
        "demo_mode": true,
    })
    .to_string()
}

#[unsafe(no_mangle)]
pub extern "C" fn hc_runtime_info_json() -> HcString {
    let string = CString::new(runtime_info_json()).expect("runtime info json is nul-free");

    HcString {
        ptr: string.into_raw(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn hc_string_free(value: HcString) {
    if value.ptr.is_null() {
        return;
    }

    unsafe {
        let _ = CString::from_raw(value.ptr);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn hc_bytes_free(value: HcBytes) {
    if value.ptr.is_null() {
        return;
    }

    unsafe {
        let _ = Vec::from_raw_parts(value.ptr, value.len, value.len);
    }
}

pub use session_bridge::{
    hc_terminal_session_drain, hc_terminal_session_resize, hc_terminal_session_snapshot_json,
    hc_terminal_session_spawn_json, hc_terminal_session_terminate, hc_terminal_session_write,
    terminal_session_drain, terminal_session_resize, terminal_session_snapshot_json,
    terminal_session_spawn_json, terminal_session_terminate, terminal_session_write,
};
pub use state_bridge::{
    hc_session_focus, hc_session_release_takeover, hc_session_takeover, hc_state_snapshot_json,
    hc_sessions_list_json, session_focus, session_release_takeover, session_takeover,
    state_snapshot_json, sessions_list_json,
};
pub use workflow_bridge::{
    hc_workflow_reload_json, hc_workflow_validate_json, workflow_reload_json,
    workflow_validate_json,
};
