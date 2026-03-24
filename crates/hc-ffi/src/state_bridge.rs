use std::ffi::CString;
use std::os::raw::c_char;

use crate::HcString;
use crate::session_bridge::lock_runtime;

pub fn reset_control_plane_for_tests() {
    hc_control_plane::reset_shared_control_plane_for_tests();
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
    if std::env::var("HC_FORCE_SNAPSHOT_FAILURE").ok().as_deref() == Some("1") {
        return Err("snapshot_unavailable".to_string());
    }
    let runtime_snapshots = lock_runtime()?
        .list_snapshots()
        .map_err(|error| error.to_string())?;
    let mut control_plane = hc_control_plane::lock_shared_control_plane()?;
    if !(
        runtime_snapshots.is_empty()
            && (!control_plane.snapshot().sessions.is_empty()
                || !control_plane.snapshot().attention.is_empty()
                || !control_plane.snapshot().retry_queue.is_empty())
    ) {
        control_plane.sync_from_runtime(&runtime_snapshots);
    }
    serde_json::to_string(control_plane.snapshot()).map_err(|error| error.to_string())
}

pub fn sessions_list_json() -> Result<String, String> {
    let runtime_snapshots = lock_runtime()?
        .list_snapshots()
        .map_err(|error| error.to_string())?;
    let mut control_plane = hc_control_plane::lock_shared_control_plane()?;
    if !(
        runtime_snapshots.is_empty()
            && (!control_plane.snapshot().sessions.is_empty()
                || !control_plane.snapshot().attention.is_empty()
                || !control_plane.snapshot().retry_queue.is_empty())
    ) {
        control_plane.sync_from_runtime(&runtime_snapshots);
    }
    serde_json::to_string(&control_plane.snapshot().sessions).map_err(|error| error.to_string())
}

pub fn session_focus(session_id: &str) -> Result<(), String> {
    let runtime_snapshots = lock_runtime()?
        .list_snapshots()
        .map_err(|error| error.to_string())?;
    let mut control_plane = hc_control_plane::lock_shared_control_plane()?;
    if !(
        runtime_snapshots.is_empty()
            && (!control_plane.snapshot().sessions.is_empty()
                || !control_plane.snapshot().attention.is_empty()
                || !control_plane.snapshot().retry_queue.is_empty())
    ) {
        control_plane.sync_from_runtime(&runtime_snapshots);
    }
    control_plane.focus_session(session_id).map_err(|error| error.to_string())
}

pub fn session_takeover(session_id: &str) -> Result<(), String> {
    let runtime_snapshots = lock_runtime()?
        .list_snapshots()
        .map_err(|error| error.to_string())?;
    let mut control_plane = hc_control_plane::lock_shared_control_plane()?;
    if !(
        runtime_snapshots.is_empty()
            && (!control_plane.snapshot().sessions.is_empty()
                || !control_plane.snapshot().attention.is_empty()
                || !control_plane.snapshot().retry_queue.is_empty())
    ) {
        control_plane.sync_from_runtime(&runtime_snapshots);
    }
    control_plane
        .takeover_session(session_id)
        .map_err(|error| error.to_string())
}

pub fn session_release_takeover(session_id: &str) -> Result<(), String> {
    let runtime_snapshots = lock_runtime()?
        .list_snapshots()
        .map_err(|error| error.to_string())?;
    let mut control_plane = hc_control_plane::lock_shared_control_plane()?;
    if !(
        runtime_snapshots.is_empty()
            && (!control_plane.snapshot().sessions.is_empty()
                || !control_plane.snapshot().attention.is_empty()
                || !control_plane.snapshot().retry_queue.is_empty())
    ) {
        control_plane.sync_from_runtime(&runtime_snapshots);
    }
    control_plane
        .release_takeover_session(session_id)
        .map_err(|error| error.to_string())
}

pub fn session_attach_task_json(session_id: &str, task_id: &str) -> Result<String, String> {
    let runtime_snapshots = lock_runtime()?
        .list_snapshots()
        .map_err(|error| error.to_string())?;
    let mut control_plane = hc_control_plane::lock_shared_control_plane()?;
    if !(
        runtime_snapshots.is_empty()
            && (!control_plane.snapshot().sessions.is_empty()
                || !control_plane.snapshot().attention.is_empty()
                || !control_plane.snapshot().retry_queue.is_empty())
    ) {
        control_plane.sync_from_runtime(&runtime_snapshots);
    }
    control_plane
        .attach_task(session_id, task_id)
        .map_err(|error| error.to_string())?;
    serde_json::to_string(control_plane.snapshot()).map_err(|error| error.to_string())
}

pub fn session_detach_task_json(session_id: &str) -> Result<String, String> {
    let runtime_snapshots = lock_runtime()?
        .list_snapshots()
        .map_err(|error| error.to_string())?;
    let mut control_plane = hc_control_plane::lock_shared_control_plane()?;
    if !(runtime_snapshots.is_empty() && !control_plane.snapshot().sessions.is_empty()) {
        control_plane.sync_from_runtime(&runtime_snapshots);
    }
    control_plane
        .detach_task(session_id)
        .map_err(|error| error.to_string())?;
    serde_json::to_string(control_plane.snapshot()).map_err(|error| error.to_string())
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

#[unsafe(no_mangle)]
pub extern "C" fn hc_session_attach_task_json(
    session_id: *const c_char,
    task_id: *const c_char,
) -> HcString {
    let payload = read_c_string(session_id).and_then(|session_id| {
        read_c_string(task_id).and_then(|task_id| session_attach_task_json(&session_id, &task_id))
    });
    string_to_hcstring(payload)
}

#[unsafe(no_mangle)]
pub extern "C" fn hc_session_detach_task_json(session_id: *const c_char) -> HcString {
    string_to_hcstring(read_c_string(session_id).and_then(|session_id| session_detach_task_json(&session_id)))
}
