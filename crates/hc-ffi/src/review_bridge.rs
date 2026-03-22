use std::ffi::CString;

use hc_api::review_queue_json as api_review_queue_json;

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

pub fn review_queue_json() -> Result<String, String> {
    api_review_queue_json()
}

#[unsafe(no_mangle)]
pub extern "C" fn hc_review_queue_json() -> HcString {
    string_to_hcstring(review_queue_json())
}
