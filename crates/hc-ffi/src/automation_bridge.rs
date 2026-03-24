use std::ffi::CString;

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

pub fn reconcile_automation_json() -> Result<String, String> {
    hc_api::reconcile_now_json(None)
}

#[unsafe(no_mangle)]
pub extern "C" fn hc_reconcile_now_json() -> HcString {
    string_to_hcstring(reconcile_automation_json())
}
