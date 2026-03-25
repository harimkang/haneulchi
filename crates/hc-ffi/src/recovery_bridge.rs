use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use hc_control_plane::{RecoveryContext, detect_degraded_issues, recovery_action_for_issue};
use hc_domain::settings::DegradedIssue;

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

pub fn degraded_issues_json(context_json: &str) -> Result<String, String> {
    let context: RecoveryContext = serde_json::from_str(context_json).map_err(|e| e.to_string())?;
    let issues = detect_degraded_issues(&context);
    serde_json::to_string(&issues).map_err(|e| e.to_string())
}

pub fn recovery_action_for_issue_json(issue_code: &str) -> Result<String, String> {
    let issue = DegradedIssue {
        issue_code: issue_code.to_string(),
        worktree_id: None,
        project_id: None,
        details: String::new(),
    };
    let action = recovery_action_for_issue(&issue);
    Ok(action.as_str().to_string())
}

#[unsafe(no_mangle)]
pub extern "C" fn hc_degraded_issues_json(context_json: *const c_char) -> HcString {
    string_to_hcstring(read_c_string(context_json).and_then(|s| degraded_issues_json(&s)))
}

#[unsafe(no_mangle)]
pub extern "C" fn hc_recovery_action_for_issue_json(issue_code: *const c_char) -> HcString {
    string_to_hcstring(
        read_c_string(issue_code).and_then(|code| recovery_action_for_issue_json(&code)),
    )
}
