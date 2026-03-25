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
    let text = unsafe { CStr::from_ptr(value) }
        .to_str()
        .map_err(|error| error.to_string())?;
    Ok(text.to_string())
}

pub fn inventory_summary_json(project_id: &str) -> Result<String, String> {
    let summary =
        hc_control_plane::shared_inventory_summary(project_id).map_err(|e| e.to_string())?;
    serde_json::to_string(&summary).map_err(|e| e.to_string())
}

pub fn inventory_list_json(project_id: &str) -> Result<String, String> {
    let rows =
        hc_control_plane::shared_inventory_for_project(project_id).map_err(|e| e.to_string())?;
    serde_json::to_string(&rows).map_err(|e| e.to_string())
}

pub fn set_worktree_pinned_json(worktree_id: &str, is_pinned: bool) -> Result<String, String> {
    hc_control_plane::shared_set_worktree_pinned(worktree_id, is_pinned)
        .map_err(|e| e.to_string())?;
    Ok(r#"{"ok":true}"#.to_string())
}

pub fn update_worktree_lifecycle_json(
    worktree_id: &str,
    new_state: &str,
) -> Result<String, String> {
    hc_control_plane::shared_update_worktree_lifecycle(worktree_id, new_state)
        .map_err(|e| e.to_string())?;
    Ok(r#"{"ok":true}"#.to_string())
}

#[unsafe(no_mangle)]
pub extern "C" fn hc_inventory_summary_json(project_id: *const c_char) -> HcString {
    string_to_hcstring(read_c_string(project_id).and_then(|id| inventory_summary_json(&id)))
}

#[unsafe(no_mangle)]
pub extern "C" fn hc_inventory_list_json(project_id: *const c_char) -> HcString {
    string_to_hcstring(read_c_string(project_id).and_then(|id| inventory_list_json(&id)))
}

#[unsafe(no_mangle)]
pub extern "C" fn hc_set_worktree_pinned_json(
    worktree_id: *const c_char,
    is_pinned: i32,
) -> HcString {
    string_to_hcstring(
        read_c_string(worktree_id).and_then(|id| set_worktree_pinned_json(&id, is_pinned != 0)),
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn hc_update_worktree_lifecycle_json(
    worktree_id: *const c_char,
    new_state: *const c_char,
) -> HcString {
    string_to_hcstring(read_c_string(worktree_id).and_then(|id| {
        read_c_string(new_state).and_then(|state| update_worktree_lifecycle_json(&id, &state))
    }))
}
