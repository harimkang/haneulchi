use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use hc_api::{review_decision_json as api_review_decision_json, review_queue_json as api_review_queue_json};

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

pub fn review_queue_json() -> Result<String, String> {
    api_review_queue_json()
}

pub fn review_decision_json(task_id: &str, decision: &str) -> Result<String, String> {
    api_review_decision_json(task_id, decision)
}

#[unsafe(no_mangle)]
pub extern "C" fn hc_review_queue_json() -> HcString {
    string_to_hcstring(review_queue_json())
}

#[unsafe(no_mangle)]
pub extern "C" fn hc_review_decision_json(
    task_id: *const c_char,
    decision: *const c_char,
) -> HcString {
    let payload = read_c_string(task_id)
        .and_then(|task_id| read_c_string(decision).and_then(|decision| review_decision_json(&task_id, &decision)));
    string_to_hcstring(payload)
}
