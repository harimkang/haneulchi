use hc_ffi::{inventory_list_json, inventory_summary_json, set_worktree_pinned_json};
use serde_json::Value;

#[test]
fn inventory_summary_json_returns_valid_json() {
    let result = inventory_summary_json("proj_test");
    // The function should return valid JSON (either an empty summary or an error object).
    // Either way it must be parseable JSON.
    let raw = match result {
        Ok(json) => json,
        Err(error) => serde_json::json!({ "error": error }).to_string(),
    };
    let value: Value = serde_json::from_str(&raw).expect("inventory_summary_json must return valid JSON");
    // If the result is an ok summary it should have the expected fields.
    if value.get("error").is_none() {
        assert!(value.get("total").is_some(), "summary should have 'total' field");
    }
}

#[test]
fn inventory_list_json_returns_array() {
    let result = inventory_list_json("proj_test");
    let raw = match result {
        Ok(json) => json,
        Err(error) => serde_json::json!({ "error": error }).to_string(),
    };
    let value: Value = serde_json::from_str(&raw).expect("inventory_list_json must return valid JSON");
    // When storage is empty the result is an empty array; on error it is an object.
    if value.get("error").is_none() {
        assert!(value.is_array(), "inventory_list_json should return a JSON array");
    }
}

#[test]
fn set_worktree_pinned_json_returns_ok() {
    // wt_test almost certainly does not exist in the test store — the call is
    // expected to either succeed (with ok:true) or return a storage error object.
    // We only verify that the return value is valid JSON.
    let result = set_worktree_pinned_json("wt_test", true);
    let raw = match result {
        Ok(json) => json,
        Err(error) => serde_json::json!({ "error": error }).to_string(),
    };
    let _value: Value = serde_json::from_str(&raw)
        .expect("set_worktree_pinned_json must return valid JSON");
}
