use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use hc_domain::settings::{SecretRef, TerminalSettings};

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

pub fn terminal_settings_json() -> Result<String, String> {
    let settings = hc_control_plane::shared_get_terminal_settings().map_err(|e| e.to_string())?;
    serde_json::to_string(&settings).map_err(|e| e.to_string())
}

pub fn upsert_terminal_settings_json(json: &str) -> Result<String, String> {
    let settings: TerminalSettings = serde_json::from_str(json).map_err(|e| e.to_string())?;
    hc_control_plane::shared_upsert_terminal_settings(settings).map_err(|e| e.to_string())?;
    Ok(r#"{"ok":true}"#.to_string())
}

pub fn list_secret_refs_json() -> Result<String, String> {
    let refs = hc_control_plane::shared_list_secret_refs().map_err(|e| e.to_string())?;
    serde_json::to_string(&refs).map_err(|e| e.to_string())
}

pub fn upsert_secret_ref_json(json: &str) -> Result<String, String> {
    let secret_ref: SecretRef = serde_json::from_str(json).map_err(|e| e.to_string())?;
    hc_control_plane::shared_upsert_secret_ref(secret_ref).map_err(|e| e.to_string())?;
    Ok(r#"{"ok":true}"#.to_string())
}

pub fn delete_secret_ref_json(ref_id: &str) -> Result<String, String> {
    hc_control_plane::shared_delete_secret_ref(ref_id).map_err(|e| e.to_string())?;
    Ok(r#"{"ok":true}"#.to_string())
}

pub fn resolve_secret_env_json() -> Result<String, String> {
    // TODO(sprint-6): pass project_id as scope_filter once project context is threaded through launch
    let environment = hc_control_plane::shared_resolve_secret_env().map_err(|e| e.to_string())?;
    serde_json::to_string(&environment).map_err(|e| e.to_string())
}

#[unsafe(no_mangle)]
pub extern "C" fn hc_terminal_settings_json() -> HcString {
    string_to_hcstring(terminal_settings_json())
}

#[unsafe(no_mangle)]
pub extern "C" fn hc_upsert_terminal_settings_json(json: *const c_char) -> HcString {
    string_to_hcstring(read_c_string(json).and_then(|s| upsert_terminal_settings_json(&s)))
}

#[unsafe(no_mangle)]
pub extern "C" fn hc_list_secret_refs_json() -> HcString {
    string_to_hcstring(list_secret_refs_json())
}

#[unsafe(no_mangle)]
pub extern "C" fn hc_upsert_secret_ref_json(json: *const c_char) -> HcString {
    string_to_hcstring(read_c_string(json).and_then(|s| upsert_secret_ref_json(&s)))
}

#[unsafe(no_mangle)]
pub extern "C" fn hc_delete_secret_ref_json(ref_id: *const c_char) -> HcString {
    string_to_hcstring(read_c_string(ref_id).and_then(|id| delete_secret_ref_json(&id)))
}

#[unsafe(no_mangle)]
pub extern "C" fn hc_resolve_secret_env_json() -> HcString {
    string_to_hcstring(resolve_secret_env_json())
}
