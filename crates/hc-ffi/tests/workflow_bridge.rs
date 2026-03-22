use std::ffi::{CStr, CString};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use hc_ffi::{
    hc_string_free, hc_workflow_reload_json, hc_workflow_validate_json, workflow_reload_json,
    workflow_validate_json,
};
use serde_json::Value;

fn temp_dir(label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("hc-ffi-workflow-{label}-{unique}"));
    fs::create_dir_all(&path).expect("temp dir");
    path
}

fn write_workflow(root: &Path, contents: &str) {
    fs::write(root.join("WORKFLOW.md"), contents).expect("workflow file");
}

#[test]
fn workflow_validate_and_reload_commands_return_runtime_status() {
    let root = temp_dir("workflow-bridge");
    write_workflow(
        &root,
        "---\nworkflow:\n  name: Bridge Workflow\n---\n{{task.title}}\n",
    );

    let validated: Value = serde_json::from_str(
        &workflow_validate_json(root.to_str().expect("utf8 path")).expect("validate json"),
    )
    .expect("valid json");
    assert_eq!(validated["state"], "ok");
    assert_eq!(validated["workflow"]["name"], "Bridge Workflow");

    let reloaded: Value = serde_json::from_str(
        &workflow_reload_json(root.to_str().expect("utf8 path")).expect("reload json"),
    )
    .expect("valid json");
    assert_eq!(reloaded["state"], "ok");
    assert!(reloaded["last_reload_at"].as_str().is_some());

    let root_c = CString::new(root.to_str().expect("utf8 path")).unwrap();
    let c_payload = hc_workflow_validate_json(root_c.as_ptr());
    let c_json = unsafe { CStr::from_ptr(c_payload.ptr) }
        .to_str()
        .unwrap()
        .to_string();
    hc_string_free(c_payload);
    let c_value: Value = serde_json::from_str(&c_json).unwrap();
    assert_eq!(c_value["state"], "ok");

    let reload_payload = hc_workflow_reload_json(root_c.as_ptr());
    let reload_json = unsafe { CStr::from_ptr(reload_payload.ptr) }
        .to_str()
        .unwrap()
        .to_string();
    hc_string_free(reload_payload);
    let reload_value: Value = serde_json::from_str(&reload_json).unwrap();
    assert_eq!(reload_value["state"], "ok");
    assert!(reload_value["last_reload_at"].as_str().is_some());
}

#[test]
fn invalid_reload_surfaces_kept_last_good_status_and_error_details() {
    let root = temp_dir("workflow-invalid-reload");
    let workflow_path = root.join("WORKFLOW.md");
    write_workflow(
        &root,
        "---\nworkflow:\n  name: Valid Workflow\n---\n{{task.title}}\n",
    );

    let valid: Value = serde_json::from_str(
        &workflow_reload_json(root.to_str().expect("utf8 path")).expect("valid reload"),
    )
    .expect("valid json");
    let initial_hash = valid["last_good_hash"].as_str().unwrap().to_string();

    fs::write(
        &workflow_path,
        "---\nworkflow:\n  name: Broken\n  max_slots: [\n---\n{{task.title}}\n",
    )
    .expect("invalid workflow write");

    let invalid: Value = serde_json::from_str(
        &workflow_reload_json(root.to_str().expect("utf8 path")).expect("invalid reload payload"),
    )
    .expect("valid json");

    assert_eq!(invalid["state"], "invalid_kept_last_good");
    assert_eq!(invalid["last_good_hash"], initial_hash);
    assert!(invalid["last_error"].as_str().unwrap().contains("front matter parse error"));
}
