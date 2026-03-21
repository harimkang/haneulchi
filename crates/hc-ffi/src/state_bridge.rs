use std::ffi::CString;
use std::os::raw::c_char;
use std::sync::{Mutex, OnceLock};

use hc_control_plane::ControlPlaneState;

use crate::HcString;

fn control_plane() -> &'static Mutex<ControlPlaneState> {
    static CONTROL_PLANE: OnceLock<Mutex<ControlPlaneState>> = OnceLock::new();
    CONTROL_PLANE.get_or_init(|| Mutex::new(ControlPlaneState::sample()))
}

fn lock_control_plane() -> Result<std::sync::MutexGuard<'static, ControlPlaneState>, String> {
    control_plane()
        .lock()
        .map_err(|_| "control plane lock poisoned".to_string())
}

fn read_c_string(value: *const c_char) -> Result<String, String> {
    if value.is_null() {
        return Err("null pointer".to_string());
    }

    let text = unsafe { std::ffi::CStr::from_ptr(value) }
        .to_str()
        .map_err(|error| error.to_string())?;

    Ok(text.to_string())
}

fn string_to_hcstring(value: Result<String, String>) -> HcString {
    let payload = match value {
        Ok(value) => value,
        Err(error) => serde_json::json!({ "error": error }).to_string(),
    };
    let string = CString::new(payload).expect("json payload is nul-free");

    HcString {
        ptr: string.into_raw(),
    }
}

pub fn state_snapshot_json() -> Result<String, String> {
    let control_plane = lock_control_plane()?;
    serde_json::to_string(control_plane.snapshot()).map_err(|error| error.to_string())
}

pub fn sessions_list_json() -> Result<String, String> {
    let control_plane = lock_control_plane()?;
    serde_json::to_string(&control_plane.snapshot().sessions).map_err(|error| error.to_string())
}

pub fn session_focus(session_id: &str) -> Result<(), String> {
    lock_control_plane()?
        .focus_session(session_id)
        .map_err(|error| error.to_string())
}

pub fn session_takeover(session_id: &str) -> Result<(), String> {
    lock_control_plane()?
        .takeover_session(session_id)
        .map_err(|error| error.to_string())
}

pub fn session_release_takeover(session_id: &str) -> Result<(), String> {
    lock_control_plane()?
        .release_takeover_session(session_id)
        .map_err(|error| error.to_string())
}

#[unsafe(no_mangle)]
pub extern "C" fn hc_state_snapshot_json() -> HcString {
    string_to_hcstring(state_snapshot_json())
}

#[unsafe(no_mangle)]
pub extern "C" fn hc_sessions_list_json() -> HcString {
    string_to_hcstring(sessions_list_json())
}

#[unsafe(no_mangle)]
pub extern "C" fn hc_session_focus(session_id: *const c_char) -> i32 {
    let result = read_c_string(session_id).and_then(|session_id| session_focus(&session_id));
    if result.is_ok() { 0 } else { 1 }
}

#[unsafe(no_mangle)]
pub extern "C" fn hc_session_takeover(session_id: *const c_char) -> i32 {
    let result = read_c_string(session_id).and_then(|session_id| session_takeover(&session_id));
    if result.is_ok() { 0 } else { 1 }
}

#[unsafe(no_mangle)]
pub extern "C" fn hc_session_release_takeover(session_id: *const c_char) -> i32 {
    let result =
        read_c_string(session_id).and_then(|session_id| session_release_takeover(&session_id));
    if result.is_ok() { 0 } else { 1 }
}
