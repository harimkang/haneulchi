use hc_control_plane::{
    ReviewQueueService, shared_provision_task_workspace, shared_task_board_projection,
    shared_task_drawer, shared_task_move,
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
    let projection = ReviewQueueService::demo()
        .and_then(|service| service.review_ready_projection())
        .map_err(|error| error.to_string())?;
    serde_json::to_string(&projection).map_err(|error| error.to_string())
}
