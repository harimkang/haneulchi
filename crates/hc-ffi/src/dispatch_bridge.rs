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

pub fn dispatch_send_json(
    target_session_id: &str,
    task_id: Option<&str>,
    target_live: bool,
    payload: &str,
) -> Result<String, String> {
    hc_api::dispatch_send_json(target_session_id, task_id, target_live, payload)
}

#[unsafe(no_mangle)]
pub extern "C" fn hc_dispatch_send_json(
    target_session_id: *const c_char,
    task_id: *const c_char,
    target_live: bool,
    payload: *const c_char,
) -> HcString {
    let result = read_c_string(target_session_id).and_then(|target_session_id| {
        let task_id = if task_id.is_null() {
            Ok(None)
        } else {
            read_c_string(task_id).map(Some)
        }?;
        read_c_string(payload).and_then(|payload| {
            dispatch_send_json(
                &target_session_id,
                task_id.as_deref(),
                target_live,
                &payload,
            )
        })
    });
    string_to_hcstring(result)
}
