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

pub fn attention_resolve_json(attention_id: &str) -> Result<String, String> {
    let mut control_plane = hc_control_plane::lock_shared_control_plane()?;
    control_plane
        .resolve_attention(attention_id)
        .map_err(|error| error.to_string())?;
    serde_json::to_string(control_plane.snapshot()).map_err(|error| error.to_string())
}

pub fn attention_dismiss_json(attention_id: &str) -> Result<String, String> {
    let mut control_plane = hc_control_plane::lock_shared_control_plane()?;
    control_plane
        .dismiss_attention(attention_id)
        .map_err(|error| error.to_string())?;
    serde_json::to_string(control_plane.snapshot()).map_err(|error| error.to_string())
}

pub fn attention_snooze_json(attention_id: &str) -> Result<String, String> {
    let mut control_plane = hc_control_plane::lock_shared_control_plane()?;
    control_plane
        .snooze_attention(attention_id)
        .map_err(|error| error.to_string())?;
    serde_json::to_string(control_plane.snapshot()).map_err(|error| error.to_string())
}

#[unsafe(no_mangle)]
pub extern "C" fn hc_attention_resolve_json(attention_id: *const c_char) -> HcString {
    string_to_hcstring(read_c_string(attention_id).and_then(|value| attention_resolve_json(&value)))
}

#[unsafe(no_mangle)]
pub extern "C" fn hc_attention_dismiss_json(attention_id: *const c_char) -> HcString {
    string_to_hcstring(read_c_string(attention_id).and_then(|value| attention_dismiss_json(&value)))
}

#[unsafe(no_mangle)]
pub extern "C" fn hc_attention_snooze_json(attention_id: *const c_char) -> HcString {
    string_to_hcstring(read_c_string(attention_id).and_then(|value| attention_snooze_json(&value)))
}
