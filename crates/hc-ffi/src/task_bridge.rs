use std::ffi::CString;
use std::os::raw::c_char;

use hc_api::{task_move_json as api_task_move_json, tasks_list_json};

use crate::HcString;

fn read_required_c_string(value: *const c_char) -> Result<String, String> {
    if value.is_null() {
        return Err("null pointer".to_string());
    }

    let text = unsafe { std::ffi::CStr::from_ptr(value) }
        .to_str()
        .map_err(|error| error.to_string())?;

    Ok(text.to_string())
}

fn read_optional_c_string(value: *const c_char) -> Result<Option<String>, String> {
    if value.is_null() {
        return Ok(None);
    }

    let text = unsafe { std::ffi::CStr::from_ptr(value) }
        .to_str()
        .map_err(|error| error.to_string())?
        .trim()
        .to_string();

    if text.is_empty() {
        Ok(None)
    } else {
        Ok(Some(text))
    }
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

pub fn task_board_json(project_id: Option<String>) -> Result<String, String> {
    tasks_list_json(project_id.as_deref())
}

pub fn task_move_json(task_id: &str, column: &str) -> Result<String, String> {
    api_task_move_json(task_id, column)
}

#[unsafe(no_mangle)]
pub extern "C" fn hc_task_board_json(project_id: *const c_char) -> HcString {
    let payload = read_optional_c_string(project_id).and_then(task_board_json);
    string_to_hcstring(payload)
}

#[unsafe(no_mangle)]
pub extern "C" fn hc_task_move_json(task_id: *const c_char, column: *const c_char) -> HcString {
    let payload = read_required_c_string(task_id).and_then(|task_id| {
        read_required_c_string(column).and_then(|column| task_move_json(&task_id, &column))
    });
    string_to_hcstring(payload)
}
