use serde_json::json;

use hc_control_plane::{reload_workflow, validate_workflow};
use hc_workflow::WorkflowState;

pub fn workflow_validate_json(project_root: &str) -> Result<String, String> {
    match validate_workflow(project_root.to_string()).map_err(|error| error.to_string())? {
        Some(loaded) => serde_json::to_string(&json!({
            "state": "ok",
            "path": loaded.discovery_path,
            "last_good_hash": loaded.contract_hash,
            "last_reload_at": serde_json::Value::Null,
            "last_error": serde_json::Value::Null
        }))
        .map_err(|error| error.to_string()),
        None => {
            serde_json::to_string(&json!({ "state": "none" })).map_err(|error| error.to_string())
        }
    }
}

pub fn workflow_reload_json(project_root: &str) -> Result<String, String> {
    let runtime = reload_workflow(project_root.to_string()).map_err(|error| error.to_string())?;
    serde_json::to_string(&json!({
        "state": match runtime.state() {
            WorkflowState::None => "none",
            WorkflowState::Ok => "ok",
            WorkflowState::InvalidKeptLastGood => "invalid_kept_last_good",
            WorkflowState::ReloadPending => "reload_pending",
        },
        "path": runtime.current().map(|loaded| loaded.discovery_path.clone()),
        "last_good_hash": runtime.last_known_good().map(|loaded| loaded.contract_hash.clone()),
        "last_reload_at": runtime.last_reload_at(),
        "last_error": runtime.last_error()
    }))
    .map_err(|error| error.to_string())
}
