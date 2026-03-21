use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use hc_control_plane::{reload_workflow, validate_workflow};
use hc_workflow::WorkflowState;

use crate::HcString;

fn read_c_string(value: *const c_char) -> Result<String, String> {
    if value.is_null() {
        return Err("null pointer".to_string());
    }

    let text = unsafe { CStr::from_ptr(value) }
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

pub fn workflow_validate_json(project_root: &str) -> Result<String, String> {
    match validate_workflow(project_root.to_string()).map_err(|error| error.to_string())? {
        Some(loaded) => serde_json::to_string(&serde_json::json!({
            "state": "ok",
            "workflow": {
                "name": loaded.effective_config.workflow.name,
                "contract_hash": loaded.contract_hash,
                "path": loaded.discovery_path,
            }
        }))
        .map_err(|error| error.to_string()),
        None => serde_json::to_string(&serde_json::json!({ "state": "none" }))
            .map_err(|error| error.to_string()),
    }
}

pub fn workflow_reload_json(project_root: &str) -> Result<String, String> {
    let runtime = reload_workflow(project_root.to_string()).map_err(|error| error.to_string())?;
    serde_json::to_string(&serde_json::json!({
        "state": match runtime.state() {
            WorkflowState::None => "none",
            WorkflowState::Ok => "ok",
            WorkflowState::InvalidKeptLastGood => "invalid_kept_last_good",
            WorkflowState::ReloadPending => "reload_pending",
        }
    }))
    .map_err(|error| error.to_string())
}

#[unsafe(no_mangle)]
pub extern "C" fn hc_workflow_validate_json(project_root: *const c_char) -> HcString {
    string_to_hcstring(read_c_string(project_root).and_then(|root| workflow_validate_json(&root)))
}

#[unsafe(no_mangle)]
pub extern "C" fn hc_workflow_reload_json(project_root: *const c_char) -> HcString {
    string_to_hcstring(read_c_string(project_root).and_then(|root| workflow_reload_json(&root)))
}
