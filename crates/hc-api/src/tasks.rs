use hc_control_plane::{
    ReviewDecision, shared_provision_task_workspace, shared_review_decision,
    shared_review_ready_projection, shared_set_automation_mode, shared_task_board_projection,
    shared_task_drawer, shared_task_move, shared_create_task,
};
use hc_domain::TaskColumn;

pub fn tasks_list_json(project_id: Option<&str>) -> Result<String, String> {
    let projection = shared_task_board_projection(project_id).map_err(|error| error.to_string())?;
    serde_json::to_string(&projection).map_err(|error| error.to_string())
}

pub fn task_move_json(task_id: &str, column: &str) -> Result<String, String> {
    let column = column
        .parse::<TaskColumn>()
        .map_err(|invalid| format!("unknown task column: {invalid}"))?;
    let result =
        shared_task_move(task_id, column, "ffi_task_move").map_err(|error| error.to_string())?;
    serde_json::to_string(&result).map_err(|error| error.to_string())
}

pub fn task_create_json(project_id: &str, title: &str, _priority: Option<&str>) -> Result<String, String> {
    let task = shared_create_task(project_id, title, _priority).map_err(|error| error.to_string())?;
    serde_json::to_string(&task).map_err(|error| error.to_string())
}

pub fn task_automation_mode_json(task_id: &str, mode: &str) -> Result<String, String> {
    let mode = mode
        .parse::<hc_domain::TaskAutomationMode>()
        .map_err(|invalid| format!("unknown automation mode: {invalid}"))?;
    let task = shared_set_automation_mode(task_id, mode).map_err(|error| error.to_string())?;
    serde_json::to_string(&task).map_err(|error| error.to_string())
}

pub fn task_drawer_json(task_id: &str) -> Result<String, String> {
    let projection = shared_task_drawer(task_id).map_err(|error| error.to_string())?;
    serde_json::to_string(&projection).map_err(|error| error.to_string())
}

pub fn task_provision_workspace_json(
    project_root: &str,
    task_id: &str,
    base_root: Option<&str>,
) -> Result<String, String> {
    let workspace = shared_provision_task_workspace(project_root, task_id, base_root)
        .map_err(|error| error.to_string())?;
    serde_json::to_string(&workspace).map_err(|error| error.to_string())
}

pub fn review_queue_json() -> Result<String, String> {
    let projection = shared_review_ready_projection().map_err(|error| error.to_string())?;
    serde_json::to_string(&projection).map_err(|error| error.to_string())
}

pub fn review_decision_json(task_id: &str, decision: &str) -> Result<String, String> {
    let decision = match decision {
        "accept" => ReviewDecision::Accept,
        "request_changes" => ReviewDecision::RequestChanges,
        "manual_continue" => ReviewDecision::ManualContinue,
        "follow_up" => ReviewDecision::FollowUp,
        other => return Err(format!("unknown review decision: {other}")),
    };
    let result = shared_review_decision(task_id, decision).map_err(|error| error.to_string())?;
    serde_json::to_string(&result).map_err(|error| error.to_string())
}
