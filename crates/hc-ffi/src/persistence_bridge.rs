use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use crate::HcString;

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

fn read_c_string(value: *const c_char) -> Result<String, String> {
    if value.is_null() {
        return Err("null pointer".to_string());
    }
    unsafe { CStr::from_ptr(value) }
        .to_str()
        .map(|s| s.to_string())
        .map_err(|e| e.to_string())
}

#[derive(serde::Deserialize)]
struct SaveAppStateInput {
    route: String,
    last_project_id: Option<String>,
    last_session_id: Option<String>,
}

pub fn load_app_state_json() -> Result<String, String> {
    let opt = hc_control_plane::shared_load_app_state()
        .map_err(|e| e.to_string())?;
    match opt {
        None => Ok("null".to_string()),
        Some(record) => serde_json::to_string(&serde_json::json!({
            "active_route": record.active_route,
            "last_project_id": record.last_project_id,
            "last_session_id": record.last_session_id,
            "saved_at": record.saved_at,
        }))
        .map_err(|e| e.to_string()),
    }
}

pub fn save_app_state_json(json: &str) -> Result<String, String> {
    let input: SaveAppStateInput =
        serde_json::from_str(json).map_err(|e| format!("parse_error: {e}"))?;
    hc_control_plane::shared_save_app_state(
        &input.route,
        input.last_project_id.as_deref(),
        input.last_session_id.as_deref(),
    )
    .map_err(|e| e.to_string())?;
    Ok(serde_json::json!({ "ok": true }).to_string())
}

pub fn list_recoverable_sessions_json(project_id: &str) -> Result<String, String> {
    let records = hc_control_plane::shared_list_recoverable_sessions(project_id)
        .map_err(|e| e.to_string())?;
    let items: Vec<_> = records
        .into_iter()
        .map(|r| {
            serde_json::json!({
                "session_id": r.session_id,
                "project_id": r.project_id,
                "title": r.title,
                "cwd": r.cwd,
                "branch": r.branch,
                "last_active_at": r.last_active_at,
                "is_recoverable": r.is_recoverable,
            })
        })
        .collect();
    serde_json::to_string(&items).map_err(|e| e.to_string())
}

#[unsafe(no_mangle)]
pub extern "C" fn hc_load_app_state_json() -> HcString {
    string_to_hcstring(load_app_state_json())
}

#[unsafe(no_mangle)]
pub extern "C" fn hc_save_app_state_json(json: *const c_char) -> HcString {
    string_to_hcstring(read_c_string(json).and_then(|j| save_app_state_json(&j)))
}

#[unsafe(no_mangle)]
pub extern "C" fn hc_list_recoverable_sessions_json(project_id: *const c_char) -> HcString {
    string_to_hcstring(
        read_c_string(project_id).and_then(|pid| list_recoverable_sessions_json(&pid)),
    )
}
